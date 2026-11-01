//! Helpers for the tests: throwaway folders and repositories, links, fakes for the traits the service reads through.
//! Everything happens in temporary folders the tests create. Never a real worktree, repository or `~/.coord`.

use crate::batch::{BatchInput, BatchOutcome};
use crate::board::BoardFacts;
use crate::board::{Caller, Link, OwnerEntry};
use crate::clock::{Clock, ManualClock, SystemClock, iso_from_ms};
use crate::error::WtError;
use crate::exec::GitOutput;
use crate::fs::{Entry, Fs, RealFs, Stat};
use crate::gh::GhRunner;
use crate::git::{Git, GitExec, SystemExec};
use crate::measure::{Bounds, Leftover, measure_folder};
use crate::paths::{clean_path, join_path};
use crate::ports::{
    ActivityLine, LiveRun, Notifier, ProcInfo, ProcessLister, WakeEvent, WorktreeStore,
};
use crate::request::{RemoveInput, RemoveOutcome};
use crate::rule::Rule;
use crate::service::{Deps, Options, Service};
use crate::sync::locked;
use argus_core::StorageError;
use argus_core::remote::{Mode, ModeSource};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

/// The folder of a temporary directory, as a clean path with its long name (git prints long names).
pub fn root_of(dir: &TempDir) -> String {
    let real = fs::canonicalize(dir.path()).unwrap_or_else(|_| dir.path().to_path_buf());
    let text = real.to_string_lossy().into_owned();
    clean_path(text.strip_prefix(r"\\?\").unwrap_or(&text))
}

/// Write a file, making its folders.
pub fn put(root: &str, rel: &str, content: &str) -> String {
    let path = join_path(root, rel);
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

/// Write a file, making its folders, and say nothing.
pub fn write(root: &str, rel: &str, content: &str) {
    put(root, rel, content);
}

/// Does the path exist as itself, link or not (a link is not followed)?
pub fn exists_itself(path: &str) -> bool {
    fs::symlink_metadata(path).is_ok()
}

/// A symlink (a junction on Windows) at `link` pointing to `target`.
pub fn make_link(link: &str, target: &str) {
    argus_core::fixture::links::junction(Path::new(link), Path::new(target));
}

pub fn set_mtime(path: &Path, when: SystemTime) {
    let mut options = fs::OpenOptions::new();
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.write(true).custom_flags(0x0200_0000);
    }
    #[cfg(not(windows))]
    options.read(true);
    if let Ok(file) = options.open(path) {
        let _ = file.set_modified(when);
    }
}

/// A number of days (possibly a fraction) as milliseconds.
#[allow(clippy::cast_possible_truncation)]
pub fn days_ms(days: f64) -> i64 {
    (days * 86_400_000.0) as i64
}

/// `days` days ago, as a file time.
pub fn days_ago(days: f64) -> SystemTime {
    SystemTime::now() - Duration::from_millis(days_ms(days).unsigned_abs())
}

/// Every entry of a folder (and the folder, last) gets the same modification time `days` days ago. A link is never followed.
pub fn backdate(dir: &str, days: f64) {
    let when = days_ago(days);
    walk_times(Path::new(dir), when);
}

fn walk_times(dir: &Path, when: SystemTime) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let kind = entry.file_type().unwrap();
        if kind.is_symlink() || crate::fs::meta_is_link(&entry.metadata().unwrap()) {
            let time = filetime::FileTime::from_system_time(when);
            filetime::set_symlink_file_times(entry.path(), time, time).unwrap();
            continue;
        }
        if kind.is_dir() {
            walk_times(&entry.path(), when);
        }
        set_mtime(&entry.path(), when);
    }
    set_mtime(dir, when);
}

