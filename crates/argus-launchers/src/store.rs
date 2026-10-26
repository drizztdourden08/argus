//! The storage the run service needs. The traits are synchronous so a SQLite store can
//! implement them with one connection behind a mutex; [`crate::MemoryStore`] is the in-memory
//! implementation.

use crate::attach::AttachmentRow;
use crate::record::{ListFilter, LogCommit, LogLine, LogRead, NewRun, RunPatch, RunRow};

/// Run rows. Ordering notes use "insertion order" for what SQL calls `rowid`.
pub trait RunStore: Send + Sync {
    /// The next run number of a scope (`R-<n>`), counting from 1.
    fn next_number(&self, scope: &str) -> u64;

    /// Stores a new `queued` run and returns it.
    fn insert(&self, new: NewRun) -> RunRow;

    fn get(&self, scope: &str, rid: &str) -> Option<RunRow>;

    /// Applies the patch and returns the updated row, `None` when the run is gone.
    fn update(&self, scope: &str, rid: &str, patch: &RunPatch) -> Option<RunRow>;

    /// Newest first (`queued_at`, then insertion order), at most `filter.row_limit()` rows.
    fn list(&self, scope: Option<&str>, filter: &ListFilter) -> Vec<RunRow>;

    /// Queued and running runs of a name, newest insertion first.
    fn live_by_name(&self, scope: &str, name: &str) -> Vec<RunRow>;

    /// Every queued and running run, in insertion order.
    fn live_rows(&self) -> Vec<RunRow>;

    /// Every queued run, oldest `queued_at` first, then insertion order.
    fn queued_rows(&self) -> Vec<RunRow>;

    /// The distinct `(scope, name)` pairs that have runs.
    fn groups(&self) -> Vec<(String, String)>;

    fn scopes(&self) -> Vec<String>;

    /// The runs of a name that are over, newest `queued_at` first.
    fn finished(&self, scope: &str, name: &str) -> Vec<RunRow>;

    /// Every run of a scope, in insertion order.
    fn of_scope(&self, scope: &str) -> Vec<RunRow>;

    /// Deletes the run with its log lines and attachment rows.
    fn remove(&self, scope: &str, rid: &str);
}

pub trait LogStore: Send + Sync {
    /// Applies one batch of the log writer in one transaction; the error text is logged by
    /// the caller and the batch is lost.
    fn commit(&self, scope: &str, rid: &str, commit: &LogCommit) -> Result<(), String>;

    /// Lines in ascending `seq` order.
    fn read(&self, scope: &str, rid: &str, read: &LogRead) -> Vec<LogLine>;
}

pub trait AttachmentStore: Send + Sync {
    /// The attachments of a run, oldest first.
    fn attachments_of(&self, scope: &str, rid: &str) -> Vec<AttachmentRow>;

    fn add_attachment(&self, scope: &str, rid: &str, row: AttachmentRow);
}

/// Everything the run service stores.
pub trait Storage: RunStore + LogStore + AttachmentStore {}

impl<T: RunStore + LogStore + AttachmentStore> Storage for T {}
