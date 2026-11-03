//! The tree of items: who sits under whom, and which parent is allowed.

use argus_core::js;
use serde_json::Value;

use super::kind::{DEFAULT_TYPE, type_id};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc, find_ids, truthy};
use crate::ops::target::item_arg;
use crate::registry::registry;

/// The most levels an ancestor chain may have before it is not read as a tree.
const SPAN: usize = 8;

/// The ids of the live items whose parent is `id`.
pub(crate) fn children_of(board: &Board, id: &str) -> Vec<String> {
    find_ids(board, "items", |i| {
        i["parent"].as_str() == Some(id) && !truthy(i, "deletedAt")
    })
}

/// The type id of an item of the board.
pub(crate) fn type_in(board: &Board, id: &str) -> String {
    doc(board, "items", id)
        .map_or(DEFAULT_TYPE, type_id)
        .to_owned()
}

/// Whether `id` is above `of`, going up the parents.
fn is_above(board: &Board, id: &str, of: &str) -> bool {
    let mut at = of.to_owned();
    for _ in 0..SPAN {
        let up = doc(board, "items", &at).and_then(|i| i["parent"].as_str().map(str::to_owned));
        match up {
            Some(up) if up == id => return true,
            Some(up) => at = up,
            None => return false,
        }
    }
    true
}

fn no_cycle(board: &Board, id: &str, parent: &str) -> Result<(), BoardError> {
    if id == parent {
        return Err(refuse(format!("{id} cannot be its own parent.")));
    }
    if is_above(board, id, parent) {
        return Err(refuse(format!(
            "{parent} sits under {id}; making it the parent of {id} would be a cycle."
        )));
    }
    Ok(())
}

/// Refuses when an item of type `child` may not sit under an item of type `parent_type`.
pub(crate) fn allowed(child: &str, parent_type: &str, parent: &str) -> Result<(), BoardError> {
    if registry().allows_parent(child, parent_type) {
        return Ok(());
    }
    let parents = registry()
        .item_type(child)
        .map(|t| t.parents.join(", "))
        .unwrap_or_default();
    let hint = if parents.is_empty() {
        format!("a {child} is always a root")
    } else {
        format!("a {child} sits under {parents}")
    };
    Err(refuse(format!("{parent} is a {parent_type}; {hint}.")))
}

/// The parent an argument names: `null` for none, else a live item of this board that is not the
/// item itself or one of its descendants, of a type that may hold `child`. `id` is `None` for an
/// item not created yet.
pub(crate) fn check_parent(
    board: &Board,
    id: Option<&str>,
    child: &str,
    v: &Value,
) -> Result<Value, BoardError> {
    if !js::truthy(v) || v.as_str() == Some("none") {
        return Ok(Value::Null);
    }
    let parent = item_arg(board, v)?;
    if let Some(id) = id {
        no_cycle(board, id, &parent)?;
    }
    allowed(child, &type_in(board, &parent), &parent)?;
    Ok(Value::String(parent))
}
