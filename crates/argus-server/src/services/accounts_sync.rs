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
use argus_accounts::sync::backup::{self, UndoReport};
use argus_accounts::sync::{
    Compare, Drift, Env, Mode, Progress, Summary, build_compare, drift_of, lock_running, run,
    scan_and_plan,
};
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

/// The result of an undo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undone {
    /// The backup that was put back.
    pub backup_dir: String,
    /// What was restored.
    pub report: UndoReport,
}

fn user_projects() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    Path::new(&home).join(".claude").join("projects")
}

impl AccountsService {
    /// Whether a sync is running now.
    #[must_use]
    pub fn sync_active(&self) -> bool {
        self.sync.active.load(Ordering::SeqCst)
    }

    fn backups(&self) -> Result<&Path, String> {
        self.sync
            .backups
            .get()
            .map(PathBuf::as_path)
            .ok_or_else(|| "the accounts sync is not started".to_owned())
    }

    /// Runs `f` over the job's environment: the app's data, the catalog and the sidebar.
    fn with_env<T>(&self, f: impl FnOnce(&Env) -> T) -> Result<T, String> {
        let config = self.config();
        let app = find_app_data(config.app_data.as_deref())
            .ok_or_else(|| "the Claude Desktop data folder was not found".to_owned())?;
        let catalog = FsCatalog::for_app(&app);
        let transcripts = FsTranscripts::scan(&user_projects())?;
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

    fn run_sync(&self, _turn: &Turn<'_>, dry_run: bool) -> Result<Summary, String> {
        let mode = if dry_run { Mode::DryRun } else { Mode::Apply };
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
            run(env, mode, &self.docs, &mut on_progress)
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
        if let Ok(summary) = &result {
            self.put_drift(&settled(mode, summary));
        }
        result
    }

    fn put_run(&self, doc: &Value) {
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

    /// The rows of the Compare view.
    ///
    /// # Errors
    /// When the chats or the sidebar cannot be read.
    pub fn compare(&self) -> Result<Compare, String> {
        let accounts = self.config().accounts;
        self.with_env(|env| scan_and_plan(env).map(|p| build_compare(&p, &accounts)))?
    }

    /// Counts the drift now, without a run; skipped while a sync or a switch runs.
    fn scan_drift(&self) {
        if accounts_switch::is_running() {
            return;
        }
        let Ok(_turn) = self.sync.take() else {
            return;
        };
        match self.with_env(|env| scan_and_plan(env).map(|p| drift_of(&p))) {
            Ok(Ok(drift)) => self.put_drift(&drift),
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

fn merge(into: &mut Value, from: serde_json::Map<String, Value>) {
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