/// Run git in a folder; panics with git's message when it fails. Authors are fixed; `env` adds dates.
pub fn git_in(cwd: &str, args: &[&str], env: &[(&str, &str)]) -> String {
    let mut command = Command::new("git");
    command.args(args).current_dir(cwd);
    for (k, v) in [
        ("GIT_AUTHOR_NAME", "Test"),
        ("GIT_AUTHOR_EMAIL", "test@example.test"),
        ("GIT_COMMITTER_NAME", "Test"),
        ("GIT_COMMITTER_EMAIL", "test@example.test"),
    ] {
        command.env(k, v);
    }
    for (k, v) in env {
        command.env(k, v);
    }
    let out = command.output().unwrap();
    assert!(
        out.status.success(),
        "git {} in {cwd}: {}",
        args.join(" "),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Commit a file, dated `days_ago` days back.
pub fn commit(cwd: &str, name: &str, text: &str, days: f64) {
    put(cwd, name, text);
    git_in(cwd, &["add", name], &[]);
    let when = iso_from_ms(SystemClock.now_ms() - days_ms(days));
    git_in(
        cwd,
        &["commit", "-q", "-m", &format!("add {name}")],
        &[("GIT_AUTHOR_DATE", &when), ("GIT_COMMITTER_DATE", &when)],
    );
}

/// The index file of a work tree (a linked one keeps its git dir under the main repository).
pub fn index_of(dir: &str) -> String {
    let dot = join_path(dir, ".git");
    let text = fs::read_to_string(&dot).unwrap_or_default();
    let gitdir = text
        .lines()
        .find_map(|l| l.strip_prefix("gitdir: "))
        .map(str::trim)
        .map_or_else(|| dot.clone(), clean_path);
    join_path(&gitdir, "index")
}

pub fn backdate_index(dir: &str, days: f64) {
    set_mtime(Path::new(&index_of(dir)), days_ago(days));
}

/// The branches of a repository.
pub fn branches(repo: &str) -> Vec<String> {
    git_in(repo, &["branch", "--format=%(refname:short)"], &[])
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

/// The names of the worktrees git lists for a repository.
pub fn listed(repo: &str) -> Vec<String> {
    git_in(repo, &["worktree", "list", "--porcelain"], &[])
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .map(|p| p.rsplit('/').next().unwrap_or(p).to_string())
        .collect()
}

/// A git that says every folder is not a repository.
pub struct NotRepo;

impl GitExec for NotRepo {
    fn exec(&self, _cwd: &str, _args: &[String], _timeout: Duration) -> GitOutput {
        GitOutput::failure(
            128,
            "fatal: not a git repository (or any of the parent directories): .git\n",
        )
    }
}

/// The real git, with every call recorded.
#[derive(Default)]
pub struct Recording {
    real: SystemExec,
    pub calls: Mutex<Vec<Vec<String>>>,
}

impl GitExec for Recording {
    fn exec(&self, cwd: &str, args: &[String], timeout: Duration) -> GitOutput {
        locked(&self.calls).push(args.to_vec());
        self.real.exec(cwd, args, timeout)
    }
}

/// A measure that sees nothing.
pub fn no_measure(_path: &str) -> Leftover {
    Leftover::default()
}

/// The scan's own measuring, on the real file system.
pub fn real_measure(path: &str) -> Leftover {
    measure_folder(&RealFs, path, Bounds::SCAN, &|| SystemClock.now_ms())
}

/// Every file under a folder with its size and modification time, to see that a scan changed nothing.
pub fn snapshot(dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    walk_snapshot(Path::new(dir), &mut out);
    out.sort();
    out
}

fn walk_snapshot(dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let meta = entry.metadata().unwrap();
        if meta.is_dir() {
            walk_snapshot(&entry.path(), out);
        } else {
            out.push(format!(
                "{}|{}|{:?}",
                entry.path().display(),
                meta.len(),
                meta.modified().ok()
            ));
        }
    }
}

/// An entry the owner posted from the page: a note that can carry the owner's say.
pub fn owner_entry(id: &str, text: &str, created_ms: i64) -> OwnerEntry {
    OwnerEntry {
        id: id.into(),
        author: "human".into(),
        via: Some("page".into()),
        kind: "followup".into(),
        created_ms: Some(created_ms),
        text: text.into(),
        ..OwnerEntry::default()
    }
}

/// [`owner_entry`] posted now.
pub fn owner_now(id: &str, text: &str) -> OwnerEntry {
    owner_entry(id, text, SystemClock.now_ms())
}

/// A throwaway repository with worktrees in every state the scanner tells apart.
///
/// - `fresh`: a commit made now (active)
/// - `old-unmerged`: a commit 10 days old, not merged, index 10 days old (stale)
/// - `old-merged`: at the tip of main (20 days old), index 15 days old, clean (merged)
/// - `dirty-old`: like old-merged, with an untracked file (dirty-stale)
/// - `gone`: added, then its folder deleted (missing)
/// - `plain`: a folder that is not a git work tree (a leftover)
pub struct Fixture {
    _dir: TempDir,
    pub root: String,
    pub repo: String,
    pub wts: String,
}

impl Fixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = root_of(&dir);
        let (repo, wts) = (join_path(&root, "proj"), join_path(&root, "proj-worktrees"));
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir_all(&wts).unwrap();
        git_in(&repo, &["init", "-q", "-b", "main"], &[]);
        commit(&repo, "a.txt", "one\n", 20.0);
        let fx = Self {
            _dir: dir,
            root,
            repo,
            wts,
        };
        fx.populate();
        fx
    }

    fn populate(&self) {
        let fresh = self.add("fresh");
        commit(&fresh, "fresh.txt", "new work\n", 0.0);
        let unmerged = self.add("old-unmerged");
        commit(&unmerged, "old.txt", "older work\n", 10.0);
        backdate_index(&unmerged, 10.0);
        backdate_index(&self.add("old-merged"), 15.0);
        let dirty = self.add("dirty-old");
        put(&dirty, "scratch.txt", "not committed\n");
        backdate_index(&dirty, 15.0);
        let gone = self.add("gone");
        fs::remove_dir_all(&gone).unwrap();
        put(&self.wts, "plain/readme.txt", "not a repository\n");
    }

    /// A linked worktree on a new branch of the same name.
    pub fn add(&self, name: &str) -> String {
        let path = self.path(name);
        git_in(
            &self.repo,
            &["worktree", "add", "-q", "-b", name, &path],
            &[],
        );
        path
    }

    pub fn path(&self, name: &str) -> String {
        join_path(&self.wts, name)
    }

    pub fn git(&self, args: &[&str]) -> String {
        git_in(&self.repo, args, &[])
    }
}

