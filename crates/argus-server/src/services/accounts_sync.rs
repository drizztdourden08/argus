//! The chat and folder sync of the accounts service: runs of the sync job one at a time, the
//! `run`, `drift` and `SyncSteps` docs they write, the compare rows and the undo of a backup. A
//! timer scans for drift every ten minutes.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use argus_accounts::catalog::{FsCatalog, FsTranscripts};
use argus_accounts::find_app_data;
use argus_accounts::sidebar::LevelDbSidebar;
use argus_accounts::sync::Route;
use argus_accounts::sync::backup::{self, UndoReport};
use argus_accounts::sync::{
    Compare, Drift, Env, Mode, Progress, Summary, build_compare, chat_counts, drift_of,
    lock_running, run, run_onto, scan_and_plan, scan_and_plan_onto,
};
use argus_accounts::types::{Account, Scope};
use argus_core::Coll;
use argus_core::clock::now_ms;
use serde_json::{Value, json};

use super::AccountsService;
use super::accounts_switch;
use super::tasks::Timer;
use crate::Services;
use crate::adapters::accounts_threads::SystemNow;

const NAME: &str = "accounts";
/// What a second sync answers while one runs; the route turns it into a 409.
pub const SYNC_BUSY: &str = "a sync is already running";
const DRIFT_EVERY: Duration = Duration::from_secs(600);
/// How long a switch waits for a run or a drift scan that holds the turn.
const TURN_WAIT: Duration = Duration::from_secs(120);
const TURN_POLL: Duration = Duration::from_millis(200);

/// Where the backups go and whether a run is active.
#[derive(Default)]
pub(crate) struct SyncState {
    backups: OnceLock<PathBuf>,
    active: AtomicBool,
}

/// Holds the right to run; released when dropped.
struct Turn<'a>(&'a AtomicBool);

impl Drop for Turn<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl SyncState {
    fn take(&self) -> Result<Turn<'_>, String> {
        if self.active.swap(true, Ordering::SeqCst) {
            Err(SYNC_BUSY.to_owned())
        } else {
            Ok(Turn(&self.active))
        }
    }

    /// Like [`Self::take`], but waits up to `limit` for the turn.
    fn take_waiting(&self, limit: Duration) -> Result<Turn<'_>, String> {
        let started = Instant::now();
        loop {
            match self.take() {
                Ok(turn) => return Ok(turn),
                Err(busy) if started.elapsed() >= limit => return Err(busy),
                Err(_) => std::thread::sleep(TURN_POLL),
            }
        }
    }
}

/// The turn of a run that outlives the call that started it, like a review run in the
/// background; released when dropped.
pub(crate) struct OwnedTurn(Arc<AccountsService>);

impl Drop for OwnedTurn {
    fn drop(&mut self) {
        self.0.sync.active.store(false, Ordering::SeqCst);
    }
}

/// The result of an undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undone {
    /// The backup that was put back.
    pub backup_dir: String,
    /// What was restored.
    pub report: UndoReport,
}

impl AccountsService {
    /// Whether a sync is running now.
    #[must_use]
    pub fn sync_active(&self) -> bool {
        self.sync.active.load(Ordering::SeqCst)
    }

    /// Takes the turn for a run that goes on in the background.
    ///
    /// # Errors
    /// [`SYNC_BUSY`] while another run holds it.
    pub(crate) fn take_owned(self: &Arc<Self>) -> Result<OwnedTurn, String> {
        if self.sync.active.swap(true, Ordering::SeqCst) {
            Err(SYNC_BUSY.to_owned())
        } else {
            Ok(OwnedTurn(Arc::clone(self)))
        }
    }

    fn backups(&self) -> Result<&Path, String> {
        self.sync
            .backups
            .get()
            .map(PathBuf::as_path)
            .ok_or_else(|| "the accounts sync is not started".to_owned())
    }

