//! The notification record: the row of the `notifications` table (Node migration 6) and its doc.

use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::kinds::{By, ToastState};
use crate::level::Level;

/// How many notifications the store keeps; the newest ones.
pub const KEEP_ROWS: usize = 50;

/// What a ping says, cleaned and ready to be recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    /// What was said and who said it.
    pub message: Message,
    /// What became of the toast.
    pub toast: ToastState,
    /// A sentence for everything but a plain `shown`.
    pub why: Option<String>,
}

impl Message {
    /// The record of this message with its time, toast state and reason.
    #[must_use]
    pub fn record(self, at: String, toast: ToastState, why: Option<String>) -> NewNotification {
        NewNotification {
            at,
            message: self,
            toast,
            why,
        }
    }
}

/// One stored notification, as the doc `n-<n>` of the `notifications` collection holds it. The
/// fields of its message (`title`, `text`, `level`, `project`, `thread`, `by`, `url`) read
/// straight through it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notification {
    /// `n-<n>`.
    pub id: String,
    /// The row number, growing by one per notification.
    pub n: u64,
    /// The time, ISO 8601 with milliseconds.
    pub at: String,
    /// What became of the toast.
    pub toast: ToastState,
    /// A sentence for everything but a plain `shown`.
    pub why: Option<String>,
    #[serde(flatten)]
    message: Message,
}

impl Notification {
    /// The stored form of `new` as row number `n`.
    #[must_use]
    pub fn from_new(n: u64, new: NewNotification) -> Self {
        Self {
            id: format!("n-{n}"),
            n,
            at: new.at,
            toast: new.toast,
            why: new.why,
            message: new.message,
        }
    }
}

impl Deref for Notification {
    type Target = Message;

    fn deref(&self) -> &Message {
        &self.message
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
