//! The record an op points at, and the checks of the arguments that name other records.

use argus_core::js;
use serde_json::Value;

use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, shown, truthy};

pub(crate) const ITEM_STATUSES: [&str; 5] = ["todo", "doing", "review", "done", "dropped"];
const TARGETS: [&str; 6] = ["items", "tests", "entries", "actions", "threads", "agents"];

/// The record an op points at, as `(collection, id)`. A purged one is `gone`, a trashed one
/// `in trash`.
pub(crate) fn target_of(
    board: &Board,
    op: &Value,
    allow_trash: bool,
) -> Result<(String, String), BoardError> {
    let target = &op["target"];
    let coll = target
        .get("c")
        .and_then(Value::as_str)
        .filter(|c| TARGETS.contains(c));
    let Some(coll) = coll else {
        return Err(refuse(format!(
            "cannot target \"{}\".",
            shown(target.get("c"))
        )));
    };
    let id = shown(target.get("id"));
    let Some(found) = doc(board, coll, &id) else {
        return Err(refuse("gone"));
    };
    if truthy(found, "deletedAt") && !allow_trash {
        return Err(refuse("in trash"));
    }
    Ok((coll.to_owned(), id))
}

/// A thread named by an op argument: its id, refused when it does not exist.
pub(crate) fn thread_arg(board: &Board, id: &Value) -> Result<String, BoardError> {
    let key = display(id);
    if doc(board, "threads", &key).is_none() {
        return Err(refuse(format!("no thread \"{key}\".")));
    }
    Ok(key)
}

/// An item named by an op argument: its id, refused when it does not exist or is in the trash.
pub(crate) fn item_arg(board: &Board, id: &Value) -> Result<String, BoardError> {
    let key = display(id);
    match doc(board, "items", &key) {
        Some(item) if !truthy(item, "deletedAt") => Ok(key),
        _ => Err(refuse(format!("no item \"{key}\"."))),
    }
}

/// `v` when it is one of the words, else a refusal naming them.
pub(crate) fn one_of(words: &[&str], v: &Value, what: &str) -> Result<Value, BoardError> {
    if v.as_str().is_some_and(|s| words.contains(&s)) {
        Ok(v.clone())
    } else {
        Err(refuse(format!(
            "{what} must be one of {}.",
            words.join(", ")
        )))
    }
}

/// The neighbour id of an order argument: `None` for anything falsy.
pub(crate) fn id_arg(v: &Value) -> Option<String> {
    js::truthy(v).then(|| display(v))
}
