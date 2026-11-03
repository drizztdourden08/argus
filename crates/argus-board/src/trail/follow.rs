//! Where a cause leads: to another item, which the walk follows, or to a record that ends the
//! branch.

use serde_json::Value;

use super::dep::{Far, far_end};
use super::{Boards, Edge};
use crate::blocked::{Cause, CauseKind};
use crate::json::doc;

/// A place on a board: a project and an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Spot {
    pub project: String,
    pub item: String,
}

impl Spot {
    pub(super) fn new(project: &str, item: &str) -> Spot {
        Spot {
            project: project.to_owned(),
            item: item.to_owned(),
        }
    }
}

/// What a cause leads to.
pub(super) enum Reach {
    /// An item the walk goes on from.
    Item(Spot),
    /// A record that ends the branch.
    Leaf {
        /// `question`, `decision`, `test`, `todo`, `request` or `legacy`.
        kind: &'static str,
        assignee: Option<String>,
    },
}

/// One step of the walk, from a node to one of its causes.
pub(super) struct Step {
    pub edge: Edge,
    pub label: String,
    pub reach: Reach,
}

/// The step a cause stands for; `None` for a request the ledger says is settled.
pub(super) fn step(boards: &Boards, from: &Spot, cause: &Cause) -> Option<Step> {
    match cause.kind {
        CauseKind::DepWaits | CauseKind::DepBlocked => across(boards, from, cause),
        CauseKind::Item => Some(local(boards, from, cause)),
        _ => Some(leaf(boards, from, cause)),
    }
}

fn edge(cause: &Cause, state: Option<String>, since: Option<String>) -> Edge {
    Edge {
        kind: cause.kind,
        id: cause.id.clone(),
        state,
        since,
    }
}

fn text(record: Option<&Value>, key: &str) -> Option<String> {
    record?[key].as_str().map(str::to_owned)
}

/// A `blockedBy` link to an item of the same board.
fn local(boards: &Boards, from: &Spot, cause: &Cause) -> Step {
    let item = boards
        .get(&from.project)
        .and_then(|b| doc(b, "items", &cause.id));
    Step {
        edge: edge(cause, text(item, "status"), text(item, "createdAt")),
        label: cause.label.clone(),
        reach: Reach::Item(Spot::new(&from.project, &cause.id)),
    }
}

/// A request to, or of, another board. An end that is not an item yet is a leaf.
fn across(boards: &Boards, from: &Spot, cause: &Cause) -> Option<Step> {
    let Far { spot, state, since } = far_end(boards, from, cause)?;
    let reach = match spot {
        Some(spot) => Reach::Item(spot),
        None => Reach::Leaf {
            kind: "request",
            assignee: cause.who.clone(),
        },
    };
    Some(Step {
        edge: edge(cause, state, since),
        label: cause.label.clone(),
        reach,
    })
}

/// The record a cause names, in the collection its id prefix says.
fn record<'a>(boards: &'a Boards, from: &Spot, cause: &Cause) -> Option<&'a Value> {
    let coll = match cause.id.split('-').next()? {
        "T" => "tests",
        "H" => "actions",
        "Q" | "D" => "entries",
        _ => return None,
    };
    doc(boards.get(&from.project)?, coll, &cause.id)
}

fn leaf_kind(kind: CauseKind) -> &'static str {
    match kind {
        CauseKind::Test => "test",
        CauseKind::Todo => "todo",
        CauseKind::Ask => "question",
        CauseKind::Decision => "decision",
        _ => "legacy",
    }
}

fn leaf(boards: &Boards, from: &Spot, cause: &Cause) -> Step {
    let record = record(boards, from, cause);
    Step {
        edge: edge(cause, text(record, "status"), text(record, "createdAt")),
        label: cause.label.clone(),
        reach: Reach::Leaf {
            kind: leaf_kind(cause.kind),
            assignee: cause.who.clone(),
        },
    }
}