/// What a store keeps for the tests: one project, `alpha`, unless a test adds more that share its board.
pub struct MemStore {
    pub board: Mutex<BoardFacts>,
    pub projects: Mutex<Vec<String>>,
    pub kv: Mutex<HashMap<String, String>>,
    pub dropped: Mutex<Vec<(Vec<String>, Vec<String>)>>,
    /// The notes posted to inboxes: thread and text.
    pub inbox: Mutex<Vec<(String, String)>>,
    /// The runs queued or running, or why they could not be read.
    pub runs: Mutex<Result<Vec<Link>, String>>,
}

impl MemStore {
    pub fn new(board: BoardFacts) -> Self {
        Self {
            board: Mutex::new(board),
            projects: Mutex::new(vec!["alpha".to_string()]),
            kv: Mutex::new(HashMap::new()),
            dropped: Mutex::new(Vec::new()),
            inbox: Mutex::new(Vec::new()),
            runs: Mutex::new(Ok(Vec::new())),
        }
    }
}

impl WorktreeStore for MemStore {
    fn board(&self, project: &str) -> Option<BoardFacts> {
        locked(&self.projects)
            .iter()
            .any(|p| p == project)
            .then(|| locked(&self.board).clone())
    }

    fn projects(&self) -> Vec<String> {
        locked(&self.projects).clone()
    }

    fn kv_get(&self, key: &str) -> Option<String> {
        locked(&self.kv).get(key).cloned()
    }

