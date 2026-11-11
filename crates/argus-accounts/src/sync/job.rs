//! The sync job: scan the chats and the sidebar, plan what would change, and, when asked, apply
//! it with the Desktop app closed: backup, the chat files, the sidebar storage, then a re-plan
//! that must come out empty. A dry run stops after the plan. Every step goes to a [`StepSink`]
//! and every phase change to a progress callback.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::backup;
use super::drift::{Drift, drift_of};
use super::steps::{StepKind, StepSink, StepState, SyncStep};
use crate::catalog::merge::LINEAGE_KEYS;
use crate::catalog::plan::{Change, ChangeKind, Plan, build_plan, is_syncable};
use crate::catalog::{StepStatus, apply};
use crate::config::ArchiveRule;
use crate::ports::{Catalog, Clock, SidebarStore, Transcripts};
use crate::sidebar::store::ensure_unlocked;
use crate::sidebar::{
    FolderPlan, FolderStep, Presence, SidebarKeys, fresh_folder_id, plan_folders,
};
use crate::types::{Account, Scope};

/// Whether the run only plans or also writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Plan and report; nothing is written.
    DryRun,
    /// Back up, write, verify.
    Apply,
}

/// Where a run is, as the `run` doc names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// Reading the chats and the sidebar.
    Scan,
    /// Working out the changes.
    Plan,
    /// Copying away what will be written.
    Backup,
    /// Writing.
    Apply,
    /// Planning again to check.
    Verify,
    /// Finished.
    Done,
    /// Stopped by an error.
    Failed,
}

/// The fields of the `run` doc the job owns.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// Where the run is.
    pub phase: Phase,
    /// Steps finished.
    pub step: u64,
    /// Steps the run will have.
    pub total: u64,
    /// The backup folder, once it exists.
    pub backup_dir: Option<String>,
}

/// What a run did or, for a dry run, would do.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    /// The mode of the run.
    pub mode: Mode,
    /// The changes found, by kind.
    pub drift: Drift,
    /// Chats left alone because their copies disagree.
    pub conflicts: usize,
    /// Chats left alone because no valid transcript was found.
    pub missing_transcripts: usize,
    /// Whether the re-plan after the writes was empty; false for a dry run.
    pub verified: bool,
    /// The backup folder of an apply that wrote something.
    pub backup_dir: Option<String>,
}

/// Everything the job reads from, bundled.
pub struct Env<'a> {
    /// The Desktop app's data folder.
    pub app: &'a Path,
    /// Where the backups go: `<DASHBOARD_HOME>\accounts\backups`.
    pub backups: &'a Path,
    /// The known accounts, for the names in the steps.
    pub accounts: &'a [Account],
    /// Which archive state wins.
    pub rule: ArchiveRule,
    /// The chat records.
    pub catalog: &'a dyn Catalog,
    /// The transcripts.
    pub transcripts: &'a dyn Transcripts,
    /// The sidebar storage.
    pub sidebar: &'a dyn SidebarStore,
    /// The time.
    pub clock: &'a dyn Clock,
    /// Whether Claude Desktop runs; an apply refuses when it does.
    pub running: &'a dyn Fn() -> bool,
}

/// Whether Claude Desktop holds the lock of its Local Storage: an exclusive open of the `LOCK`
/// file fails while the app runs. No process is listed.
#[must_use]
pub fn lock_running(app: &Path) -> bool {
    ensure_unlocked(&app.join("Local Storage").join("leveldb")).is_err()
}

/// One chat as a scope holds it.
#[derive(Debug, Clone)]
pub struct Chat {
    /// The title.
    pub title: String,
    /// Archived by its record or its scope's index.
    pub archived: bool,
    /// `lastFocusedAt`, else `lastActivityAt`, else 0.
    pub focused: i64,
    /// The chat it was spawned from, unless the link was detached.
    pub parent: Option<String>,
    /// The record as the app wrote it.
    pub record: Value,
}

/// The chats of one scope.
#[derive(Debug, Clone)]
pub struct ScopeScan {
    /// The scope.
    pub scope: Scope,
    /// Its chats by id.
    pub chats: BTreeMap<String, Chat>,
}

/// What the catalog and the sidebar hold now.
#[derive(Debug, Clone)]
pub struct Scan {
    /// When the scan ran, epoch milliseconds.
    pub at: i64,
    /// Every scope, sorted by name.
    pub scopes: Vec<ScopeScan>,
    /// The sidebar keys.
    pub keys: SidebarKeys,
}

/// A scan and what a sync would change in it.
#[derive(Debug, Clone)]
pub struct Planned {
    /// The scan the plan is from.
    pub scan: Scan,
    /// The chat file changes.
    pub catalog: Plan,
    /// The step kind of each chat file change, in order.
    pub kinds: Vec<StepKind>,
    /// The folder changes and the keys after them.
    pub folders: FolderPlan,
}

