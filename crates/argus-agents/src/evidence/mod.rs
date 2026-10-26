//! What Claude itself leaves on disk about a sub-agent, read-only, shared by the reconciler (classifies every live
//! record) and the reaper (must not close an agent that is still running).
//!
//! - `projects/<slug>/<sessionId>/subagents/agent-<id>.jsonl` (and `.meta.json`): the transcript and its meta.
//! - `<tmp>/claude/<slug>/<sessionId>/tasks/<id>.output`: a background task's output.
//!
//! A transcript is only appended when a message is complete, so a long tool call or a big file write leaves it
//! untouched for many minutes while the agent works. [`sign::last_sign`] therefore also looks at the output file of
//! the task and at the newest file of the record's worktree, and answers with the newest sign of life among the three.

pub(crate) mod session;
pub(crate) mod sign;
pub(crate) mod state;
mod walk;

use crate::config::ReconcilerOptions;
use crate::fs::FileSystem;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

/// The inputs of a look at the disk that do not change while it runs.
pub(crate) struct Env<'a> {
    pub opts: &'a ReconcilerOptions,
    pub fs: &'a dyn FileSystem,
    pub project_dirs: &'a Mutex<HashMap<String, PathBuf>>,
    pub boot_at: i64,
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
