//! The link rules of a build's `for`: what a build may serve.

use serde_json::Value;

use super::state::closure;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::doc;
use crate::registry::registry;

/// The type of an item record: its own, else `task`.
fn type_of(item: &Value) -> &str {
    item["type"].as_str().unwrap_or("task")
}

fn served_list() -> String {
    let all: Vec<&str> = registry()
        .types
        .iter()
        .filter(|t| !t.serves.is_empty())
        .flat_map(|t| t.serves.iter().map(String::as_str))
        .collect();
    all.join(", ")
}

/// Whether `text` is an `I-n` id.
fn is_item_id(text: &str) -> bool {
    text.strip_prefix("I-")
        .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// The refusal for a target that is not the id of an item.
fn not_an_item(target: &str) -> BoardError {
    let other_kind = target
        .split_once('-')
        .is_some_and(|(prefix, n)| prefix.len() == 1 && n.chars().all(|c| c.is_ascii_digit()));
    if other_kind {
        refuse(format!(
            "{target} is not a work item: a build serves a work item (I-n), never a test case, \
             a question, a decision, a to-do or a note."
        ))
    } else if target.contains(['/', ':']) {
        refuse(format!(
            "a build serves an item of its own board, not {target}."
        ))
    } else {
        refuse(format!(
            "--for takes an item id like I-12, not \"{target}\"."
        ))
    }
}

/// The item a build may serve, or the sentence that says why not. The master build may go
/// without an item and may serve a closed one; every other build needs an open item of this
/// board whose type the registry lets a build serve.
///
/// # Errors
/// Refused with the reason: no link, not an item id, not on this board, a type a build cannot
/// serve, or an item that is done, dropped or in the trash.
pub fn check_for(board: &Board, master: bool, target: Option<&str>) -> Result<(), BoardError> {
    let Some(target) = target.map(str::trim).filter(|t| !t.is_empty()) else {
        return if master {
            Ok(())
        } else {
            Err(refuse(
                "a build needs the item it serves: pass --for I-n. Only the master build goes without one.",
            ))
        };
    };
    if !is_item_id(target) {
        return Err(not_an_item(target));
    }
    let Some(item) = doc(board, "items", target) else {
        return Err(refuse(format!(
            "no item {target} on this board: a build serves an item of its own board."
        )));
    };
    if !registry().can_serve(type_of(item)) {
        return Err(refuse(format!(
            "a build cannot serve {target}, a {}: it serves {}.",
            type_of(item),
            served_list()
        )));
    }
    match closure(item) {
        Some(why) if !master => Err(refuse(format!(
            "{target} is {why}: a build cannot serve a closed item. Link it to an open item, or reopen {target}."
        ))),
        _ => Ok(()),
    }
}