    fn kv_set(&self, key: &str, value: &str) -> Result<(), StorageError> {
        locked(&self.kv).insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn drop_kept(
        &self,
        _project: &str,
        keep_ids: &[String],
        answer_ids: &[String],
    ) -> Result<(), StorageError> {
        let mut board = locked(&self.board);
        for id in keep_ids {
            board.keeps.remove(id);
        }
        for id in answer_ids {
            board.answers.remove(id);
        }
        locked(&self.dropped).push((keep_ids.to_vec(), answer_ids.to_vec()));
        Ok(())
    }

    fn post_inbox(&self, _project: &str, thread: &str, text: &str) -> Result<bool, StorageError> {
        let known = locked(&self.board).threads.contains_key(thread);
        if known {
            locked(&self.inbox).push((thread.to_string(), text.to_string()));
        }
        Ok(known)
    }

    fn live_runs(&self) -> Result<Vec<Link>, String> {
        locked(&self.runs).clone()
    }
}

/// Records the activity lines and the wakes.
#[derive(Default)]
pub struct Recorder {
    pub lines: Mutex<Vec<ActivityLine>>,
    pub wakes: Mutex<Vec<WakeEvent>>,
    pub refuse_wakes: AtomicBool,
    /// The failures said for the server's log.
    pub problems: Mutex<Vec<String>>,
}

impl Notifier for Recorder {
    fn activity(&self, line: ActivityLine) {
        locked(&self.lines).push(line);
    }

    fn wake(&self, events: &[WakeEvent]) -> Result<(), String> {
        if self.refuse_wakes.load(Ordering::SeqCst) {
            return Err("queue down".to_string());
        }
        locked(&self.wakes).extend(events.iter().cloned());
        Ok(())
    }

    fn problem(&self, text: &str) {
        locked(&self.problems).push(text.to_string());
    }
}

impl Recorder {
    /// The activity lines of kind `worktree-remove` whose text contains this.
    pub fn removals(&self, needle: &str) -> Vec<ActivityLine> {
        locked(&self.lines)
            .iter()
            .filter(|l| l.kind == "worktree-remove" && l.text.contains(needle))
            .cloned()
            .collect()
    }

    pub fn wake_count(&self) -> usize {
        locked(&self.wakes).len()
    }
}

/// The processes of the machine and Argus's live runs, as the test says.
#[derive(Default)]
pub struct FakeProcs(pub Mutex<Vec<ProcInfo>>, pub Mutex<Vec<LiveRun>>);

impl ProcessLister for FakeProcs {
    fn list(&self) -> Result<Vec<ProcInfo>, String> {
        Ok(locked(&self.0).clone())
    }

    fn runs(&self) -> Vec<LiveRun> {
        locked(&self.1).clone()
    }
}

/// `gh`, as the test says: one fixed answer, and the calls it saw.
pub struct FakeGh {
    pub answer: Mutex<GitOutput>,
    pub calls: Mutex<Vec<Vec<String>>>,
}

impl FakeGh {
    pub fn new(answer: GitOutput) -> Self {
        Self {
            answer: Mutex::new(answer),
            calls: Mutex::new(Vec::new()),
        }
    }
}

impl GhRunner for FakeGh {
    fn run(&self, _cwd: &str, args: &[String]) -> GitOutput {
        locked(&self.calls).push(args.to_vec());
        locked(&self.answer).clone()
    }
}

/// The switch, as the test says: remote until the test makes it local.
pub struct FakeMode(AtomicBool);

impl FakeMode {
    pub fn set_local(&self, local: bool) {
        self.0.store(local, Ordering::SeqCst);
    }
}

impl ModeSource for FakeMode {
    fn mode(&self) -> Mode {
        if self.0.load(Ordering::SeqCst) {
            Mode::Local
        } else {
            Mode::Remote
        }
    }
}

/// A door the test keeps shut until it opens it.
#[derive(Default)]
pub struct Gate {
    open: Mutex<bool>,
    changed: Condvar,
}

impl Gate {
    pub fn wait(&self) {
        let mut open = locked(&self.open);
        while !*open {
            open = crate::sync::waited(&self.changed, open);
        }
    }

