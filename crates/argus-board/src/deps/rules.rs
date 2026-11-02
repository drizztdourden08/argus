//! The rules of a new request, checked against the ledger under its lock: one request per link,
//! one request per assignee item, and no loop.

use super::cycle::closes;
use super::error::{DepError, already_requested, already_waits, closes_loop};
use super::model::{Dep, node};

/// The live dependency a requester item already has, if any.
pub(crate) fn waiting_from<'a>(deps: &'a [Dep], project: &str, item: &str) -> Option<&'a Dep> {
    deps.iter().find(|d| {
        d.state.is_live() && d.from.project == project && d.from.item.as_deref() == Some(item)
    })
}

/// The live dependency that already requests an assignee item, if any.
pub(crate) fn requesting<'a>(deps: &'a [Dep], project: &str, item: &str) -> Option<&'a Dep> {
    deps.iter().find(|d| {
        d.state.is_live() && d.to.project == project && d.to.item.as_deref() == Some(item)
    })
}

/// Refuse a request from `from` that would be a second one on its link, land on an item that
/// is already requested, or close a loop. `onto` is the existing assignee item, when named.
pub(crate) fn check_new(
    deps: &[Dep],
    from: (&str, &str),
    onto: Option<(&str, &str)>,
) -> Result<(), DepError> {
    if let Some(d) = waiting_from(deps, from.0, from.1) {
        return Err(already_waits(&node(from.0, from.1), &d.assignee(), &d.id));
    }
    let Some((project, item)) = onto else {
        return Ok(());
    };
    if let Some(d) = requesting(deps, project, item) {
        return Err(already_requested(
            &node(project, item),
            &d.requester(),
            &d.id,
        ));
    }
    match closes(deps, &node(from.0, from.1), &node(project, item)) {
        Some(path) => Err(closes_loop(&path)),
        None => Ok(()),
    }
}
