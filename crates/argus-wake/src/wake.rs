//! The records of the wake engine: a wake going in, a row in the wakes table, an event going out.

use serde::Serialize;

/// One wake as a producer hands it to the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeInput {
    /// Project key of the board the wake belongs to.
    pub scope: String,
    /// The thread that should look at its inbox.
    pub thread: String,
    /// Why (one of the reasons in [`crate::reasons`]).
    pub reason: String,
    /// What it is about (an entry, a test, a thread).
    pub subject: Option<String>,
    /// The line the thread hears.
    pub text: String,
}

impl WakeInput {
    /// A wake with a subject.
    #[must_use]
    pub fn about(
        scope: &str,
        thread: &str,
        reason: &str,
        subject: &str,
        text: impl Into<String>,
    ) -> Self {
        Self {
            scope: scope.to_owned(),
            thread: thread.to_owned(),
            reason: reason.to_owned(),
            subject: Some(subject.to_owned()),
            text: text.into(),
        }
    }
}

/// Where a row of the wakes table stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WakeStatus {
    /// Waiting for a window to close, or closed with nobody listening.
    Queued,
    /// An event that listeners received.
    Delivered,
    /// Folded into the event of its window.
    Coalesced,
}

/// One row of the wakes table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeRow {
    /// `W-n`.
    pub id: String,
    /// The counter behind the id.
    pub n: u64,
    /// Project key.
    pub scope: String,
    /// Target thread.
    pub thread: String,
    /// Why it was made.
    pub reason: String,
    /// What it is about.
    pub subject: Option<String>,
    /// The line the thread hears.
    pub text: String,
    /// When it was made (ISO).
    pub at: String,
    /// Where it stands.
    pub status: WakeStatus,
    /// How many listeners took the event.
    pub listeners: u32,
    /// When the event was delivered (ISO).
    pub delivered_at: Option<String>,
    /// The lines of the event; `None` while the window is open.
    pub lines: Option<Vec<String>>,
    /// How many wakes the event folds.
    pub count: u32,
    /// The event row a coalesced row folds into.
    pub merged_into: Option<String>,
}

/// What a window closing writes onto the lead row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedEvent {
    /// The event lines.
    pub lines: Vec<String>,
    /// How many wakes it folds.
    pub count: u32,
    /// Delivered, or still queued.
    pub status: WakeStatus,
    /// How many listeners took it.
    pub listeners: u32,
    /// When it was delivered (ISO).
    pub delivered_at: Option<String>,
}

/// An event as listeners receive it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WakeEvent {
    /// The id of the first wake of the window.
    pub id: String,
    /// Project key.
    pub scope: String,
    /// Target thread.
    pub thread: String,
    /// When the first wake was made (ISO).
    pub at: String,
    /// The first eight texts, then `+N more`.
    pub lines: Vec<String>,
    /// How many wakes the event folds.
    pub count: u32,
}

impl WakeRow {
    /// The row of a wake just recorded: queued, `n`th of all, waiting for its window.
    #[must_use]
    pub fn queued(n: u64, wake: &WakeInput, at: &str) -> Self {
        Self {
            id: format!("W-{n}"),
            n,
            scope: wake.scope.clone(),
            thread: wake.thread.clone(),
            reason: wake.reason.clone(),
            subject: wake.subject.clone(),
            text: wake.text.clone(),
            at: at.to_owned(),
            status: WakeStatus::Queued,
            listeners: 0,
            delivered_at: None,
            lines: None,
            count: 1,
            merged_into: None,
        }
    }
}

impl WakeEvent {
    /// The event a closed row stands for.
    #[must_use]
    pub fn of_row(row: &WakeRow) -> Self {
        Self {
            id: row.id.clone(),
            scope: row.scope.clone(),
            thread: row.thread.clone(),
            at: row.at.clone(),
            lines: row.lines.clone().unwrap_or_default(),
            count: row.count,
        }
    }
}