    pub fn open(&self) {
        *locked(&self.open) = true;
        self.changed.notify_all();
    }
}

/// The real file system that fails on cue.
#[derive(Default)]
pub struct HookFs {
    pub rename_error: Mutex<Option<io::ErrorKind>>,
    pub fail_remove_containing: Mutex<Option<String>>,
    pub fail_unlink: Mutex<Option<io::ErrorKind>>,
    /// Listing a folder whose path contains this text fails like a folder without the right to read it.
    pub unreadable_containing: Mutex<Option<String>>,
    pub gate: Mutex<Option<Arc<Gate>>>,
    /// The gate holds only the paths that contain this text (every path when unset).
    pub gate_on: Mutex<Option<String>>,
}

impl HookFs {
    fn removal_error(&self, path: &str) -> Option<io::Error> {
        let needle = locked(&self.fail_remove_containing).clone()?;
        path.contains(&needle)
            .then(|| io::Error::from(io::ErrorKind::ResourceBusy))
    }

    fn wait_at_gate(&self, path: &str) {
        let held = locked(&self.gate_on)
            .as_ref()
            .is_none_or(|needle| path.contains(needle));
        let gate = locked(&self.gate).clone().filter(|_| held);
        if let Some(gate) = gate {
            gate.wait();
        }
    }
}

impl Fs for HookFs {
    fn exists(&self, path: &str) -> bool {
        RealFs.exists(path)
    }

    fn is_dir(&self, path: &str) -> bool {
        RealFs.is_dir(path)
    }

    fn dirs(&self, path: &str) -> io::Result<Vec<String>> {
        RealFs.dirs(path)
    }

    fn mtime_ms(&self, path: &str) -> Option<i64> {
        RealFs.mtime_ms(path)
    }

    fn lstat(&self, path: &str) -> io::Result<Stat> {
        RealFs.lstat(path)
    }

    fn read_dir(&self, path: &str) -> io::Result<Vec<Entry>> {
        let needle = locked(&self.unreadable_containing).clone();
        if needle.is_some_and(|n| path.contains(&n)) {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
        RealFs.read_dir(path)
    }

    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        match *locked(&self.rename_error) {
            Some(kind) => Err(io::Error::from(kind)),
            None => RealFs.rename(from, to),
        }
    }

    fn remove_file(&self, path: &str) -> io::Result<()> {
        self.wait_at_gate(path);
        if let Some(kind) = *locked(&self.fail_unlink) {
            return Err(io::Error::from(kind));
        }
        self.removal_error(path)
            .map_or_else(|| RealFs.remove_file(path), Err)
    }

    fn remove_dir(&self, path: &str) -> io::Result<()> {
        if let Some(kind) = *locked(&self.fail_unlink) {
            return Err(io::Error::from(kind));
        }
        self.removal_error(path)
            .map_or_else(|| RealFs.remove_dir(path), Err)
    }
}

/// How a `git worktree remove` ends in the test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveMode {
    /// The real git does it.
    Real,
    /// git deleted the tracked files and the `.git` file, then failed: `Directory not empty`.
    Stuck,
    /// git failed at once and changed nothing.
    Untouched,
    /// Like `Stuck`, but the answer is the door's timeout (the process was killed after deleting the files).
    Timeout,
    /// The timeout with nothing changed: the worktree is still registered with its `.git`.
    TimeoutUntouched,
    /// The files and `.git` went but the error is not a folder-deletion one.
    Other,
    /// git succeeded but the folder is still there with a file in it.
    Leaves,
}

pub type RemoveHook = Box<dyn Fn(&str) + Send>;
pub type FailureHook = Box<dyn Fn() + Send>;

/// The real git, except for `worktree remove`, which can fail like git does on a folder it cannot delete.
pub struct ScriptedExec {
    real: SystemExec,
    pub mode: Mutex<RemoveMode>,
    pub calls: Mutex<Vec<Vec<String>>>,
    pub on_remove: Mutex<Option<RemoveHook>>,
    pub after_failure: Mutex<Option<FailureHook>>,
    /// Every `git status` fails like a damaged index does.
    pub fail_status: Mutex<bool>,
}

/// The door's own answer to a process that ran past the removal timeout and was killed.
fn timed_out() -> GitOutput {
    GitOutput {
        timed_out: true,
        error: Some("git timed out after 900 s".to_string()),
        ..GitOutput::default()
    }
}

