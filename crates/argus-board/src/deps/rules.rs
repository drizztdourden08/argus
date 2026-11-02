//! The rules of a new request, checked against the ledger under its lock: one request per link
//! (a requester item and an assignee item) and no loop. An item may wait on several others and
//! may be requested by several boards.

use super::cycle::closes;
use super::error::{DepError, already_linked, closes_loop};
use super::model::{Dep, node};

/// The live dependency that already links a requester item to an assignee item, if any.
pub(crate) fn linking<'a>(
    deps: &'a [Dep],
    from: (&str, &str),
    to: (&str, &str),
) -> Option<&'a Dep> {
    deps.iter().find(|d| {
        d.state.is_live()
            && (d.from.project.as_str(), d.from.item.as_deref()) == (from.0, Some(from.1))
            && (d.to.project.as_str(), d.to.item.as_deref()) == (to.0, Some(to.1))
    })
}

/// Refuse a request from `from` that would repeat a live link or close a loop. `onto` is the
/// existing assignee item, when named.
pub(crate) fn check_new(
    deps: &[Dep],
    from: (&str, &str),
    onto: Option<(&str, &str)>,
) -> Result<(), DepError> {
    let Some((project, item)) = onto else {
        return Ok(());
    };
    if let Some(d) = linking(deps, from, (project, item)) {
        return Err(already_linked(
            &node(from.0, from.1),
            &node(project, item),
            &d.id,
        ));
    }
    match closes(deps, &node(from.0, from.1), &node(project, item)) {
        Some(path) => Err(closes_loop(&path)),
        None => Ok(()),
    }
}
