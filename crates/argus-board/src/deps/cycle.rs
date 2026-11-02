//! The loop walk. Nodes are `project/item`, edges are the dependencies in `opening`, `open` or
//! `blocked`. One request per link gives each node at most one outgoing edge, so the check is a
//! walk along the ledger, never a search.

use super::model::Dep;

/// The nodes from `start` to `goal` along live edges, `start` and `goal` included; `None` when
/// the walk ends or runs past the number of dependencies without reaching `goal`.
pub(crate) fn path(deps: &[Dep], start: &str, goal: &str) -> Option<Vec<String>> {
    let mut nodes = vec![start.to_owned()];
    for _ in 0..=deps.len() {
        let here = nodes.last()?;
        if here == goal {
            return Some(nodes);
        }
        let next = deps
            .iter()
            .find(|d| d.state.is_live() && d.to.item.is_some() && d.from.node() == *here)?;
        nodes.push(next.to.node());
    }
    None
}

/// The loop a new request from `requester` to `target` would close, as `a -> b -> a`; `None`
/// when it closes none.
pub(crate) fn closes(deps: &[Dep], requester: &str, target: &str) -> Option<String> {
    let walked = path(deps, target, requester)?;
    let mut nodes = vec![requester.to_owned()];
    nodes.extend(walked);
    Some(nodes.join(" -> "))
}
