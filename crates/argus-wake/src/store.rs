//! What the wake engine reads and writes: the wakes table, behind a trait the server plugs SQLite into.

use argus_core::StorageError;

use crate::time::iso;
use crate::wake::{FinishedEvent, WakeInput, WakeRow};

/// Rows older than this many days are pruned.
pub const KEEP_DAYS: i64 = 7;

/// How many rows of a project the page mirror shows.
pub const MIRROR_KEEP: usize = 50;

/// The result of a storage call.
pub type StoreResult<T> = Result<T, StorageError>;

/// Which rows of a project to list, newest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowsQuery {
    /// At most this many rows.
    pub limit: usize,
    /// Only this thread (`None` and `*` list every thread).
    pub thread: Option<String>,
}

/// The wakes table.
pub trait WakeStore: Send + Sync {
    /// Record a wake as `queued` and return its row (`W-n`, one counter for all projects).
    ///
    /// # Errors
    /// When the row cannot be written.
    fn insert(&self, wake: &WakeInput, at: &str) -> StoreResult<WakeRow>;

    /// Close a window: `lead` carries the event, `others` fold into it.
    ///
    /// # Errors
    /// When the rows cannot be written.
    fn finish(&self, lead: &WakeRow, others: &[WakeRow], event: &FinishedEvent) -> StoreResult<()>;

    /// An event a late listener took: it becomes `delivered`.
    ///
    /// # Errors
    /// When the row cannot be written.
    fn mark_delivered(&self, id: &str, listeners: u32, at: &str) -> StoreResult<()>;

    /// Events nobody received yet (window closed, no listener), oldest first; thread `*` lists all.
    ///
    /// # Errors
    /// When the rows cannot be read.
    fn queued_events(
        &self,
        scope: &str,
        thread: &str,
        since_iso: &str,
    ) -> StoreResult<Vec<WakeRow>>;

    /// Rows whose window never closed (the server stopped meanwhile), oldest first.
    ///
    /// # Errors
    /// When the rows cannot be read.
    fn unflushed(&self) -> StoreResult<Vec<WakeRow>>;

    /// One row by id.
    ///
    /// # Errors
    /// When the row cannot be read.
    fn get(&self, id: &str) -> StoreResult<Option<WakeRow>>;

    /// The newest rows of a project, newest first.
    ///
    /// # Errors
    /// When the rows cannot be read.
    fn rows(&self, scope: &str, query: &RowsQuery) -> StoreResult<Vec<WakeRow>>;

    /// When the newest wake of a reason went to a thread (about a subject): epoch ms, 0 when none.
    ///
    /// # Errors
    /// When the rows cannot be read.
    fn last_at(
        &self,
        scope: &str,
        thread: &str,
        reason: &str,
        subject: Option<&str>,
    ) -> StoreResult<i64>;

    /// Every project that has wake rows.
    ///
    /// # Errors
    /// When the rows cannot be read.
    fn scopes(&self) -> StoreResult<Vec<String>>;

    /// Delete rows older than [`KEEP_DAYS`] (see [`prune_cutoff`]); returns the projects that lost rows.
    ///
    /// # Errors
    /// When the rows cannot be deleted.
    fn prune(&self, now_ms: i64) -> StoreResult<Vec<String>>;
}

/// The ISO instant before which rows are pruned.
#[must_use]
pub fn prune_cutoff(now_ms: i64) -> String {
    iso(now_ms - KEEP_DAYS * 86_400_000)
}
