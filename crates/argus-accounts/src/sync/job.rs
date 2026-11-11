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

pub(super) fn millis(record: &Value, key: &str) -> Option<i64> {
    let v = record.get(key)?;
    v.as_i64().or_else(|| {
        // JSON numbers the app writes as floats are whole milliseconds.
        #[allow(clippy::cast_possible_truncation)]
        v.as_f64().map(|f| f as i64)
    })
}

pub(super) fn parent_of(record: &Value) -> Option<String> {
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

pub(super) fn kind_of(scan: &Scan, change: &Change) -> StepKind {
    match change.kind {
        ChangeKind::CopySession => StepKind::CopySession,
        ChangeKind::ArchiveIndex => StepKind::Archive,
        ChangeKind::DeleteSession => StepKind::DeleteChat,
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
pub(super) fn account_of(accounts: &[Account], scope: &Scope) -> String {
    let owns = |a: &&Account| a.desktop_account.as_deref() == Some(scope.account.as_str());
    let home = |a: &&Account| a.desktop_org.as_deref() == Some(scope.org.as_str());
    accounts
        .iter()
        .find(|a| owns(a) && home(a))
        .or_else(|| accounts.iter().find(owns))
        .map_or_else(|| scope.name(), |a| a.key.clone())
}

/// The run in progress: numbers the steps, tells the sink and the progress callback.
pub(super) struct Run<'a> {
    pub(super) env: &'a Env<'a>,
    sink: &'a dyn StepSink,
    on_progress: &'a mut dyn FnMut(&Progress),
    seq: u64,
    pub(super) progress: Progress,
}

impl<'a> Run<'a> {
    /// A run that has not reported anything yet.
    pub(super) fn new(
        env: &'a Env<'a>,
        sink: &'a dyn StepSink,
        on_progress: &'a mut dyn FnMut(&Progress),
    ) -> Self {
        Run {
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
        }
    }

    /// Reports the start; an apply first forgets the steps of the last run.
    pub(super) fn start(&mut self, mode: Mode) {
        if mode == Mode::Apply {
            self.sink.clear();
        }
        (self.on_progress)(&self.progress);
    }

    pub(super) fn phase(&mut self, phase: Phase) {
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
    let mut run = Run::new(env, sink, on_progress);
    run.start(mode);
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
    let backup_dir = write_planned(run, &planned, &|env| {
        let again = scan_and_plan(env)?;
        let left = again.catalog.changes.len() + again.folders.steps.len();
        if left > 0 {
            return Err(format!("{left} changes are still planned"));
        }
        Ok(())
    })?;
    summary.backup_dir = Some(backup_dir);
    summary.verified = true;
    Ok(summary)
}

/// Backs up, writes the chat files and the sidebar storage of a plan, then runs `check` as the
/// verify step. Returns the backup folder.
pub(super) fn write_planned(
    run: &mut Run,
    planned: &Planned,
    check: &dyn Fn(&Env) -> Result<(), String>,
) -> Result<String, String> {
    let env = run.env;
    run.progress.total = total_steps(planned);
    let backup_dir = back_up(run, planned)?;
    run.phase(Phase::Apply);
    let written = write_chats(run, planned).and_then(|()| write_folders(run, planned));
    // The undo needs to know what the run wrote, also when it stopped half way.
    let recorded = backup::record_applied(Path::new(&backup_dir), env.app, &planned.catalog);
    written?;
    recorded.map_err(|e| format!("the backup {backup_dir} was not updated: {e}"))?;
    run.phase(Phase::Verify);
    verify(run, &backup_dir, check)?;
    Ok(backup_dir)
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

/// How many chat files are written between two checks that Claude is still closed.
const BATCH: usize = 20;

const STARTED: &str =
    "Claude Desktop started during the sync; the run stopped, close it and run the sync again";

/// Calls `each(offset, batch)` for consecutive batches of `items`, checking before every batch
/// that Claude Desktop is still closed.
fn in_batches<'a, T>(
    items: &'a [T],
    running: &dyn Fn() -> bool,
    mut each: impl FnMut(usize, &'a [T]) -> Result<(), String>,
) -> Result<(), String> {
    let mut offset = 0;
    for batch in items.chunks(BATCH) {
        if running() {
            return Err(STARTED.to_owned());
        }
        each(offset, batch)?;
        offset += batch.len();
    }
    Ok(())
}

fn write_chats(run: &mut Run, planned: &Planned) -> Result<(), String> {
    let env = run.env;
    in_batches(&planned.catalog.changes, env.running, |offset, changes| {
        let part = Plan {
            changes: changes.to_vec(),
            ..Plan::default()
        };
        write_batch(run, &part, &planned.kinds[offset..offset + changes.len()])
    })
}

fn write_batch(run: &mut Run, part: &Plan, kinds: &[StepKind]) -> Result<(), String> {
    let env = run.env;
    let mut index = 0;
    let mut current: Option<SyncStep> = None;
    apply(env.catalog, part, |change, status| match status {
        StepStatus::Running => {
            let account = account_of(env.accounts, &change.scope);
            let who = (
                account.as_str(),
                change.session_id.as_str(),
                change.title.as_str(),
            );
            current = Some(run.begin(kinds[index], who, "", &change.file));
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
    })
    .map(|_| ())
}

fn write_folders(run: &mut Run, planned: &Planned) -> Result<(), String> {
    if planned.folders.steps.is_empty() {
        return Ok(());
    }
    let env = run.env;
    if (env.running)() {
        return Err(STARTED.to_owned());
    }
    let mut shown = Vec::new();
    let mut reordered = 0;
    for step in &planned.folders.steps {
        match shown_as(planned, env.accounts, step) {
            Some(line) => {
                let who = (line.account.as_str(), line.session_id, line.title.as_str());
                shown.push(run.begin(line.kind, who, line.folder, &line.detail));
            }
            None => reordered += 1,
        }
    }
    let detail = format!("{reordered} folders reordered");
    let storage = run.begin(StepKind::WriteStorage, ("", "", ""), "", &detail);
    let written = env.sidebar.write(
        &planned.folders.keys.to_value(),
        &planned.scan.keys.to_value(),
    );
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

/// How a folder step shows in the step log.
struct Shown<'a> {
    kind: StepKind,
    account: String,
    session_id: &'a str,
    title: String,
    folder: &'a str,
    detail: String,
}

/// The log line of a folder step; none for a reorder, which only counts.
fn shown_as<'a>(
    planned: &Planned,
    accounts: &[Account],
    step: &'a FolderStep,
) -> Option<Shown<'a>> {
    let line = |kind, folder: &'a str| Shown {
        kind,
        account: account_of_name(accounts, step.scope()),
        session_id: "",
        title: String::new(),
        folder,
        detail: String::new(),
    };
    let chat = |kind, session_id: &'a str, folder: &'a str| Shown {
        session_id,
        title: title_in(&planned.scan, step.scope(), session_id),
        ..line(kind, folder)
    };
    Some(match step {
        FolderStep::CreateFolder { name, .. } => line(StepKind::CreateFolder, name),
        FolderStep::AssignChat {
            session_id,
            folder_name,
            ..
        } => chat(StepKind::AssignFolder, session_id, folder_name),
        FolderStep::MoveChat {
            session_id,
            folder_name,
            ..
        } => chat(
            StepKind::MoveChat,
            session_id,
            folder_name.as_deref().unwrap_or_default(),
        ),
        FolderStep::RenameFolder { name, new_name, .. } => Shown {
            detail: format!("renamed to {new_name}"),
            ..line(StepKind::RenameFolder, name)
        },
        FolderStep::DeleteFolder { name, .. } => line(StepKind::DeleteFolder, name),
        FolderStep::ReorderFolder { .. } => return None,
    })
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

fn verify(
    run: &mut Run,
    backup_dir: &str,
    check: &dyn Fn(&Env) -> Result<(), String>,
) -> Result<(), String> {
    let step = run.begin(StepKind::Verify, ("", "", ""), "", "");
    match check(run.env).err() {
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

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn writes_go_in_batches_and_stop_when_claude_starts() {
        let items: Vec<u32> = (0..50).collect();
        let checks = Cell::new(0);
        let running = || {
            checks.set(checks.get() + 1);
            checks.get() > 2
        };
        let mut seen = Vec::new();
        let result = in_batches(&items, &running, |offset, batch| {
            seen.push((offset, batch.len()));
            Ok(())
        });
        assert_eq!(result, Err(STARTED.to_owned()));
        assert_eq!(seen, vec![(0, BATCH), (BATCH, BATCH)]);
    }

    #[test]
    fn every_item_is_written_while_claude_stays_closed() {
        let items: Vec<u32> = (0..45).collect();
        let mut written = 0;
        let result = in_batches(&items, &|| false, |_, batch| {
            written += batch.len();
            Ok(())
        });
        assert!(result.is_ok());
        assert_eq!(written, 45);
    }
}
