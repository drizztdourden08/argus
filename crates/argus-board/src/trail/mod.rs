//! The dependency trail of an item: the whole chain of what holds it back, across boards. The
//! walk follows every cause of the one blocked rule ([`crate::blocked::causes`]): the
//! `blockedBy` links of the same board, the requests to other boards through the ledger of the
//! portfolio board, and the open questions, decisions, tests and to-dos as leaves. An item that
//! waits on several things branches, so the trail is a tree. A node already on the path is shown
//! once, marked `cycle`, and not followed. Settled links are never followed. The walk is pure:
//! it reads the boards it is given and nothing else.

mod all;
mod count;
mod dep;
mod follow;
mod node;
// test-tag: critical
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use serde::Serialize;

use crate::Board;
use crate::blocked::{CauseKind, causes};

pub use all::{Summary, summaries};
use follow::{Reach, Spot, Step, step};

/// The boards the walk reads: project key to board, with the portfolio board (`_portfolio`)
/// for the ledger.
pub type Boards = BTreeMap<String, Board>;

/// The longest chain followed, in hops from the start.
pub const MAX_DEPTH: usize = 12;
/// The most nodes of one trail, so a tangle of shared causes cannot grow without bound.
const MAX_NODES: usize = 500;

/// The link that leads to a node from its parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edge {
    /// What kind of record holds the parent back.
    pub kind: CauseKind,
    /// The id of that record: the linked item, the `D-n` of a request or a decision, `Q-n`, `T-n`, `H-n`.
    pub id: String,
    /// Where the record stands: an item's status, a request's state, `open`, `pending`.
    pub state: Option<String>,
    /// When the record began, so a reader can show its age; absent when it carries no date.
    pub since: Option<String>,
}

/// One node of a trail: an item, or the leaf record that ends a branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Node {
    /// The board the node lives on.
    pub project: String,
    /// `I-n`, or the id of the leaf record.
    pub item: String,
    /// The title, or the text of a leaf.
    pub title: String,
    /// The item type; for a leaf, `question`, `decision`, `test`, `todo`, `request` or `legacy`.
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// The lane of an item (its `status`); for a leaf, where the record stands.
    pub lane: Option<String>,
    /// The stage of an item inside its lane.
    pub stage: Option<String>,
    /// Who holds the node.
    pub assignee: Option<String>,
    /// The link from the parent; absent on the start.
    pub edge: Option<Edge>,
    /// The node is already on the path: shown once, not followed.
    pub cycle: bool,
    /// What holds this node back.
    pub children: Vec<Node>,
}

/// The trail of one item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Trail {
    /// Nodes on the longest chain, the start included: the chain Bxel I-136, Tessera I-70,
    /// Brock I-105 is 3.
    pub depth: usize,
    /// Nodes not done, each counted once, the start included.
    pub open: usize,
    /// The start and, under it, everything it waits on.
    pub root: Node,
}

/// The trail of `item` on `project`.
#[must_use]
pub fn walk(boards: &Boards, project: &str, item: &str) -> Trail {
    let mut walker = Walker { boards, nodes: 0 };
    let start = Spot::new(project, item);
    let root = walker.visit(&start, (None, ""), &mut Vec::new());
    Trail {
        depth: count::depth(&root),
        open: count::open(&root),
        root,
    }
}

struct Walker<'a> {
    boards: &'a Boards,
    nodes: usize,
}

impl Walker<'_> {
    /// The node of an item and, unless it is a cycle or a limit is reached, its causes.
    fn visit(
        &mut self,
        at: &Spot,
        (edge, label): (Option<Edge>, &str),
        path: &mut Vec<Spot>,
    ) -> Node {
        self.nodes += 1;
        let mut node = node::of_item(self.boards, at, label, edge);
        if path.contains(at) {
            node.cycle = true;
        } else if path.len() < MAX_DEPTH && self.nodes < MAX_NODES {
            path.push(at.clone());
            node.children = self.children(at, path);
            path.pop();
        }
        node
    }

    fn children(&mut self, at: &Spot, path: &mut Vec<Spot>) -> Vec<Node> {
        let Some(board) = self.boards.get(&at.project) else {
            return Vec::new();
        };
        let steps: Vec<Step> = causes(board, &at.item)
            .iter()
            .filter_map(|cause| step(self.boards, at, cause))
            .collect();
        steps.into_iter().map(|s| self.child(at, s, path)).collect()
    }

    fn child(&mut self, at: &Spot, step: Step, path: &mut Vec<Spot>) -> Node {
        match step.reach {
            Reach::Item(spot) => self.visit(&spot, (Some(step.edge), &step.label), path),
            Reach::Leaf { kind, assignee } => {
                self.nodes += 1;
                node::leaf(at, kind, step.label, (step.edge, assignee))
            }
        }
    }
}
