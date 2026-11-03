//! Which transition of a dependency tells which side what.

use super::text;
use super::view::{DepEnd, DepStep, DepView};

/// What a transition of the ledger says to one side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    /// The request reached `open`: the assignee is asked.
    Request,
    /// The assignee item is `done`, `dropped` or `gone`: the requester resumes.
    Resolved,
    /// The assignee is blocked on the requester.
    Blocked,
    /// The requester withdrew: the assignee is told.
    Cancelled,
    /// The request is open again after `blocked` or `done`.
    Reopened,
}

impl Kind {
    /// What a transition says, when it says anything.
    pub(super) fn of(step: &DepStep) -> Option<Self> {
        match (step.from.as_str(), step.to.as_str()) {
            ("opening", "open") => Some(Self::Request),
            ("blocked" | "done", "open") => Some(Self::Reopened),
            (_, "done" | "dropped" | "gone") => Some(Self::Resolved),
            (_, "blocked") => Some(Self::Blocked),
            (_, "cancelled") => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// The reason of the wake.
    pub(super) fn reason(self) -> &'static str {
        match self {
            Self::Request => "dep-request",
            Self::Resolved => "dep-resolved",
            Self::Blocked => "dep-blocked",
            Self::Cancelled => "dep-cancelled",
            Self::Reopened => "dep-reopened",
        }
    }

    /// The side that hears it: the assignee for a request or a cancel, else the requester.
    pub(super) fn audience(self, dep: &DepView) -> &DepEnd {
        match self {
            Self::Request | Self::Cancelled => &dep.to,
            Self::Resolved | Self::Blocked | Self::Reopened => &dep.from,
        }
    }

    /// The line that side hears.
    pub(super) fn text(self, dep: &DepView, step: &DepStep) -> String {
        match self {
            Self::Request => text::request(dep),
            Self::Resolved => text::resolved(dep, step),
            Self::Blocked => text::blocked(dep, step),
            Self::Cancelled => text::cancelled(dep),
            Self::Reopened => text::reopened(dep),
        }
    }
}
