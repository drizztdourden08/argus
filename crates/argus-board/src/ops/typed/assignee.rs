//! Who holds the ball: the assignee of an item, and of a to-do.

use argus_core::js;
use serde_json::{Map, Value};

use super::known::Known;
use crate::Board;
use crate::content::check as content_check;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc};

/// The assignee of an item: a thread of the board or the owner. A blank value is `null`.
pub(crate) fn check(board: &Board, v: &Value) -> Result<Value, BoardError> {
    content_check("assignee", v, &Known::of(board, Vec::new()).context())
}

/// The assignee of a to-do: a thread of the board, the owner, or `lead:<key>`. A blank value is
/// `null`.
pub(crate) fn todo(board: &Board, v: &Value) -> Result<Value, BoardError> {
    content_check("todoAssignee", v, &Known::of(board, Vec::new()).context())
}

/// The thread an assignee names, when it names one: the owner and the leads are no thread.
pub(crate) fn thread_of(board: &Board, assignee: &Value) -> Option<String> {
    assignee
        .as_str()
        .filter(|a| *a != "owner" && doc(board, "threads", a).is_some())
        .map(str::to_owned)
}

/// The thread a new item belongs to when its assignee is a thread: the assignee and the thread
/// move together, so both may be named only when they are the same.
pub(crate) fn hint(board: &Board, set: &Map<String, Value>) -> Result<Option<Value>, BoardError> {
    let Some(v) = set.get("assignee").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let assignee = check(board, v)?;
    let Some(thread) = thread_of(board, &assignee) else {
        return Ok(None);
    };
    match set.get("thread").filter(|t| js::truthy(t)) {
        Some(named) if display(named) != thread => Err(refuse(format!(
            "thread {} and assignee {thread} differ; name one thread.",
            display(named)
        ))),
        _ => Ok(Some(Value::String(thread))),
    }
}

/// The assignee of a new to-do, written when one is given.
pub(crate) fn stamp_todo(
    board: &Board,
    set: &Map<String, Value>,
    fields: &mut Map<String, Value>,
) -> Result<(), BoardError> {
    if let Some(v) = set.get("assignee") {
        let who = todo(board, v)?;
        if !who.is_null() {
            fields.insert("assignee".to_owned(), who);
        }
    }
    Ok(())
}