    /// Runs `f` over the job's environment: the app's data, the catalog and the sidebar.
    pub(crate) fn with_env<T>(&self, f: impl FnOnce(&Env) -> T) -> Result<T, String> {
        let config = self.config();
        let app = find_app_data(config.app_data.as_deref())
            .ok_or_else(|| "the Claude Desktop data folder was not found".to_owned())?;
        let catalog = FsCatalog::for_app(&app);
        let transcripts = FsTranscripts::scan(&self.user_claude().join("projects"))?;
        let sidebar = LevelDbSidebar::new(&app);
        let running = || lock_running(&app);
        let env = Env {
            app: &app,
            backups: self.backups()?,
            accounts: &config.accounts,
            rule: config.archive_rule,
            catalog: &catalog,
            transcripts: &transcripts,
            sidebar: &sidebar,
            clock: &SystemNow,
            running: &running,
        };
        Ok(f(&env))
    }

    /// [`Self::try_sync`], but when a run or a drift scan holds the turn it waits for it, up to
    /// two minutes. The switch uses this so a scan never makes it fail; the web route uses
    /// `try_sync` and answers 409 instead.
    ///
    /// # Errors
    /// [`SYNC_BUSY`] when the turn did not come free in time, or the reason the run failed.
    pub fn sync(&self, dry_run: bool) -> Result<Summary, String> {
        let turn = self.sync.take_waiting(TURN_WAIT)?;
        self.run_sync(&turn, dry_run)
    }

    /// Plans (a dry run) or applies the sync of chats and folders between the accounts.
    /// Blocking, and one run at a time. The `run` doc follows the phases, the `SyncSteps` docs
    /// the steps of an apply, and the `drift` doc is rewritten when the run ends.
    ///
    /// # Errors
    /// [`SYNC_BUSY`] while another run is active, or the reason the run failed.
    pub fn try_sync(&self, dry_run: bool) -> Result<Summary, String> {
        let turn = self.sync.take()?;
        self.run_sync(&turn, dry_run)
    }

    /// The one-way copy of `from` onto `to`, as the switch runs it: the target ends up as the
    /// current account is. Like [`Self::sync`] it waits up to two minutes for the turn.
    ///
    /// # Errors
    /// [`SYNC_BUSY`] when the turn did not come free in time, an account with no Desktop scope,
    /// or the reason the run failed.
    pub fn copy_onto(
        &self,
        from: &Account,
        to: &Account,
        dry_run: bool,
    ) -> Result<Summary, String> {
        let route = route_of(from, to)?;
        let turn = self.sync.take_waiting(TURN_WAIT)?;
        self.run_onto_route(&turn, &route, dry_run)
    }

    /// [`Self::copy_onto`] for the web route: answers [`SYNC_BUSY`] instead of waiting.
    ///
    /// # Errors
    /// [`SYNC_BUSY`] while another run is active, an account with no Desktop scope, or the
    /// reason the run failed.
    pub fn try_copy_onto(
        &self,
        from: &Account,
        to: &Account,
        dry_run: bool,
    ) -> Result<Summary, String> {
        let route = route_of(from, to)?;
        let turn = self.sync.take()?;
        self.run_onto_route(&turn, &route, dry_run)
    }

    fn run_onto_route(
        &self,
        turn: &Turn<'_>,
        route: &Route,
        dry_run: bool,
    ) -> Result<Summary, String> {
        let mode = if dry_run { Mode::DryRun } else { Mode::Apply };
        let result = self.run_job(turn, |env, on_progress| {
            run_onto(env, route, mode, &self.docs, on_progress)
        });
        if mode == Mode::Apply && result.is_ok() {
            // A copy leaves the two-way drift as it was; count it again for the cards.
            self.scan_drift_unturned();
        }
        result
    }

    fn run_sync(&self, turn: &Turn<'_>, dry_run: bool) -> Result<Summary, String> {
        let mode = if dry_run { Mode::DryRun } else { Mode::Apply };
        let result = self.run_job(turn, |env, on_progress| {
            run(env, mode, &self.docs, on_progress)
        });
        if let Ok(summary) = &result {
            self.put_drift(&settled(mode, summary));
        }
        result
    }

