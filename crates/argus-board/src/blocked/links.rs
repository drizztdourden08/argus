//! The causes that come from records: the ids in `blockedBy`, and the open asks of the item.

use serde_json::Value;

use super::{Cause, CauseKind, gone};
use crate::Board;
use crate::json::doc;

/// The causes of the ids in the `blockedBy` of an item. A link to the item itself, to a record
/// that is missing, deleted or archived, or to one that is settled, is no cause.
pub(super) fn linked(board: &Board, item_id: &str, item: &Value) -> Vec<Cause> {
    ids_of(&item["blockedBy"])
        .into_iter()
        .filter(|id| id != item_id)
        .filter_map(|id| cause_of(board, &id))
        .collect()
}

/// The ids of a `blockedBy` field: a list, or a lone id.
fn ids_of(field: &Value) -> Vec<String> {
    match field {
        Value::Array(list) => list
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        Value::String(id) => vec![id.clone()],
        _ => Vec::new(),
    }
}

/// The cause a linked id stands for, when it still holds the item back.
fn cause_of(board: &Board, id: &str) -> Option<Cause> {
    match id.split('-').next()? {
        "I" => check(board, "items", id, item_cause),
        "T" => check(board, "tests", id, test_cause),
        "H" => check(board, "actions", id, todo_cause),
        "Q" | "D" => check(board, "entries", id, open_ask),
        _ => None,
    }
}

/// Apply a rule to a record that is still there.
fn check(board: &Board, coll: &str, id: &str, rule: Rule) -> Option<Cause> {
    let record = doc(board, coll, id).filter(|r| !gone(r))?;
    rule(id, record)
}

/// What decides whether a record still holds an item back.
type Rule = fn(&str, &Value) -> Option<Cause>;

fn item_cause(id: &str, item: &Value) -> Option<Cause> {
    let settled = matches!(item["status"].as_str(), Some("done" | "dropped"));
    (!settled).then(|| cause(CauseKind::Item, id, item, None))
}

fn test_cause(id: &str, test: &Value) -> Option<Cause> {
    (test["status"].as_str() != Some("pass")).then(|| cause(CauseKind::Test, id, test, None))
}

fn todo_cause(id: &str, todo: &Value) -> Option<Cause> {
    let who = todo["assignee"]
        .as_str()
        .or_else(|| todo["thread"].as_str())
        .filter(|w| !w.is_empty());
    (todo["status"].as_str() == Some("pending")).then(|| cause(CauseKind::Todo, id, todo, who))
}

/// An open question or a pending decision; any other entry is no cause.
pub(super) fn open_ask(id: &str, entry: &Value) -> Option<Cause> {
    let kind = match (entry["kind"].as_str()?, entry["status"].as_str()?) {
        ("question", "open") => CauseKind::Ask,
        ("decision", "pending") => CauseKind::Decision,
        _ => return None,
    };
    Some(cause(kind, id, entry, None))
}

/// A cause labelled by the title of the record, else its text, else its id.
fn cause(kind: CauseKind, id: &str, record: &Value, who: Option<&str>) -> Cause {
    let label = ["title", "text", "name"]
        .iter()
        .find_map(|key| record[*key].as_str().filter(|t| !t.is_empty()))
        .unwrap_or(id);
    Cause {
        kind,
        id: id.to_owned(),
        label: label.to_owned(),
        who: who.map(str::to_owned),
    }
}
