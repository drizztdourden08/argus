//! A ping as a caller asks for it, and what `send` answers.

use crate::kinds::{By, ToastState};
use crate::level::Level;
use crate::record::Notification;

/// A ping to the owner. The service cleans the text and the title.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PingRequest {
    /// The title; empty or none means `<Project name>: <thread>`, else the project, else `Argus`.
    pub title: Option<String>,
    /// The text; a ping with no text is not recorded.
    pub text: String,
    /// The level.
    pub level: Level,
    /// The project key.
    pub project: Option<String>,
    /// The thread that pings.
    pub thread: Option<String>,
    /// Who sends it.
    pub by: By,
    /// The click address; ignored unless it is an http address on this machine.
    pub url: Option<String>,
}

impl PingRequest {
    /// A `ping` from a thread with this text.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }
}

/// What `send` answers; it never fails its caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The stored notification, none when nothing was recorded.
    pub notification: Option<Notification>,
    /// What became of the toast.
    pub toast: ToastState,
    /// The sentence saying why there was no plain toast.
    pub why: Option<String>,
}

impl Outcome {
    pub(crate) fn recorded(notification: Notification) -> Self {
        Self {
            toast: notification.toast,
            why: notification.why.clone(),
            notification: Some(notification),
        }
    }

    pub(crate) fn unrecorded(toast: ToastState, why: &str) -> Self {
        Self {
            notification: None,
            toast,
            why: Some(why.to_owned()),
        }
    }

    /// True when the notification was stored.
    #[must_use]
    pub fn is_recorded(&self) -> bool {
        self.notification.is_some()
    }

    /// The id `n-<n>` of the stored notification.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.notification.as_ref().map(|note| note.id.as_str())
    }
}
