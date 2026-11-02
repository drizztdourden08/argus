//! The part of the dependency ledger and of the item mirrors the wake logic reads.

use serde::Deserialize;

use crate::lenient::{self, Text};

/// One side of a dependency: an item of a project, and the thread that owned it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DepEnd {
    /// The project key.
    pub project: Text,
    /// The item id.
    pub item: Text,
    /// The thread that owns the item.
    pub thread: Text,
}

/// One transition of a dependency.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DepStep {
    /// The number of the transition, growing forever; it names the wake. 0 when missing.
    pub seq: u64,
    /// When it happened (ISO).
    pub at: Text,
    /// The state it left.
    pub from: Text,
    /// The state it reached.
    pub to: Text,
    /// Why, when the actor said.
    pub reason: Text,
}

/// A dependency of the ledger on the portfolio board.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DepView {
    /// `D-n`.
    pub id: Text,
    /// The requester.
    pub from: DepEnd,
    /// The assignee.
    pub to: DepEnd,
    /// The title of the request.
    pub title: Text,
    /// `opening`, `open`, `done`, `blocked`...
    pub state: Text,
    /// When the dependency first became `open` (ISO).
    pub opened_at: Text,
    /// When the assignee first read the request (ISO).
    pub acked_at: Text,
    /// Why it was last blocked or closed.
    pub reason: Text,
    /// The last transitions, oldest first.
    #[serde(deserialize_with = "lenient::list")]
    pub history: Vec<DepStep>,
}

/// The `waitsOn` mirror on the requester item.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WaitsOn {
    /// The state of the dependency.
    pub state: Text,
}

impl WaitsOn {
    /// Does the item still wait for the other board: the request is being opened or is open?
    #[must_use]
    pub fn is_waiting(&self) -> bool {
        self.state.is_any(&["opening", "open"])
    }
}

/// The `requestedBy` mirror on the assignee item.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RequestedBy {
    /// The state of the dependency.
    pub state: Text,
}

impl RequestedBy {
    /// Is the assignee blocked on the requester, who has to answer first?
    #[must_use]
    pub fn is_blocked(&self) -> bool {
        self.state.is("blocked")
    }
}
