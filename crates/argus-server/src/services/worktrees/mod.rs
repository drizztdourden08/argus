//! The worktree scanner: the registered thread tells Argus where its worktrees live (rules on
//! the board), the service looks there with read-only git, mirrors what it finds as docs for the
//! Worktrees tab, and wakes the thread when a worktree needs a decision. Removal goes only
//! through its guards, from the routes. Shadow mode scans and shows, and removes nothing.

use std::sync::Arc;
use std::time::Duration;

use argus_core::remote::ModeSource;
use argus_worktrees::{
    Deps, Fs, GhRunner, Git, GitExec, ProcessLister, RealFs, Service as WorktreeService,
    SystemClock, SystemExec, SystemGh,
};
use tokio::sync::mpsc::unbounded_channel;

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::worktrees::{BoardWorktrees, MachineProcs, TreeNotifier};
use crate::{ServeOptions, Services};

mod config;
mod publish;
mod timers;
mod watch;

pub(crate) use publish::publish;

/// How the scanner starts; the fields after `enabled` are seams for tests.
#[derive(Clone)]
pub struct WorktreesOptions {
    /// Run the scanner.
    pub enabled: bool,
    /// Scan on timers: the first scan after `startDelayMs`, then every `liveMs` the live check and
    /// the look at the disk (a scan only of what changed, and of everything every `scanMs`), and the
    /// rescans a removal or a change of rules asks for.
    pub timers: bool,
    /// Another way to run `git` than the command.
    pub git: Option<Arc<dyn GitExec>>,
    /// Another way to ask GitHub whether a pull request merged than the `gh` command.
    pub gh: Option<Arc<dyn GhRunner>>,
    /// Another list of the machine's processes than the process monitor's.
    pub procs: Option<Arc<dyn ProcessLister>>,
}

impl Default for WorktreesOptions {
    fn default() -> Self {
        WorktreesOptions {
            enabled: true,
            timers: true,
            git: None,
            gh: None,
            procs: None,
        }
    }
}

fn git_of(svc: &Services, opts: &WorktreesOptions) -> Git {
    let (max_procs, read_ms, remove_ms) = config::git_limits(svc);
    let exec: Arc<dyn GitExec> = opts
        .git
        .clone()
        .unwrap_or_else(|| Arc::new(SystemExec::default()));
    Git::with_limits(
        exec,
        max_procs,
        Duration::from_millis(read_ms),
        Duration::from_secs(120),
    )
    .with_remove_timeout(Duration::from_millis(remove_ms))
}

fn deps_of(svc: &Services, opts: &WorktreesOptions) -> Deps {
    let store = BoardWorktrees::new(
        svc.boards(),
        Arc::clone(&svc.db),
        svc.paths.coord_home.clone(),
        svc.shadow,
    )
    .with_runs(svc.runs.clone())
    .with_auto(config::auto_setting(svc));
    let procs: Arc<dyn ProcessLister> = opts
        .procs
        .clone()
        .unwrap_or_else(|| Arc::new(MachineProcs::new(svc.procs.clone(), svc.runs.clone())));
    let gh: Arc<dyn GhRunner> = opts.gh.clone().unwrap_or_else(|| Arc::new(SystemGh));
    let fs: Arc<dyn Fs> = Arc::new(RealFs);
    let remote: Arc<dyn ModeSource> = svc.remote.clone();
    Deps {
        store: Arc::new(store),
        notifier: Arc::new(TreeNotifier::new(
            ActivityAdapter::of(svc),
            svc.wake_hook(),
            Arc::clone(&svc.log),
        )),
        fs,
        git: git_of(svc, opts),
        gh,
        remote,
        procs,
        clock: Arc::new(SystemClock),
    }
}

/// The scanner needs the boards, so it needs the sync service.
pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<WorktreeService>> {
    let options = &opts.worktrees;
    svc.sync.as_ref()?;
    options.enabled.then(|| {
        Arc::new(WorktreeService::new(
            deps_of(svc, options),
            config::options_of(svc),
        ))
    })
}

/// The docs are the live scan's only: whatever an earlier run left is emptied until this run has
/// looked.
pub(crate) fn start(svc: &Arc<Services>) {
    let Some(service) = svc.worktrees.clone() else {
        return;
    };
    service.start();
    for key in service.deps.store.projects() {
        publish(svc, &service, &key);
    }
    let (asked, scans) = unbounded_channel();
    watch::watch(svc, asked);
    if svc.tasks.runs(crate::services::tasks::Timer::Worktrees) {
        timers::start(svc, &service, config::timing_of(svc), scans);
    }
}

pub(crate) fn stop(svc: &Services) {
    timers::stop(svc);
    if let Some(service) = &svc.worktrees {
        service.stop();
    }
}
