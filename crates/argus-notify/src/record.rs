//! The notification record: the row of the `notifications` table (Node migration 6) and its doc.

use serde::{Deserialize, Serialize};

use crate::kinds::{By, ToastState};
use crate::level::Level;

/// How many notifications the store keeps; the newest ones.
pub const KEEP_ROWS: usize = 50;

/// What a ping says, cleaned and ready to be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// The title, at most 60 characters.
    pub title: String,
    /// The text, at most 240 characters.
    pub text: String,
    /// The level.
    pub level: Level,
    /// The project key.
    pub project: Option<String>,
    /// The thread that pinged.
    pub thread: Option<String>,
    /// Who sent it.
    pub by: By,
    /// The click address of the toast; none for a plain `info`.
    pub url: Option<String>,
}

/// A notification about to be inserted: the message and what became of its toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewNotification {
    /// The time, ISO 8601 with milliseconds.
    pub at: String,
    /// The level.
    pub level: Level,
    /// The title.
    pub title: String,
    /// The text.
    pub text: String,
    /// The project key.
    pub project: Option<String>,
    /// The thread.
    pub thread: Option<String>,
    /// Who sent it.
    pub by: By,
    /// What became of the toast.
    pub toast: ToastState,
    /// The click address of the toast.
    pub url: Option<String>,
    /// A sentence for everything but a plain `shown`.
    pub why: Option<String>,
}

impl Message {
    /// The record of this message with its time, toast state and reason.
    #[must_use]
    pub fn record(self, at: String, toast: ToastState, why: Option<String>) -> NewNotification {
        NewNotification {
            at,
            level: self.level,
            title: self.title,
            text: self.text,
            project: self.project,
            thread: self.thread,
            by: self.by,
            toast,
            url: self.url,
            why,
        }
    }
}

/// One stored notification, as the doc `n-<n>` of the `notifications` collection holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    /// `n-<n>`.
    pub id: String,
    /// The row number, growing by one per notification.
    pub n: u64,
    /// The time, ISO 8601 with milliseconds.
    pub at: String,
    /// The level.
    pub level: Level,
    /// The title.
    pub title: String,
    /// The text.
    pub text: String,
    /// The project key.
    pub project: Option<String>,
    /// The thread.
    pub thread: Option<String>,
    /// Who sent it.
    pub by: By,
    /// What became of the toast.
    pub toast: ToastState,
    /// The click address of the toast.
    pub url: Option<String>,
    /// A sentence for everything but a plain `shown`.
    pub why: Option<String>,
}

impl Notification {
    /// The stored form of `new` as row number `n`.
    #[must_use]
    pub fn from_new(n: u64, new: NewNotification) -> Self {
        Self {
            id: format!("n-{n}"),
            n,
            at: new.at,
            level: new.level,
            title: new.title,
            text: new.text,
            project: new.project,
            thread: new.thread,
            by: new.by,
            toast: new.toast,
            url: new.url,
            why: new.why,
        }
    }
}

/// What `list` asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    /// At most this many, newest first; clamped to 1 and `KEEP_ROWS`.
    pub limit: usize,
    /// Only this level.
    pub level: Option<Level>,
    /// Only this project.
    pub project: Option<String>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            limit: KEEP_ROWS,
            level: None,
            project: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{KEEP_ROWS, Notification};
    use crate::kinds::{By, ToastState};
    use crate::level::Level;

    #[test]
    fn the_doc_has_the_shape_of_the_node_doc() {
        let note = Notification {
            id: "n-1".into(),
            n: 1,
            at: "2026-10-26T12:00:00.000Z".into(),
            level: Level::Ping,
            title: "Rotp: lead".into(),
            text: "B1 is ready to test".into(),
            project: Some("rotp".into()),
            thread: None,
            by: By::Thread,
            toast: ToastState::Shown,
            url: None,
            why: None,
        };
        let doc = serde_json::to_value(&note).expect("serializes");
        assert_eq!(doc["by"], "thread");
        assert_eq!(doc["toast"], "shown");
        assert_eq!(doc["thread"], serde_json::Value::Null);
        assert_eq!(KEEP_ROWS, 50);
    }
}
