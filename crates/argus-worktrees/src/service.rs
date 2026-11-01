//! The worktrees service: its settings, its inputs and the state it keeps between calls.
//!
//! Scanning never changes a worktree. A removal goes only through the guards in `remove`, `leftover_remove` and
//! `batch`. Scans and removals of a project run one at a time; deletion jobs of a project run one at a time on a
//! background worker. Another project never waits for them.

use crate::board::BoardFacts;
use crate::clock::Clock;
use crate::doc::{Derived, WorktreeDoc};
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::fs::Fs;
use crate::gh::GhRunner;
use crate::git::Git;
use crate::jobs::{Job, JobDoc};
use crate::measure::{Bounds, Leftover};
use crate::ports::{ActivityLine, Notifier, ProcInfo, ProcessLister, WorktreeStore};
use crate::probe::{Fingerprint, SAFETY_SCAN_MS};
use crate::queue::Queues;
use crate::requests::Requests;
use crate::scan::RuleInfo;
use crate::sync::locked;
use crate::turns::{Asker, Turn, Turns};
use argus_core::remote::ModeSource;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

/// The settings of the service (`config.json` `worktrees` on the server's side).
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct Options {
    /// The safety scan: every worktree is read with git again this often, whatever the fingerprints say (`scanMs`,
    /// `SAFETY_SCAN_MS` by default). Between safety scans git runs only for a worktree whose fingerprint changed.
    pub scan_ms: i64,
    pub repeat_ms: i64,
    pub live_ms: i64,
    pub many_at: usize,
    pub stale_days: u32,
    /// A leftover untouched for this many days may be deleted by a thread (1; the page may confirm a younger one).
    pub leftover_age_days: f64,
    pub remeasure_ms: i64,
    pub budget_ms: i64,
    pub measure_bounds: Bounds,
    pub remove_bounds: Bounds,
    pub case_insensitive: bool,
    /// Remove every junction and symlink inside a worktree as a link before git touches it (git for Windows follows them).
    pub drop_links_first: bool,
    pub backup_dir: String,
    pub keep_backups: usize,
    pub keep_backup_days: i64,
    /// Run the removal requests of threads (a restart resumes them). Off in shadow mode and in tests that steer the worker.
    pub run_requests: bool,
    /// The worktree hygiene: flag the released worktrees (no running agent, nothing holds them, nothing is lost) and remove them after the grace. Off in
    /// shadow mode.
    pub release: bool,
    /// How long a released worktree stays after its thread was told (`worktrees.releaseGraceMs`, 1 hour).
    pub release_grace_ms: i64,
    /// A newly released worktree is told once nothing new was released for this long, so a burst makes one wake.
    pub release_debounce_ms: i64,
    /// How long one target of a request may take before the request gives up on it and goes on (30 minutes: more
    /// than the longest git removal, 15 minutes, and a scan before it).
    pub target_limit_ms: i64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            scan_ms: SAFETY_SCAN_MS,
            repeat_ms: 24 * 3_600_000,
            live_ms: 20_000,
            many_at: 12,
            stale_days: crate::rule::DEFAULT_STALE_DAYS,
            leftover_age_days: 1.0,
            remeasure_ms: 3_600_000,
            budget_ms: 40_000,
            measure_bounds: Bounds::SCAN,
            remove_bounds: Bounds::REMOVE,
            case_insensitive: cfg!(windows),
            drop_links_first: cfg!(windows),
            backup_dir: "worktree-backups".to_string(),
            keep_backups: 30,
            keep_backup_days: 30,
            run_requests: true,
            release: true,
            release_grace_ms: 3_600_000,
            release_debounce_ms: 30_000,
            target_limit_ms: 30 * 60_000,
        }
    }
}

/// Everything the service reads from and writes to.
pub struct Deps {
    pub store: Arc<dyn WorktreeStore>,
    pub notifier: Arc<dyn Notifier>,
    pub fs: Arc<dyn Fs>,
    pub git: Git,
    pub gh: Arc<dyn GhRunner>,
    /// The local or remote switch: in local mode `gh` is never asked.
    pub remote: Arc<dyn ModeSource>,
    pub procs: Arc<dyn ProcessLister>,
    pub clock: Arc<dyn Clock>,
}

