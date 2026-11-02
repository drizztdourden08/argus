//! The records of the ledger: a dependency, its two ends, its history rows and its states.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Where a dependency stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepState {
    /// Created, the mirrors are not both written yet.
    Opening,
    /// The assignee item exists and the requester waits on it.
    Open,
    /// The assignee item is done.
    Done,
    /// The assignee item was dropped.
    Dropped,
    /// The assignee waits on the requester.
    Blocked,
    /// The assignee item or board is gone.
    Gone,
    /// The requester withdrew, or its item ended.
    Cancelled,
}

impl DepState {
    /// The wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            DepState::Opening => "opening",
            DepState::Open => "open",
            DepState::Done => "done",
            DepState::Dropped => "dropped",
            DepState::Blocked => "blocked",
            DepState::Gone => "gone",
            DepState::Cancelled => "cancelled",
        }
    }

    /// Whether the dependency is an edge of the graph: `opening`, `open` or `blocked`.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, DepState::Opening | DepState::Open | DepState::Blocked)
    }
}

impl fmt::Display for DepState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The text is not a dependency state.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("state must be one of opening, open, done, dropped, blocked, gone, cancelled")]
pub struct UnknownState;

impl FromStr for DepState {
    type Err = UnknownState;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        [
            DepState::Opening,
            DepState::Open,
            DepState::Done,
            DepState::Dropped,
            DepState::Blocked,
            DepState::Gone,
            DepState::Cancelled,
        ]
        .into_iter()
        .find(|state| state.as_str() == s)
        .ok_or(UnknownState)
    }
}

/// One end of a dependency: a project, its item and the thread that owns the item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct End {
    /// The project key.
    pub project: String,
    /// The item, `null` on the assignee end until it exists.
    #[serde(default)]
    pub item: Option<String>,
    /// The thread that owns the item.
    #[serde(default)]
    pub thread: Option<String>,
}

impl End {
    /// `project/I-n`, with `?` for an item that does not exist yet.
    #[must_use]
    pub fn node(&self) -> String {
        node(&self.project, self.item.as_deref().unwrap_or("?"))
    }
}

/// `project/I-n`: a node of the dependency graph.
#[must_use]
pub fn node(project: &str, item: &str) -> String {
    format!("{project}/{item}")
}

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

/// A dependency: one request from an item of a project to an item of another.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dep {
    /// `D-n`.
    pub id: String,
    /// One more on every change.
    pub rev: i64,
    /// When it was asked.
    pub created_at: String,
    /// Who asked: `project/T-n` or `project lead`.
    pub created_by: String,
    /// The last change.
    pub updated_at: String,
    /// Where it stands.
    pub state: DepState,
    /// The requester.
    pub from: End,
    /// The assignee.
    pub to: End,
    /// The title of the request.
    pub title: String,
    /// Why it is blocked, cancelled or gone.
    #[serde(default)]
    pub reason: Option<String>,
    /// When it first opened.
    #[serde(default)]
    pub opened_at: Option<String>,
    /// When it last ended.
    #[serde(default)]
    pub resolved_at: Option<String>,
    /// The last 20 transitions.
    #[serde(default)]
    pub history: Vec<Row>,
}

impl Dep {
    /// A ledger doc as a record; `None` when it does not have the shape.
    #[must_use]
    pub fn from_value(doc: &Value) -> Option<Dep> {
        serde_json::from_value(doc.clone()).ok()
    }

    /// The record as the contract's JSON.
    #[must_use]
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// The requester node, `project/I-n`.
    #[must_use]
    pub fn requester(&self) -> String {
        self.from.node()
    }

    /// The assignee node, `project/I-n`.
    #[must_use]
    pub fn assignee(&self) -> String {
        self.to.node()
    }

    /// One line: `D-4 open: archipelia/I-7 -> rotp/I-12 "Expose the save hook"`.
    #[must_use]
    pub fn line(&self) -> String {
        format!(
            "{} {}: {} -> {} \"{}\"",
            self.id,
            self.state,
            self.requester(),
            self.assignee(),
            self.title
        )
    }
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

/// What a call did: the record as it stands, the transitions it made and one line to print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Done {
    /// The dependency after the call.
    pub dep: Dep,
    /// The transitions made, oldest first.
    pub transitions: Vec<Transition>,
    /// One line for the caller.
    pub text: String,
}

impl Done {
    pub(crate) fn of(dep: Dep, transitions: Vec<Transition>) -> Done {
        let text = dep.line();
        Done {
            dep,
            transitions,
            text,
        }
    }
}

/// The project an actor (`project/T-n` or `project lead`) belongs to; `None` for the owner.
#[must_use]
pub fn project_of(actor: &str) -> Option<&str> {
    let project = actor.split(['/', ' ']).next().unwrap_or_default();
    (!project.is_empty() && !matches!(project, "human" | "owner")).then_some(project)
}
