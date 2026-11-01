//! What the crate asks of the rest of Argus: the board and a small key-value store, the activity log and the wake
//! queue, the machine's processes.

use crate::board::BoardFacts;
use argus_core::StorageError;

/// The storage the crate needs. Rules, keeps and answers live on the project's board; the "already told" and "made by a
/// removal" memories are small texts in a key-value table (`worktrees:sent:<project>`, `worktrees:made:<project>`).
pub trait WorktreeStore: Send + Sync {
    /// The project's board as one snapshot, or none when there is no such project.
    fn board(&self, project: &str) -> Option<BoardFacts>;
    /// The projects that have a board.
    fn projects(&self) -> Vec<String>;
    fn kv_get(&self, key: &str) -> Option<String>;
    fn kv_set(&self, key: &str, value: &str) -> Result<(), StorageError>;
    /// Drop the keeps and the answers of worktrees that no longer exist (a locked board write on the store's side).
    fn drop_kept(
        &self,
        project: &str,
        keep_ids: &[String],
        answer_ids: &[String],
    ) -> Result<(), StorageError>;
    /// One note from Argus into a thread's inbox, under the project's board lock. False when the thread is not on the board.
    fn post_inbox(&self, project: &str, thread: &str, text: &str) -> Result<bool, StorageError>;
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
    /// A failure of the scanner or of a request worker, for the server's log.
    fn problem(&self, text: &str);
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

/// One of Argus's own live runs. A dev server or a watcher holds the folders below its working directory open, and
/// Windows says nothing about the process of a folder held that way.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiveRun {
    pub id: String,
    pub project: String,
    pub cmd: String,
    pub cwd: String,
}

/// The machine's processes now.
pub trait ProcessLister: Send + Sync {
    fn list(&self) -> Result<Vec<ProcInfo>, String>;

    /// Argus's own runs that are live now; none where the process list knows no runs.
    fn runs(&self) -> Vec<LiveRun> {
        Vec::new()
    }
}