/// What the last scan of a project found.
pub struct ScanState {
    pub at_ms: i64,
    pub took_ms: i64,
    pub rules: Vec<RuleInfo>,
    pub facts: Vec<Fact>,
    pub listed: HashSet<String>,
    pub error: Option<String>,
    /// When a scan last read every worktree with git (a scan that kept some facts leaves it as it was).
    pub full_ms: i64,
    /// How the rules sat on disk at this scan (`probe::shape_of`).
    pub shape: String,
    /// The fingerprint of each candidate folder when it was read, by path.
    pub prints: HashMap<String, Fingerprint>,
}

pub struct Measured {
    pub at: i64,
    pub mtime: Option<i64>,
    pub value: Leftover,
}

pub struct GhCached {
    pub tip: String,
    pub at: i64,
    pub merged: bool,
}

/// The mutable part of the service. Locked for short stretches only: never across git or file system work.
#[derive(Default)]
pub struct State {
    pub scans: HashMap<String, ScanState>,
    pub jobs: Vec<Job>,
    pub job_seq: u64,
    pub sigs: HashMap<String, String>,
    /// The agent records that ended (done, stopped, failed), by project, as the last look at the board saw them.
    pub ended_agents: HashMap<String, BTreeSet<String>>,
    pub measured: HashMap<String, Measured>,
    pub budget_spent_ms: i64,
    pub adopted: HashSet<String>,
    pub gh_cache: HashMap<String, GhCached>,
    pub proc_cache: Option<(i64, Vec<ProcInfo>)>,
    pub rescans: BTreeSet<String>,
    /// What each running removal does now, by worktree id.
    pub steps: HashMap<String, crate::progress::Step>,
    pub stopped: bool,
}

pub struct Inner {
    pub deps: Deps,
    pub opts: Options,
    pub state: Mutex<State>,
    pub exclusive: Turns,
    /// The deletion jobs, one worker per project.
    pub queue: Queues,
    pub requests: Requests,
    pub(crate) hygiene: crate::hygiene::Memories,
}

/// The service. Cheap to clone: every clone is the same service.
#[derive(Clone)]
pub struct Service(Arc<Inner>);

impl Deref for Service {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.0
    }
}

impl Service {
    pub fn new(deps: Deps, opts: Options) -> Self {
        let state = State {
            stopped: false,
            ..State::default()
        };
        Self(Arc::new(Inner {
            deps,
            opts,
            state: Mutex::new(state),
            exclusive: Turns::default(),
            queue: Queues::default(),
            requests: Requests::default(),
            hygiene: crate::hygiene::Memories::default(),
        }))
    }

