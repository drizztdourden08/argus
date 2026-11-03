//! The nodes of a trail: an item as its board holds it, or the leaf record that ends a branch.

use serde_json::Value;

use super::follow::Spot;
use super::{Boards, Edge, Node};
use crate::json::doc;

fn text(item: Option<&Value>, key: &str) -> Option<String> {
    item?[key]
        .as_str()
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

/// The node of an item. The title falls back to `label` (the title the link carried) when the
/// board or the item is missing.
pub(super) fn of_item(boards: &Boards, at: &Spot, label: &str, edge: Option<Edge>) -> Node {
    let item = boards
        .get(&at.project)
        .and_then(|board| doc(board, "items", &at.item));
    Node {
        project: at.project.clone(),
        item: at.item.clone(),
        title: text(item, "title").unwrap_or_else(|| label.to_owned()),
        kind: text(item, "type"),
        lane: text(item, "status"),
        stage: text(item, "stage"),
        assignee: text(item, "assignee").or_else(|| text(item, "thread")),
        edge,
        cycle: false,
        children: Vec::new(),
    }
}

/// The node of a record that ends a branch: its lane is where the record stands.
pub(super) fn leaf(
    at: &Spot,
    kind: &str,
    label: String,
    (edge, assignee): (Edge, Option<String>),
) -> Node {
    Node {
        project: at.project.clone(),
        item: edge.id.clone(),
        title: label,
        kind: Some(kind.to_owned()),
        lane: edge.state.clone(),
        stage: None,
        assignee,
        edge: Some(edge),
        cycle: false,
        children: Vec::new(),
    }
}
