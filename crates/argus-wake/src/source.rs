//! Where the wake logic reads the project boards from.

use std::path::PathBuf;
use std::sync::Arc;

use argus_core::remote::SyncRecord;

use crate::board::BoardView;

/// The boards of every project on this machine.
pub trait BoardSource: Send + Sync {
    /// The project keys that have a board (the portfolio board included).
    fn scopes(&self) -> Vec<String>;

    /// The board of a project as the last export left it.
    fn board(&self, scope: &str) -> Option<Arc<BoardView>>;
}

/// What the agent reminders ask the live Claude tasks.
pub trait AgentLive: Send + Sync {
    /// When the agent last showed a sign of life in its transcript (ISO), when Claude knows it.
    fn last_activity_at(&self, scope: &str, agent: &str) -> Option<String>;
}

/// An [`AgentLive`] that knows nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLive;

impl AgentLive for NoLive {
    fn last_activity_at(&self, _scope: &str, _agent: &str) -> Option<String> {
        None
    }
}

/// The last full remote sync of every project (`coord remote synced`).
pub trait SyncSource: Send + Sync {
    /// The records, a project at most once.
    fn syncs(&self) -> Vec<(String, SyncRecord)>;
}

/// A [`SyncSource`] that knows nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoSyncs;

impl SyncSource for NoSyncs {
    fn syncs(&self) -> Vec<(String, SyncRecord)> {
        Vec::new()
    }
}

/// The syncs of the `remote.json` of a coord home, read at every call.
#[derive(Debug, Clone)]
pub struct FileSyncs {
    coord_home: PathBuf,
}

impl FileSyncs {
    /// The syncs of a coord home.
    #[must_use]
    pub fn new(coord_home: PathBuf) -> Self {
        FileSyncs { coord_home }
    }
}

impl SyncSource for FileSyncs {
    fn syncs(&self) -> Vec<(String, SyncRecord)> {
        argus_core::remote::syncs(&self.coord_home)
    }
}