    pub(crate) fn st(&self) -> MutexGuard<'_, State> {
        locked(&self.state)
    }

    /// Scans and removals of a project run one at a time, first come first served: hold this while doing either.
    /// `what` names the holder to the ones that wait; `key` is the worktree a removal is on.
    pub(crate) fn one_at_a_time(&self, project: &str, what: &str, key: Option<&str>) -> Turn {
        self.exclusive
            .take(project, &Asker { what, key }, &|| self.now())
    }

    pub(crate) fn now(&self) -> i64 {
        self.deps.clock.now_ms()
    }

    pub(crate) fn ci(&self) -> bool {
        self.opts.case_insensitive
    }

    pub fn leftover_age_days(&self) -> f64 {
        self.opts.leftover_age_days
    }

    pub(crate) fn board(&self, project: &str) -> Result<BoardFacts> {
        self.deps
            .store
            .board(project)
            .ok_or_else(|| WtError::new(ErrorCode::NotFound, format!("no project \"{project}\"")))
    }

    /// An activity line of kind `worktree-remove` for this project.
    pub(crate) fn note(
        &self,
        project: &str,
        level: &'static str,
        text: &str,
        thread: Option<&str>,
    ) {
        self.deps.notifier.activity(ActivityLine {
            kind: "worktree-remove",
            level,
            text: format!("{project}: {text}"),
            project: project.to_string(),
            thread: thread.map(ToString::to_string),
        });
    }

    /// Ask for a scan of the project. The server drains these and scans (a removal never waits for the scan).
    pub(crate) fn request_rescan(&self, project: &str) {
        let mut state = self.st();
        if !state.stopped {
            state.rescans.insert(project.to_string());
        }
    }

    /// The projects that asked for a new scan since the last call.
    pub fn take_rescans(&self) -> Vec<String> {
        std::mem::take(&mut self.st().rescans).into_iter().collect()
    }

    /// Stop: no scan runs, no rescan is asked for. The board listener and the timers are the server's to stop.
    pub fn stop(&self) {
        self.st().stopped = true;
        self.stop_requests();
    }

    /// Start again after a stop.
    pub fn start(&self) {
        self.st().stopped = false;
        self.start_requests();
    }

    pub(crate) fn stopped(&self) -> bool {
        self.st().stopped
    }

    /// The facts of the last scan of a project.
    pub(crate) fn facts_of(&self, project: &str) -> Vec<Fact> {
        self.st()
            .scans
            .get(project)
            .map(|s| s.facts.clone())
            .unwrap_or_default()
    }

    /// The fact with this id, when its rule exists and is valid.
    pub(crate) fn fact_of(&self, project: &str, id: &str, board: &BoardFacts) -> Option<Fact> {
        let state = self.st();
        let fact = state
            .scans
            .get(project)?
            .facts
            .iter()
            .find(|f| f.id == id)?;
        board
            .valid_rules()
            .iter()
            .any(|r| r.id == fact.rule)
            .then(|| fact.clone())
    }

    /// The worktree leaves the listing at once; the rescan after confirms it.
    pub(crate) fn forget(&self, project: &str, id: &str) {
        if let Some(scan) = self.st().scans.get_mut(project) {
            scan.facts.retain(|f| f.id != id);
        }
    }

    /// Every repository any project's scan knows.
    pub(crate) fn known_repos(&self) -> Vec<String> {
        let state = self.st();
        let mut repos: Vec<String> = state
            .scans
            .values()
            .flat_map(|s| s.facts.iter().filter_map(|f| f.repo.clone()))
            .collect();
        repos.sort();
        repos.dedup();
        repos
    }

    /// The worktrees git still lists whose folder is gone.
    pub(crate) fn gone_of(&self, project: &str) -> Vec<Fact> {
        self.facts_of(project)
            .into_iter()
            .filter(|f| !f.exists)
            .collect()
    }

    /// The rows and the summary of a project, judged now from the last scan.
    pub fn derive(&self, project: &str) -> Option<Derived> {
        let board = self.deps.store.board(project)?;
        Some(self.derive_with(project, &board))
    }

    /// The rows only.
    pub fn list(&self, project: &str) -> Vec<WorktreeDoc> {
        self.derive(project).map(|d| d.docs).unwrap_or_default()
    }

    /// The deletion jobs of a project, oldest first.
    pub fn jobs(&self, project: &str) -> Vec<JobDoc> {
        self.st()
            .jobs
            .iter()
            .filter(|j| j.project == project)
            .map(Job::doc)
            .collect()
    }

    /// Block until every deletion job of every project has run.
    pub fn jobs_idle(&self) {
        self.queue.wait_all_idle();
    }

    /// Block until the project's deletion jobs queued so far have run: never for another project's, never for the
    /// ones queued after.
    pub(crate) fn jobs_queued_ran(&self, project: &str) {
        self.queue.of(project).wait_queued();
    }
}

/// Days (possibly a fraction) as milliseconds.
pub(crate) fn age_ms(days: f64) -> i64 {
    const MS_PER_DAY: f64 = 86_400_000.0;
    #[allow(clippy::cast_possible_truncation)]
    let ms = (days * MS_PER_DAY).round() as i64;
    ms
}
