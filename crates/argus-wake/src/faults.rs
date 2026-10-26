//! Wake faults: what the registry records when a wake failed, behind a trait the server plugs SQLite into.

use serde::Serialize;

use crate::store::StoreResult;

/// A wake fault as the CLI and the routes show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fault {
    /// Row id.
    pub id: i64,
    /// Project key.
    pub project: String,
    /// The thread that could not be woken.
    pub thread: String,
    /// When it opened (ISO).
    pub since: String,
    /// `no-listener`, `no-reaction`, `delivery-error`, `not-listening` or `board-closed`.
    pub reason: String,
    /// The wake that failed.
    pub wake_id: Option<String>,
    /// What happened, in words.
    pub detail: Option<String>,
    /// When it was cleared (ISO).
    pub cleared_at: Option<String>,
    /// Who cleared it (`registry` or the Overlord's thread).
    pub cleared_by: Option<String>,
    /// Why it was cleared.
    pub note: Option<String>,
}

/// A fault to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFault {
    /// Project key.
    pub scope: String,
    /// The thread that could not be woken.
    pub thread: String,
    /// When it opened (ISO).
    pub since: String,
    /// The reason code.
    pub reason: String,
    /// The wake that failed.
    pub wake_id: Option<String>,
    /// What happened, in words.
    pub detail: Option<String>,
}

/// A fault to clear.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearFault {
    /// Project key.
    pub scope: String,
    /// The thread.
    pub thread: String,
    /// When it was cleared (ISO).
    pub at: String,
    /// Who clears it.
    pub by: String,
    /// Why.
    pub note: String,
}

/// The `wake_faults` table: one open row per project and thread.
pub trait FaultStore: Send + Sync {
    /// The new fault, or `None` when the project and thread already have an open one.
    ///
    /// # Errors
    /// When the fault cannot be written.
    fn open(&self, fault: &NewFault) -> StoreResult<Option<Fault>>;

    /// The cleared fault, or `None` when there was none open.
    ///
    /// # Errors
    /// When the fault cannot be written.
    fn clear(&self, request: &ClearFault) -> StoreResult<Option<Fault>>;

    /// The open faults, or every fault with `all`, oldest first.
    ///
    /// # Errors
    /// When the faults cannot be read.
    fn list(&self, all: bool) -> StoreResult<Vec<Fault>>;
}