impl ScriptedExec {
    pub fn new(mode: RemoveMode) -> Self {
        Self {
            real: SystemExec::default(),
            mode: Mutex::new(mode),
            calls: Mutex::new(Vec::new()),
            on_remove: Mutex::new(None),
            after_failure: Mutex::new(None),
            fail_status: Mutex::new(false),
        }
    }

    fn simulate(
        &self,
        mode: RemoveMode,
        cwd: &str,
        args: &[String],
        timeout: Duration,
    ) -> GitOutput {
        let path = args.last().cloned().unwrap_or_default();
        if let Some(hook) = locked(&self.on_remove).as_ref() {
            hook(&path);
        }
        match mode {
            RemoveMode::Untouched => GitOutput::failure(
                128,
                &format!("error: failed to delete '{path}': Permission denied\n"),
            ),
            RemoveMode::TimeoutUntouched => timed_out(),
            RemoveMode::Leaves => {
                let out = self.real.exec(cwd, args, timeout);
                put(&path, "left-behind.txt", "git could not delete me\n");
                out
            }
            _ => {
                let _ = fs::remove_file(join_path(&path, "a.txt"));
                let _ = fs::remove_file(join_path(&path, ".git"));
                if let Some(hook) = locked(&self.after_failure).as_ref() {
                    hook();
                }
                if mode == RemoveMode::Timeout {
                    return timed_out();
                }
                let said = if mode == RemoveMode::Other {
                    "fatal: something else went wrong\n".to_string()
                } else {
                    format!("error: failed to delete '{path}': Directory not empty\n")
                };
                GitOutput::failure(128, &said)
            }
        }
    }
}

impl GitExec for ScriptedExec {
    fn exec(&self, cwd: &str, args: &[String], timeout: Duration) -> GitOutput {
        locked(&self.calls).push(args.to_vec());
        if *locked(&self.fail_status) && args.iter().any(|a| a == "status") {
            return GitOutput::failure(
                128,
                "fatal: unable to read the index
",
            );
        }
        let mode = *locked(&self.mode);
        let is_remove = args.first().map(String::as_str) == Some("worktree")
            && args.get(1).map(String::as_str) == Some("remove");
        if is_remove && mode != RemoveMode::Real {
            return self.simulate(mode, cwd, args, timeout);
        }
        self.real.exec(cwd, args, timeout)
    }
}

/// A service over a fixture, with every dependency a fake the test can steer.
pub struct Harness {
    pub svc: Service,
    pub store: Arc<MemStore>,
    pub notes: Arc<Recorder>,
    pub procs: Arc<FakeProcs>,
    pub gh: Arc<FakeGh>,
    pub mode: Arc<FakeMode>,
    pub clock: Arc<ManualClock>,
    pub exec: Arc<ScriptedExec>,
    pub fs: Arc<HookFs>,
    pub backups: String,
}

/// The standard board: a lead, the rule's thread, another thread and an ended one.
pub fn board_with(rules: Vec<Rule>) -> BoardFacts {
    use crate::board::ThreadInfo;
    let mut board = BoardFacts {
        name: Some("Alpha".to_string()),
        rules,
        ..BoardFacts::default()
    };
    for (id, coordinator, live) in [
        ("lead", true, true),
        ("worker", false, true),
        ("other", false, true),
        ("ended", false, false),
    ] {
        board.threads.insert(
            id.to_string(),
            ThreadInfo {
                id: id.to_string(),
                coordinator,
                live,
            },
        );
    }
    board
}

impl Harness {
    /// A service with the given rules, over a root folder (the backups go under it), the real git and the real file system.
    pub fn new(root: &str, rules: Vec<Rule>, options: impl FnOnce(&mut Options)) -> Self {
        Self::with_mode(root, board_with(rules), RemoveMode::Real, options)
    }

    pub fn with_mode(
        root: &str,
        board: BoardFacts,
        mode: RemoveMode,
        options: impl FnOnce(&mut Options),
    ) -> Self {
        Self::on_store(Arc::new(MemStore::new(board)), root, mode, options)
    }