    /// Runs one job over the environment and keeps the `run` doc of it.
    fn run_job(
        &self,
        _turn: &Turn<'_>,
        job: impl FnOnce(&Env, &mut dyn FnMut(&Progress)) -> Result<Summary, String>,
    ) -> Result<Summary, String> {
        let started = now_ms();
        let mut doc = json!({
            "id": format!("SY-{started}"), "phase": "scan", "step": 0, "total": 0,
            "backupDir": null, "startedAt": started, "finishedAt": null, "result": null,
        });
        let result = self.with_env(|env| {
            let mut on_progress = |p: &Progress| {
                if let Ok(Value::Object(fields)) = serde_json::to_value(p) {
                    merge(&mut doc, fields);
                }
                self.put_run(&doc);
            };
            job(env, &mut on_progress)
        })?;
        let end = match &result {
            Ok(summary) => json!(summary),
            Err(e) => json!({ "error": e }),
        };
        merge(
            &mut doc,
            json!({ "finishedAt": now_ms(), "result": end })
                .as_object()
                .cloned()
                .unwrap_or_default(),
        );
        self.put_run(&doc);
        result
    }

    pub(crate) fn put_run(&self, doc: &Value) {
        if let Err(e) = self.docs.put(Coll::AccountsState, "run", doc.clone()) {
            self.log
                .error("accounts run doc not written", json!({ "error": e }));
        }
    }

    fn put_drift(&self, drift: &Drift) {
        if let Err(e) = self.docs.put(Coll::AccountsState, "drift", drift.to_doc()) {
            self.log
                .error("accounts drift doc not written", json!({ "error": e }));
        }
    }

    /// How many changes the one-way copy of `from` onto `to` would still make now; the switch
    /// checks with it that the copy held once Claude started again. Blocking.
    ///
    /// # Errors
    /// When an account has no Desktop scope, or the chats or the sidebar cannot be read.
    pub(crate) fn copy_left(&self, from: &Account, to: &Account) -> Result<usize, String> {
        let route = route_of(from, to)?;
        self.with_env(|env| scan_and_plan_onto(env, &route).map(|p| drift_of(&p).total))?
    }

    /// The rows of the Compare view.
    ///
    /// # Errors
    /// When the chats or the sidebar cannot be read.
    pub fn compare(&self) -> Result<Compare, String> {
        let accounts = self.config().accounts;
        self.with_env(|env| scan_and_plan(env).map(|p| build_compare(&p, &accounts)))?
    }

    /// Counts the drift now, without a run; skipped while a sync or a switch runs, and before
    /// any account is known (the first tick can come before discovery).
    pub(crate) fn scan_drift(&self) {
        self.scan_drift_with(self.sync.take());
    }

    /// The start-up scan: it waits for a tick that holds the turn instead of skipping, so the
    /// account cards get their counts right after discovery.
    pub(crate) fn scan_drift_now(&self) {
        self.scan_drift_with(self.sync.take_waiting(TURN_WAIT));
    }

    fn scan_drift_with(&self, turn: Result<Turn<'_>, String>) {
        if accounts_switch::is_running() || self.config().accounts.is_empty() {
            return;
        }
        let Ok(_turn) = turn else {
            return;
        };
        self.scan_drift_unturned();
    }

    /// The drift scan itself; the caller holds the turn.
    fn scan_drift_unturned(&self) {
        let accounts = self.config().accounts;
        let scanned = self
            .with_env(|env| scan_and_plan(env).map(|p| (drift_of(&p), chat_counts(&p, &accounts))));
        match scanned {
            Ok(Ok((drift, counts))) => {
                self.put_drift(&drift);
                if let Err(e) = self.docs.put_chat_counts(&accounts, &counts) {
                    self.log
                        .error("accounts chat counts not written", json!({ "error": e }));
                }
            }
            Ok(Err(e)) | Err(e) => self
                .log
                .warn("accounts drift scan failed", json!({ "error": e })),
        }
    }

