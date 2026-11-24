//! What the monitor needs from the outside: a sampler, a store, a host, a kill and a window probe.

use crate::project::{ProjectFolders, ProjectRoot};
use crate::types::{Meta, ProcDoc, Sample};
use std::collections::HashMap;

/// Takes one look at every process of the machine.
pub trait Sampler: Send + Sync {
    /// One sample.
    ///
    /// # Errors
    /// The reason the machine could not be read.
    fn sample(&self) -> Result<Sample, String>;

    /// The folder each of these pids runs in; a pid whose folder cannot be read is left out.
    fn cwds(&self, _pids: &[u32]) -> HashMap<u32, String> {
        HashMap::new()
    }
}

/// Tells which windows do not respond, for the processes whose rule asks for the hung flag.
pub trait Responder: Send + Sync {
    /// For each pid with a window, whether it responds; a pid without a window is left out.
    ///
    /// # Errors
    /// The reason the windows could not be asked.
    fn responding(&self, pids: &[u32]) -> Result<HashMap<u32, bool>, String>;
}

/// The outcome of asking the system to end a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillResult {
    pub ok: bool,
    pub detail: Option<String>,
}

/// Ends processes and tells whether one still runs.
pub trait Killer: Send + Sync {
    /// Ends `pid`; with `tree` also `descendants` (or the whole tree where the system can).
    fn kill(&self, pid: u32, tree: bool, descendants: &[u32]) -> KillResult;

    /// Whether the pid still runs.
    fn is_alive(&self, pid: u32) -> bool;
}

/// Keeps the listing and its meta where the dashboard reads them (scope `_portfolio`).
pub trait ProcsStore: Send + Sync {
    /// Replaces collection `procs` with `docs` and sets doc `meta/procs`, in one transaction.
    ///
    /// # Errors
    /// The reason the docs could not be written.
    fn replace(&self, docs: &[ProcDoc], meta: Option<&Meta>) -> Result<(), String>;
}

/// A wake for the Overlord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeRequest {
    pub scope: String,
    pub thread: String,
    pub reason: String,
    pub subject: String,
    pub text: String,
}

/// How loud an activity line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Info,
    Warn,
    Error,
}

/// One line of the activity log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub kind: &'static str,
    pub level: Level,
    pub text: String,
}

/// The dashboard around the monitor: projects, the tray, the wake engine, the activity log and the log.
pub trait ProcsHost: Send + Sync {
    /// The folders of the known projects.
    fn project_roots(&self) -> Vec<ProjectRoot>;

    /// Every folder of each known project: its root, its worktrees and its ci root. By default
    /// only the roots.
    fn project_folders(&self) -> Vec<ProjectFolders> {
        self.project_roots()
            .into_iter()
            .map(|r| ProjectFolders {
                key: r.key,
                folders: vec![r.root],
            })
            .collect()
    }

    /// The lead (lead thread) of a project, when it has one.
    fn lead_of(&self, _project: &str) -> Option<String> {
        None
    }

    /// The tray's pid, when it runs.
    fn tray_pid(&self) -> Option<u32>;

    /// The id of the Overlord's board, where the process alerts go.
    fn overlord_board(&self) -> String;

    /// The lead thread of the portfolio board, when there is one.
    fn lead(&self) -> Option<String>;

    /// Hands a wake to the engine.
    ///
    /// # Errors
    /// The reason the engine refused it.
    fn submit_wake(&self, wake: &WakeRequest) -> Result<(), String>;

    /// Adds a line to the activity log.
    fn activity(&self, entry: &Activity);

    /// Logs a warning with its detail.
    fn warn(&self, message: &str, detail: &str);
}