    /// A new service over a store another one used (a restart of the server).
    pub fn on_store(
        store: Arc<MemStore>,
        root: &str,
        mode: RemoveMode,
        options: impl FnOnce(&mut Options),
    ) -> Self {
        let (notes, procs) = (
            Arc::new(Recorder::default()),
            Arc::new(FakeProcs::default()),
        );
        let gh = Arc::new(FakeGh::new(GitOutput {
            error: Some("ENOENT".to_string()),
            ..GitOutput::default()
        }));
        let remote = Arc::new(FakeMode(AtomicBool::new(false)));
        let clock = Arc::new(ManualClock::at(SystemClock.now_ms()));
        let exec = Arc::new(ScriptedExec::new(mode));
        let hook_fs = Arc::new(HookFs::default());
        let backups = join_path(root, "worktree-backups");
        let mut opts = Options {
            backup_dir: backups.clone(),
            case_insensitive: cfg!(windows),
            drop_links_first: false,
            // The hygiene has its own specs (`specs/hygiene.rs`) that turn it on.
            release: false,
            ..Options::default()
        };
        options(&mut opts);
        let deps = Deps {
            store: store.clone(),
            notifier: notes.clone(),
            fs: hook_fs.clone(),
            git: Git::new(exec.clone()),
            gh: gh.clone(),
            remote: remote.clone(),
            procs: procs.clone(),
            clock: clock.clone(),
        };
        Self {
            svc: Service::new(deps, opts),
            store,
            notes,
            procs,
            gh,
            mode: remote,
            clock,
            exec,
            fs: hook_fs,
            backups,
        }
    }

    pub fn board(&self) -> std::sync::MutexGuard<'_, BoardFacts> {
        locked(&self.store.board)
    }

    /// The rows by folder name.
    pub fn docs(&self) -> HashMap<String, crate::doc::WorktreeDoc> {
        self.svc
            .list("alpha")
            .into_iter()
            .map(|d| (d.name.clone(), d))
            .collect()
    }

    pub fn meta(&self) -> crate::doc::Meta {
        self.svc.derive("alpha").unwrap().meta
    }

    pub fn id_of(&self, path: &str) -> String {
        crate::paths::worktree_id(path, self.svc.opts.case_insensitive)
    }

    pub fn scan(&self) {
        self.svc.scan_now("alpha");
    }

    /// Wait for the jobs.
    pub fn idle(&self) {
        self.svc.jobs_idle();
    }

    /// The input for a removal by the rule's thread, `worker`.
    pub fn input(id: &str) -> RemoveInput {
        RemoveInput::new("alpha", id, Caller::Thread("worker".to_string()))
    }

    pub fn rm(&self, id: &str) -> crate::error::Result<RemoveOutcome> {
        self.svc.remove(&Self::input(id))
    }

    /// A removal with changes to its input.
    pub fn rm_with(
        &self,
        id: &str,
        change: impl FnOnce(&mut RemoveInput),
    ) -> crate::error::Result<RemoveOutcome> {
        let mut input = Self::input(id);
        change(&mut input);
        self.svc.remove(&input)
    }

    /// A bulk deletion of leftovers, asked by `caller`, with changes to its input.
    pub fn batch(
        &self,
        caller: Caller,
        change: impl FnOnce(&mut BatchInput),
    ) -> crate::error::Result<BatchOutcome> {
        let mut input = BatchInput::new("alpha", caller);
        change(&mut input);
        self.svc.remove_leftovers(&input)
    }

    /// The refusal of a removal; fails the test when it was not refused.
    pub fn refused(&self, id: &str, change: impl FnOnce(&mut RemoveInput)) -> WtError {
        match self.rm_with(id, change) {
            Ok(out) => panic!("the removal of {id} should have been refused: {}", out.text),
            Err(err) => err,
        }
    }

    /// The facts of the last scan, by folder name.
    pub fn fact(&self, name: &str) -> crate::fact::Fact {
        self.svc
            .facts_of("alpha")
            .into_iter()
            .find(|f| f.path.ends_with(&format!("/{name}")))
            .unwrap_or_else(|| panic!("no fact for {name}"))
    }

    /// The deletion job whose folder ends with this name.
    pub fn job_ending(&self, name: &str) -> crate::jobs::JobDoc {
        let suffix = format!("/{name}");
        self.svc
            .jobs("alpha")
            .into_iter()
            .find(|j| j.path.ends_with(&suffix))
            .unwrap_or_else(|| panic!("no job for {name}"))
    }

    pub fn clock_now(&self) -> i64 {
        self.clock.now_ms()
    }

    /// The text of the wake sent to `thread`.
    pub fn wake_text(&self, thread: &str) -> String {
        let wakes = locked(&self.notes.wakes);
        wakes
            .iter()
            .find(|w| w.thread == thread)
            .unwrap()
            .text
            .clone()
    }

    /// The text of the lead's `worktree-stale` wake.
    pub fn lead_stale_text(&self) -> String {
        let wakes = locked(&self.notes.wakes);
        let lead = wakes.iter().find(|w| w.thread == "lead").unwrap();
        assert_eq!(lead.reason, "worktree-stale");
        lead.text.clone()
    }

    pub fn wake_texts(&self) -> Vec<String> {
        locked(&self.notes.wakes)
            .iter()
            .map(|w| w.text.clone())
            .collect()
    }
}

