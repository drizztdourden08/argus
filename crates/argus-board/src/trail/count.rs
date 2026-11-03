//! The two numbers of a trail: how deep it goes and how much of it is still open.

use std::collections::HashSet;

use super::Node;

/// Nodes on the longest chain from `node` down, itself included.
pub(super) fn depth(node: &Node) -> usize {
    1 + node.children.iter().map(depth).max().unwrap_or(0)
}

/// Distinct nodes under and including `node` that are not done. A node that two branches share,
/// or a cycle mark, is the same node seen again, so it is counted once.
pub(super) fn open(node: &Node) -> usize {
    let mut seen = HashSet::new();
    collect(node, &mut seen);
    seen.len()
}

fn collect<'a>(node: &'a Node, seen: &mut HashSet<(&'a str, &'a str)>) {
    if !finished(node) {
        seen.insert((&node.project, &node.item));
    }
    for child in &node.children {
        collect(child, seen);
    }
}

fn finished(node: &Node) -> bool {
    matches!(node.lane.as_deref(), Some("done" | "dropped"))
}
