//! What serves an item.

use super::state::{for_of, is_retired};
use crate::Board;
use crate::json::section;

/// The builds and worktrees linked to one item.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Served {
    /// The names of the live builds that serve it.
    pub builds: Vec<String>,
    /// The names of the closed builds that served it.
    pub closed: Vec<String>,
    /// The ids of the worktrees linked to it (`board.worktreeLinks`).
    pub worktree_links: Vec<String>,
}

/// Everything that serves `item`, in board order.
#[must_use]
pub fn served_by(board: &Board, item: &str) -> Served {
    let mut served = Served::default();
    for (name, rec) in section(board, "launchers") {
        if for_of(rec) != Some(item) {
            continue;
        }
        if is_retired(rec) {
            served.closed.push(name.clone());
        } else {
            served.builds.push(name.clone());
        }
    }
    served.worktree_links = section(board, "worktreeLinks")
        .iter()
        .filter(|(_, link)| link["item"].as_str() == Some(item))
        .map(|(id, _)| id.clone())
        .collect();
    served
}