impl Planned {
    /// Whether the plan changes nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.catalog.changes.is_empty() && self.folders.steps.is_empty()
    }
}

fn millis(record: &Value, key: &str) -> Option<i64> {
    let v = record.get(key)?;
    v.as_i64().or_else(|| {
        // JSON numbers the app writes as floats are whole milliseconds.
        #[allow(clippy::cast_possible_truncation)]
        v.as_f64().map(|f| f as i64)
    })
}

fn parent_of(record: &Value) -> Option<String> {
    if record.get("lineageDetached") == Some(&Value::Bool(true)) {
        return None;
    }
    let spawned = record.get("spawnedFrom").and_then(|s| s.get("sessionId"));
    let dispatch = record.get("dispatchParentId");
    [spawned, dispatch]
        .into_iter()
        .flatten()
        .find_map(Value::as_str)
        .map(str::to_owned)
}

fn scan_scope(catalog: &dyn Catalog, scope: Scope) -> Result<ScopeScan, String> {
    let indexed: Vec<String> = catalog.archived(&scope)?;
    let mut chats = BTreeMap::new();
    for id in catalog.session_ids(&scope)? {
        let record = catalog.read_session(&scope, &id)?;
        if !is_syncable(&record, &id) {
            continue;
        }
        let archived =
            record.get("isArchived") == Some(&Value::Bool(true)) || indexed.contains(&id);
        let chat = Chat {
            title: record
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            archived,
            focused: millis(&record, "lastFocusedAt")
                .or_else(|| millis(&record, "lastActivityAt"))
                .unwrap_or(0),
            parent: parent_of(&record),
            record,
        };
        chats.insert(id, chat);
    }
    Ok(ScopeScan { scope, chats })
}

/// Reads the chats of every scope and the sidebar keys.
///
/// # Errors
/// When a record or the sidebar storage cannot be read.
pub fn scan(env: &Env) -> Result<Scan, String> {
    let mut scopes = env.catalog.scopes()?;
    scopes.sort_by_key(Scope::name);
    let scopes = scopes
        .into_iter()
        .map(|s| scan_scope(env.catalog, s))
        .collect::<Result<Vec<_>, _>>()?;
    let keys = SidebarKeys::from_value(&env.sidebar.read()?)?;
    Ok(Scan {
        at: env.clock.now_ms(),
        scopes,
        keys,
    })
}

fn lineage_only(before: &Value, after: &Value) -> bool {
    let strip = |v: &Value| {
        let mut v = v.clone();
        if let Some(map) = v.as_object_mut() {
            for key in LINEAGE_KEYS {
                map.remove(key);
            }
        }
        v
    };
    before != after && strip(before) == strip(after)
}

fn kind_of(scan: &Scan, change: &Change) -> StepKind {
    match change.kind {
        ChangeKind::CopySession => StepKind::CopySession,
        ChangeKind::ArchiveIndex => StepKind::Archive,
        ChangeKind::UpdateSession => {
            let before = scan
                .scopes
                .iter()
                .find(|s| s.scope == change.scope)
                .and_then(|s| s.chats.get(&change.session_id));
            match before {
                Some(chat) if lineage_only(&chat.record, &change.data) => StepKind::LinkThread,
                _ => StepKind::UpdateSession,
            }
        }
    }
}

/// Which chats each scope holds once the chat changes are written, with their focus time.
fn presence_after(scan: &Scan, plan: &Plan) -> Presence {
    let mut presence = Presence::new();
    for s in &scan.scopes {
        let chats = s
            .chats
            .iter()
            .map(|(id, c)| (id.clone(), c.focused))
            .collect();
        presence.insert(s.scope.name(), chats);
    }
    for change in plan
        .changes
        .iter()
        .filter(|c| c.kind != ChangeKind::ArchiveIndex)
    {
        let focused = millis(&change.data, "lastFocusedAt")
            .or_else(|| millis(&change.data, "lastActivityAt"))
            .unwrap_or(0);
        presence
            .entry(change.scope.name())
            .or_default()
            .insert(change.session_id.clone(), focused);
    }
    presence
}

/// Plans a sync of what a scan holds.
///
/// # Errors
/// When the chat plan or the folder plan fails.
pub fn plan(env: &Env, scan: Scan) -> Result<Planned, String> {
    let catalog = build_plan(env.catalog, env.transcripts, env.rule)?;
    let kinds = catalog.changes.iter().map(|c| kind_of(&scan, c)).collect();
    let presence = presence_after(&scan, &catalog);
    let folders = plan_folders(&scan.keys, &presence, &mut fresh_folder_id)?;
    Ok(Planned {
        scan,
        catalog,
        kinds,
        folders,
    })
}

