//! Who sent a notification and what became of its toast.
// test-tag: critical

use serde::{Deserialize, Serialize};

/// Who sent the notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum By {
    /// A registered thread, the default.
    #[default]
    Thread,
    /// The lead of the portfolio board.
    Overlord,
    /// Argus itself, from a board event.
    Auto,
    /// The owner's page.
    Page,
}

impl By {
    /// The sender named by `by`; `source` `auto` means `Auto`; anything else is a thread.
    #[must_use]
    pub fn resolve(by: Option<&str>, source: Option<&str>) -> Self {
        match by {
            Some("thread") => Self::Thread,
            Some("overlord") => Self::Overlord,
            Some("auto") => Self::Auto,
            Some("page") => Self::Page,
            _ if source == Some("auto") => Self::Auto,
            _ => Self::Thread,
        }
    }
}

/// What became of the toast of a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToastState {
    /// The toast was shown, or a tray balloon took its place.
    Shown,
    /// No toast was asked for: info, a quiet level, or notifications off.
    Skipped,
    /// The toast was asked for and could not be shown.
    Failed,
    /// Over a rate limit; recorded only.
    Limited,
}

impl ToastState {
    /// The lowercase name used in records and sentences.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shown => "shown",
            Self::Skipped => "skipped",
            Self::Failed => "failed",
            Self::Limited => "limited",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::By;

    #[test]
    fn the_sender_follows_by_then_source() {
        assert_eq!(By::resolve(Some("nobody"), None), By::Thread);
        assert_eq!(By::resolve(None, Some("auto")), By::Auto);
        assert_eq!(By::resolve(Some("page"), Some("auto")), By::Page);
        assert_eq!(By::resolve(Some("overlord"), None), By::Overlord);
    }
}
