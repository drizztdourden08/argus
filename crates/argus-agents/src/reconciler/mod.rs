//! Compares the board's agent records with what Claude itself keeps for the owning session, and
//! acts on the differences. A record made by hand, or whose `SubagentStop` never arrived, can say
//! `working` for hours about an agent that ended or never existed. Every `agents.reconcileMs`
//! (default 2 minutes) each record that is working, waiting or stop-requested is looked up.
//!
//! What Claude leaves on disk (read-only):
//! - `projects/<slug>/<sessionId>/subagents/agent-<id>.jsonl`, the sub-agent's transcript, and
//!   `agent-<id>.meta.json` beside it, with `finished: true` once the turn ended and
//!   `stoppedByUser: true` when the user stopped it. `<slug>` is the working directory with every
//!   character that is not a letter or digit turned into `-`.
//! - `<tmp>/claude/<slug>/<sessionId>/tasks/<id>.output`, one file per background task: a shell
//!   ends in `[exited with code N]` or `[killed]`.
//!
//! What is done, per record, in five steps: verify the ids, link a record with no id to a
//! transcript, merge duplicates, classify (alive, quiet, idle, orphan, pending, unknown), and
//! reopen a closed record whose agent grows again. The flags live on the record, so a restart
//! neither forgets what was sent nor sends it again, and the closing clocks wait one full period
//! after the start. A poll long after the previous one (the machine slept) only reports.

mod apply;
mod classify;
mod duplicates;
mod ids;
mod link;
mod orphan;
mod pass;
mod report;
mod resume;
mod service;
mod session;
mod state;

pub use report::{Class, Counts, Finding, RunReport};
pub use service::AgentReconciler;

use crate::config::ReconcilerOptions;
use crate::fs::FileSystem;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

/// The inputs of an evaluation that do not change while it runs.
struct Env<'a> {
    opts: &'a ReconcilerOptions,
    fs: &'a dyn FileSystem,
    project_dirs: &'a Mutex<HashMap<String, PathBuf>>,
    boot_at: i64,
}

impl Env<'_> {
    fn projects_dir(&self) -> PathBuf {
        self.opts.claude_home.join("projects")
    }

    fn cached_dir(&self, sid: &str) -> Option<PathBuf> {
        let dirs = self
            .project_dirs
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        dirs.get(sid).cloned()
    }

    fn remember_dir(&self, sid: &str, dir: PathBuf) {
        let mut dirs = self
            .project_dirs
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        dirs.insert(sid.to_owned(), dir);
    }
}