/// A fixture prepared by `prep`, a service over it with the fixture's rule, and one scan.
pub fn started(prep: impl FnOnce(&Fixture)) -> (Fixture, Harness) {
    let fx = Fixture::new();
    prep(&fx);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    (fx, h)
}

/// A record git lists for a worktree OUTSIDE every rule (an unplugged drive): its folder is gone, so a prune would forget it.
pub fn unplugged(commits: bool) -> impl FnOnce(&Fixture) {
    move |fx| {
        let path = format!("{}/unplugged/drive-x", fx.root);
        fx.git(&["worktree", "add", "-q", "--detach", &path, "main"]);
        if commits {
            commit(
                &path,
                "u.txt",
                "work on a detached HEAD
",
                3.0,
            );
        }
        fs::remove_dir_all(&path).unwrap();
    }
}

/// The names git keeps worktree records under, sorted.
pub fn records(fx: &Fixture) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(format!("{}/.git/worktrees", fx.repo))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The id of a worktree folder of the fixture.
pub fn id(h: &Harness, fx: &Fixture, name: &str) -> String {
    h.id_of(&fx.path(name))
}

/// A fixture prepared by `prep` whose merged worktree `old-merged` holds a nested repository at `nested`, started, and the
/// refusal of removing it: forbidden, naming the nested repository. Returns the fixture, harness, the folder's id and the
/// refusal.
pub fn refused_for_nested(
    prep: impl FnOnce(&Fixture),
    nested: &str,
) -> (Fixture, Harness, String, WtError) {
    let (fx, h) = started(prep);
    assert_eq!(
        h.docs()["old-merged"].dirty,
        0,
        "git sees nothing to commit"
    );
    let target = id(&h, &fx, "old-merged");
    let err = h.refused(&target, |_| {});
    assert_eq!(err.code, crate::error::ErrorCode::Forbidden);
    let named = format!("a nested git repository ({nested})");
    assert!(err.message.contains(&named), "{}", err.message);
    (fx, h, target, err)
}

/// A project with a GitHub remote whose pull request lookup answers `answer`, and the removal of the unmerged worktree
/// with its branch. The branch must survive: the answer proves no merge. Returns the fixture.
pub fn pull_request_answer_keeps_branch(answer: &str) -> Fixture {
    let fx = Fixture::new();
    let remote = [
        "remote",
        "add",
        "origin",
        "https://github.com/acme/proj.git",
    ];
    git_in(&fx.repo, &remote, &[]);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    *h.gh.answer.lock().unwrap() = GitOutput {
        code: Some(0),
        stdout: answer.into(),
        ..GitOutput::default()
    };
    let out = h
        .rm_with(&id(&h, &fx, "old-unmerged"), |i| i.branch = true)
        .unwrap();
    assert!(!out.deleted_branch, "{}", out.text);
    assert!(branches(&fx.repo).contains(&"old-unmerged".to_string()));
    fx
}

/// The rule of the fixture's folder, for the rule's thread `worker`.
pub fn rule_for(fx: &Fixture) -> Rule {
    Rule::folder("W-1", &fx.wts, "worker")
}
