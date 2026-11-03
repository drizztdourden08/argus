//! The links of an agent record, checked the way an op and the `coord agent` verbs both need them:
//! its items, its `master` flag and the records it waits on.

use argus_core::js;
use serde_json::{Value, json};

use super::target::item_arg;
use super::typed;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, truthy};

/// The most items one agent works on.
const ITEMS_MAX: usize = 10;
/// The most records one agent waits on.
const BLOCKED_BY_MAX: usize = 10;

fn list_of<'a>(v: &'a Value, what: &str) -> Result<&'a [Value], BoardError> {
    match v {
        Value::Null => Ok(&[]),
        Value::Array(list) => Ok(list),
        Value::String(_) => Ok(std::slice::from_ref(v)),
        _ => Err(refuse(format!("{what} must be a list of ids."))),
    }
}

fn push_new(out: &mut Vec<String>, id: String) {
    if !out.contains(&id) {
        out.push(id);
    }
}

/// The items an agent works on: each one a live item of the board, repeats dropped, at most ten.
///
/// # Errors
/// Refused with `no item "<id>".`, or when the value is not a list or is too long.
pub fn checked_items(board: &Board, v: &Value) -> Result<Vec<String>, BoardError> {
    let mut out = Vec::new();
    for entry in list_of(v, "items")? {
        push_new(
            &mut out,
            item_arg(board, &json!(js::trim(&display(entry))))?,
        );
    }
    if out.len() > ITEMS_MAX {
        return Err(refuse(format!(
            "{} items, an agent works on at most {ITEMS_MAX}.",
            out.len()
        )));
    }
    Ok(out)
}

fn agent_link(board: &Board, id: &str) -> Result<String, BoardError> {
    match doc(board, "agents", id) {
        Some(agent) if !truthy(agent, "deletedAt") => Ok(id.to_owned()),
        _ => Err(refuse(format!("{id} is not on this board."))),
    }
}

/// The records an agent waits on: live items, tests, to-dos, questions, decisions or other agents of
/// the board, never free text.
///
/// # Errors
/// Refused with the teaching message of a blocker, or when the list is too long.
pub fn checked_blocked_by(board: &Board, v: &Value) -> Result<Vec<String>, BoardError> {
    let mut out = Vec::new();
    for entry in list_of(v, "blockedBy")? {
        let id = js::trim(&display(entry)).to_owned();
        let link = if id.starts_with("A-") {
            agent_link(board, &id)?
        } else {
            let checked = typed::blocked_by_list(board, None, &json!([id]))?;
            display(&checked[0])
        };
        push_new(&mut out, link);
    }
    if out.len() > BLOCKED_BY_MAX {
        return Err(refuse(format!(
            "{} blockers, an agent waits on at most {BLOCKED_BY_MAX}.",
            out.len()
        )));
    }
    Ok(out)
}

/// The `items` field of an edit.
pub(crate) fn items(board: &Board, v: &Value) -> Result<Value, BoardError> {
    Ok(json!(checked_items(board, v)?))
}

/// The `blockedBy` field of an edit.
pub(crate) fn blocked_by(board: &Board, v: &Value) -> Result<Value, BoardError> {
    Ok(json!(checked_blocked_by(board, v)?))
}

/// The `master` field of an edit.
pub(crate) fn master(v: &Value) -> Result<Value, BoardError> {
    match v {
        Value::Bool(_) => Ok(v.clone()),
        _ => Err(refuse("master must be true or false.")),
    }
}
