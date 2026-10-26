//! What the crate asks of the rest of Argus: the board and a small key-value store, the activity log and the wake
//! queue, the machine's processes.

use crate::board::BoardFacts;
use std::fmt;

/// A storage failure, as a sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreError(pub String);

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for StoreError {}

/// The storage the crate needs. Rules, keeps and answers live on the project's board; the "already told" and "made by a
/// removal" memories are small texts in a key-value table (`worktrees:sent:<project>`, `worktrees:made:<project>`).
pub trait WorktreeStore: Send + Sync {
    /// The project's board as one snapshot, or none when there is no such project.
    fn board(&self, project: &str) -> Option<BoardFacts>;
    /// The projects that have a board.
    fn projects(&self) -> Vec<String>;
    fn kv_get(&self, key: &str) -> Option<String>;
    fn kv_set(&self, key: &str, value: &str) -> Result<(), StoreError>;
    /// Drop the keeps and the answers of worktrees that no longer exist (a locked board write on the store's side).
    fn drop_kept(
        &self,
        project: &str,
        keep_ids: &[String],
        answer_ids: &[String],
    ) -> Result<(), StoreError>;
}

/// One line of the activity log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityLine {
    pub kind: &'static str,
    pub level: &'static str,
    pub text: String,
    pub project: String,
    pub thread: Option<String>,
}

/// One wake for a thread.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WakeEvent {
    pub scope: String,
    pub thread: String,
    pub reason: &'static str,
    pub subject: String,
    pub text: String,
}

/// Where the crate says what it did and whom it wants to wake.
pub trait Notifier: Send + Sync {
    fn activity(&self, line: ActivityLine);
    /// Queue wakes. An error means the queue refused them: they are not remembered as told.
    fn wake(&self, events: &[WakeEvent]) -> Result<(), String>;
}

/// A running process, as far as a removal cares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProcInfo {
    pub pid: u32,
    pub name: String,
    pub cmd: String,
    pub path: Option<String>,
    pub cwd: Option<String>,
}

/// The machine's processes now.
pub trait ProcessLister: Send + Sync {
    fn list(&self) -> Result<Vec<ProcInfo>, String>;
}