/// Scans and plans.
///
/// # Errors
/// See [`scan`] and [`plan`].
pub fn scan_and_plan(env: &Env) -> Result<Planned, String> {
    plan(env, scan(env)?)
}

/// The name an account goes by in a step: its key, or the scope name when no account owns it.
fn account_of(accounts: &[Account], scope: &Scope) -> String {
    let owns = |a: &&Account| a.desktop_account.as_deref() == Some(scope.account.as_str());
    let home = |a: &&Account| a.desktop_org.as_deref() == Some(scope.org.as_str());
    accounts
        .iter()
        .find(|a| owns(a) && home(a))
        .or_else(|| accounts.iter().find(owns))
        .map_or_else(|| scope.name(), |a| a.key.clone())
}

/// The run in progress: numbers the steps, tells the sink and the progress callback.
struct Run<'a> {
    env: &'a Env<'a>,
    sink: &'a dyn StepSink,
    on_progress: &'a mut dyn FnMut(&Progress),
    seq: u64,
    progress: Progress,
}

impl Run<'_> {
    fn phase(&mut self, phase: Phase) {
        self.progress.phase = phase;
        (self.on_progress)(&self.progress);
    }

    fn begin(
        &mut self,
        kind: StepKind,
        who: (&str, &str, &str),
        folder: &str,
        detail: &str,
    ) -> SyncStep {
        self.seq += 1;
        let (account, session_id, title) = who;
        let step = SyncStep {
            seq: self.seq,
            kind,
            account: account.to_owned(),
            session_id: session_id.to_owned(),
            title: title.to_owned(),
            folder: folder.to_owned(),
            status: StepState::Running,
            at: self.env.clock.now_ms(),
            detail: detail.to_owned(),
        };
        self.sink.put(&step);
        step
    }

    fn finish(&mut self, mut step: SyncStep, status: StepState, detail: Option<String>) {
        step.status = status;
        step.at = self.env.clock.now_ms();
        if let Some(d) = detail {
            step.detail = d;
        }
        self.sink.put(&step);
        self.progress.step += 1;
        (self.on_progress)(&self.progress);
    }
}

/// Runs the sync. A dry run returns after the plan. An apply refuses while Claude Desktop runs,
/// then backs up, writes the chat files and the sidebar storage and checks that a new plan is
/// empty. On an error the progress callback sees phase `failed`.
///
/// # Errors
/// With a message for the owner: Claude is running, a read or write failed, or the check found
/// changes left (the backup is then named in the message).
pub fn run(
    env: &Env,
    mode: Mode,
    sink: &dyn StepSink,
    on_progress: &mut dyn FnMut(&Progress),
) -> Result<Summary, String> {
    let mut run = Run {
        env,
        sink,
        on_progress,
        seq: 0,
        progress: Progress {
            phase: Phase::Scan,
            step: 0,
            total: 0,
            backup_dir: None,
        },
    };
    if mode == Mode::Apply {
        sink.clear();
    }
    (run.on_progress)(&run.progress);
    let result = run_inner(&mut run, mode);
    run.phase(if result.is_ok() {
        Phase::Done
    } else {
        Phase::Failed
    });
    result
}

fn run_inner(run: &mut Run, mode: Mode) -> Result<Summary, String> {
    let env = run.env;
    let scanned = scan(env)?;
    run.phase(Phase::Plan);
    let planned = plan(env, scanned)?;
    let drift = drift_of(&planned);
    let mut summary = Summary {
        mode,
        drift,
        conflicts: planned.catalog.conflicts.len(),
        missing_transcripts: planned.catalog.missing_transcripts.len(),
        verified: false,
        backup_dir: None,
    };
    if mode == Mode::DryRun {
        return Ok(summary);
    }
    if (env.running)() {
        return Err("Claude Desktop is running; close it and run the sync again".to_owned());
    }
    if planned.is_empty() {
        summary.verified = true;
        return Ok(summary);
    }
    run.progress.total = total_steps(&planned);
    summary.backup_dir = Some(back_up(run, &planned)?);
    run.phase(Phase::Apply);
    write_chats(run, &planned)?;
    write_folders(run, &planned)?;
    run.phase(Phase::Verify);
    verify(run, summary.backup_dir.as_deref().unwrap_or_default())?;
    summary.verified = true;
    Ok(summary)
}

fn total_steps(planned: &Planned) -> u64 {
    let folder = planned
        .folders
        .steps
        .iter()
        .filter(|s| !matches!(s, FolderStep::ReorderFolder { .. }))
        .count();
    let storage = usize::from(!planned.folders.steps.is_empty());
    // The backup and the verify step bracket the writes.
    u64::try_from(2 + planned.catalog.changes.len() + folder + storage).unwrap_or(u64::MAX)
}

