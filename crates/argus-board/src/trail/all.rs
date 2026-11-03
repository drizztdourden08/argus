//! The trail numbers of every blocked item of a board.

use serde::Serialize;

use super::{Boards, walk};
use crate::blocked::causes;
use crate::json::{section, truthy};

/// The depth and open count of one blocked item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// The item, `I-n`.
    pub item: String,
    /// See [`super::Trail::depth`].
    pub depth: usize,
    /// See [`super::Trail::open`].
    pub open: usize,
}

/// The summary of each item of `project` that is not settled and has a cause, in board order.
#[must_use]
pub fn summaries(boards: &Boards, project: &str) -> Vec<Summary> {
    let Some(board) = boards.get(project) else {
        return Vec::new();
    };
    section(board, "items")
        .iter()
        .filter(|(_, item)| !settled(item))
        .filter(|(id, _)| !causes(board, id).is_empty())
        .map(|(id, _)| {
            let trail = walk(boards, project, id);
            Summary {
                item: id.clone(),
                depth: trail.depth,
                open: trail.open,
            }
        })
        .collect()
}

/// Done, dropped, deleted or archived: it holds nothing back to show.
fn settled(item: &serde_json::Value) -> bool {
    matches!(item["status"].as_str(), Some("done" | "dropped"))
        || truthy(item, "deletedAt")
        || truthy(item, "archivedAt")
}