    /// Puts a backup back: the one in `dir`, which must be under the backups folder, else the
    /// last one. Blocking, and not while a sync runs. Files that changed since the sync are
    /// kept and reported unless `force`; every file that is overwritten is copied into the
    /// backup's `after-apply` first.
    ///
    /// # Errors
    /// [`SYNC_BUSY`], no backup, a path outside the backups, or why the undo failed (Claude
    /// holds the storage, a copy was damaged).
    pub fn undo_sync(&self, dir: Option<&str>, force: bool) -> Result<Undone, String> {
        let _turn = self.sync.take()?;
        let root = self.backups()?;
        let chosen = match dir {
            Some(d) => inside(root, Path::new(d))?,
            None => backup::last(root).ok_or("there is no backup to undo")?,
        };
        let report = backup::undo(&chosen, force)?;
        Ok(Undone {
            backup_dir: chosen.display().to_string(),
            report,
        })
    }
}

/// The route of a copy: the Desktop scope of each account.
fn route_of(from: &Account, to: &Account) -> Result<Route, String> {
    let scope = |a: &Account| match (&a.desktop_account, &a.desktop_org) {
        (Some(account), Some(org)) => Ok(Scope {
            account: account.clone(),
            org: org.clone(),
        }),
        _ => Err(format!(
            "Open Claude once on {} before the first copy",
            a.label
        )),
    };
    Ok(Route {
        source: scope(from)?,
        target: scope(to)?,
    })
}

pub(crate) fn merge(into: &mut Value, from: serde_json::Map<String, Value>) {
    if let Some(map) = into.as_object_mut() {
        map.extend(from);
    }
}

/// What the drift is after a run: a dry run's counts, nothing after a verified apply.
fn settled(mode: Mode, summary: &Summary) -> Drift {
    if mode == Mode::DryRun || !summary.verified {
        return summary.drift;
    }
    Drift {
        scanned_at: now_ms(),
        ..Drift::default()
    }
}

/// `dir` when it is a folder directly under `root`.
fn inside(root: &Path, dir: &Path) -> Result<PathBuf, String> {
    let direct = dir.parent() == Some(root) && dir.file_name().is_some();
    if direct && !dir.to_string_lossy().contains("..") {
        Ok(dir.to_path_buf())
    } else {
        Err(format!("{} is not a backup folder", dir.display()))
    }
}

/// Where the backups go: `<DASHBOARD_HOME>\accounts\backups`.
pub(crate) fn attach(svc: &Services, accounts: &AccountsService) {
    let dir = svc.paths.home.join("accounts").join("backups");
    let _ = accounts.sync.backups.set(dir);
}

/// The drift scan, every ten minutes, when the accounts timer is on.
pub(crate) fn start(svc: &Arc<Services>, accounts: &Arc<AccountsService>) {
    if !svc.tasks.runs(Timer::Accounts) {
        return;
    }
    let accounts = Arc::clone(accounts);
    let task = tokio::spawn(async move {
        loop {
            let scanning = Arc::clone(&accounts);
            if tokio::task::spawn_blocking(move || scanning.scan_drift())
                .await
                .is_err()
            {
                return;
            }
            tokio::time::sleep(DRIFT_EVERY).await;
        }
    });
    svc.tasks.keep(NAME, task);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_waiting_turn_comes_when_the_holder_lets_go() {
        let state = SyncState::default();
        std::thread::scope(|scope| {
            let held = state.take().unwrap();
            scope.spawn(move || {
                std::thread::sleep(Duration::from_millis(100));
                drop(held);
            });
            assert!(state.take_waiting(Duration::from_secs(5)).is_ok());
        });
    }

    #[test]
    fn a_waiting_turn_gives_up_busy() {
        let state = SyncState::default();
        let _held = state.take().unwrap();
        let started = Instant::now();
        let busy = state.take_waiting(Duration::from_millis(300)).err();
        assert_eq!(busy.as_deref(), Some(SYNC_BUSY));
        assert!(started.elapsed() >= Duration::from_millis(300));
    }

    #[test]
    fn a_plain_take_does_not_wait() {
        let state = SyncState::default();
        let _held = state.take().unwrap();
        assert_eq!(state.take().err().as_deref(), Some(SYNC_BUSY));
    }
}