fn back_up(run: &mut Run, planned: &Planned) -> Result<String, String> {
    run.phase(Phase::Backup);
    let step = run.begin(StepKind::Backup, ("", "", ""), "", "");
    let stamp = backup::utc_stamp(run.env.clock.now_ms());
    match backup::create(run.env.backups, run.env.app, &planned.catalog, &stamp) {
        Ok(dir) => {
            let shown = dir.display().to_string();
            run.progress.backup_dir = Some(shown.clone());
            run.finish(step, StepState::Done, Some(shown.clone()));
            Ok(shown)
        }
        Err(e) => {
            run.finish(step, StepState::Failed, Some(e.clone()));
            Err(format!("backup failed: {e}"))
        }
    }
}

fn write_chats(run: &mut Run, planned: &Planned) -> Result<(), String> {
    let env = run.env;
    let mut index = 0;
    let mut current: Option<SyncStep> = None;
    apply(
        env.catalog,
        &planned.catalog,
        |change, status| match status {
            StepStatus::Running => {
                let account = account_of(env.accounts, &change.scope);
                let who = (
                    account.as_str(),
                    change.session_id.as_str(),
                    change.title.as_str(),
                );
                current = Some(run.begin(planned.kinds[index], who, "", &change.file));
            }
            StepStatus::Done => {
                if let Some(step) = current.take() {
                    run.finish(step, StepState::Done, None);
                }
                index += 1;
            }
            StepStatus::Failed(e) => {
                if let Some(step) = current.take() {
                    run.finish(step, StepState::Failed, Some(e));
                }
            }
        },
    )
    .map(|_| ())
}

fn write_folders(run: &mut Run, planned: &Planned) -> Result<(), String> {
    if planned.folders.steps.is_empty() {
        return Ok(());
    }
    let env = run.env;
    let mut shown = Vec::new();
    let mut reordered = 0;
    for step in &planned.folders.steps {
        match step {
            FolderStep::CreateFolder { scope, name, .. } => {
                let account = account_of_name(env.accounts, scope);
                shown.push(run.begin(StepKind::CreateFolder, (&account, "", ""), name, ""));
            }
            FolderStep::AssignChat {
                scope,
                session_id,
                folder_name,
            } => {
                let account = account_of_name(env.accounts, scope);
                let title = title_in(&planned.scan, scope, session_id);
                shown.push(run.begin(
                    StepKind::AssignFolder,
                    (&account, session_id, &title),
                    folder_name,
                    "",
                ));
            }
            FolderStep::ReorderFolder { .. } => reordered += 1,
        }
    }
    let detail = format!("{reordered} folders reordered");
    let storage = run.begin(StepKind::WriteStorage, ("", "", ""), "", &detail);
    let written = env.sidebar.write(&planned.folders.keys.to_value());
    let (state, note) = match &written {
        Ok(()) => (StepState::Done, None),
        Err(e) => (StepState::Failed, Some(e.clone())),
    };
    for step in shown {
        run.finish(step, state, note.clone());
    }
    run.finish(storage, state, note);
    written.map_err(|e| format!("the sidebar storage was not written: {e}"))
}

/// The account of a scope given by name (`<account>/<org>`).
fn account_of_name(accounts: &[Account], name: &str) -> String {
    let (account, org) = name.split_once('/').unwrap_or((name, ""));
    let scope = Scope {
        account: account.to_owned(),
        org: org.to_owned(),
    };
    account_of(accounts, &scope)
}

fn title_in(scan: &Scan, scope: &str, session_id: &str) -> String {
    scan.scopes
        .iter()
        .find(|s| s.scope.name() == scope)
        .and_then(|s| s.chats.get(session_id))
        .map(|c| c.title.clone())
        .unwrap_or_default()
}

fn verify(run: &mut Run, backup_dir: &str) -> Result<(), String> {
    let step = run.begin(StepKind::Verify, ("", "", ""), "", "");
    let again = scan_and_plan(run.env);
    let left = match &again {
        Ok(p) => p.catalog.changes.len() + p.folders.steps.len(),
        Err(_) => 0,
    };
    let failure = match again {
        Err(e) => Some(e),
        Ok(_) if left > 0 => Some(format!("{left} changes are still planned")),
        Ok(_) => None,
    };
    match failure {
        None => {
            run.finish(step, StepState::Done, None);
            Ok(())
        }
        Some(e) => {
            run.finish(step, StepState::Failed, Some(e.clone()));
            Err(format!(
                "verify failed: {e}; undo with the backup {backup_dir}"
            ))
        }
    }
}
