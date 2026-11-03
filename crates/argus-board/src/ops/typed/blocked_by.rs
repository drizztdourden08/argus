//! The links that block an item: ids of this board, never a text.

use argus_core::js;
use serde_json::{Value, json};

use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc, truthy};

/// The collection an id lives in, and the entry kind when it is an entry.
fn home(link: &str) -> Result<(&'static str, Option<&'static str>), BoardError> {
    let shaped = link
        .split_once('-')
        .filter(|(_, n)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
    match shaped.map(|(prefix, _)| prefix) {
        Some("I") => Ok(("items", None)),
        Some("T") => Ok(("tests", None)),
        Some("H") => Ok(("actions", None)),
        Some("Q") => Ok(("entries", Some("question"))),
        Some("D") => Ok(("entries", Some("decision"))),
        Some("N") => Err(refuse(
            "a note never blocks; link an item, test, to-do, question or decision.",
        )),
        _ => Err(refuse(format!(
            "\"{link}\" is not an id; a blocker is an I-n, T-n, H-n, Q-n or D-n of this board."
        ))),
    }
}

/// The ids an item is blocked by.
fn links_of(board: &Board, at: &str) -> Vec<String> {
    doc(board, "items", at)
        .and_then(|i| i["blockedBy"].as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(str::to_owned))
        .collect()
}

/// Whether `target` is blocked by `from`, directly or through the items `from` waits on.
fn reaches(board: &Board, from: &str, target: &str) -> bool {
    let mut seen: Vec<String> = Vec::new();
    let mut todo: Vec<String> = vec![from.to_owned()];
    while let Some(at) = todo.pop() {
        if at == target {
            return true;
        }
        if seen.contains(&at) {
            continue;
        }
        todo.extend(links_of(board, &at));
        seen.push(at);
    }
    false
}

fn one(board: &Board, id: Option<&str>, entry: &Value) -> Result<String, BoardError> {
    let link = js::trim(entry.as_str().unwrap_or_default()).to_owned();
    if id == Some(link.as_str()) {
        return Err(refuse("an item cannot block itself."));
    }
    let (coll, kind) = home(&link)?;
    let live = doc(board, coll, &link)
        .is_some_and(|d| !truthy(d, "deletedAt") && kind.is_none_or(|k| d["kind"] == k));
    if !live {
        return Err(refuse(format!("{link} is not on this board.")));
    }
    if coll == "items" && id.is_some_and(|me| reaches(board, &link, me)) {
        return Err(refuse(format!(
            "{link} already waits on this item; blocking on it would be a cycle."
        )));
    }
    Ok(link)
}

/// The list checked: every id is live on the board, none is the item itself, and no link closes a
/// cycle. A blank value is an empty list. `id` is `None` for an item not created yet.
pub(crate) fn check(board: &Board, id: Option<&str>, v: &Value) -> Result<Value, BoardError> {
    let list = match v {
        Value::Null => return Ok(json!([])),
        Value::Array(list) => list,
        _ => return Err(refuse("blockedBy must be a list of ids.")),
    };
    let mut out: Vec<String> = Vec::new();
    for entry in list {
        let link = one(board, id, entry)?;
        if !out.contains(&link) {
            out.push(link);
        }
    }
    Ok(json!(out))
}
