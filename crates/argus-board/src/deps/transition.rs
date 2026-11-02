//! The history of a dependency: its rows and the transitions a call reports.

use serde::{Deserialize, Serialize};

use super::model::DepState;

/// One transition in the history of a dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    /// Grows forever; names the wake of the transition.
    pub seq: i64,
    /// When.
    pub at: String,
    /// Who: an actor or `argus`.
    pub by: String,
    /// The state before.
    pub from: DepState,
    /// The state after.
    pub to: DepState,
    /// Why, when it matters.
    #[serde(default)]
    pub reason: Option<String>,
}

/// A state change the ledger made. The server turns it into a wake (`dep#seq` names it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Transition {
    /// The dependency.
    pub dep: String,
    /// The history row number.
    pub seq: i64,
    /// The state before.
    pub from: DepState,
    /// The state after.
    pub to: DepState,
}
