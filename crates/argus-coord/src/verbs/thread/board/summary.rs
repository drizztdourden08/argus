//! The summary of a board: what `board show` prints as text and, with `--json`, as JSON.
//! Archived and deleted records are out of every summary; the board file keeps them.

use argus_board::Board;
use argus_board::agents::live_agent_ids;
use argus_board::json::{section, truthy};
use serde_json::{Map, Value, json};

use crate::verbs::thread::docs::{field, id_of, list, object, or_null, visible};

fn thread_row(board: &Board, t: &Value) -> Value {
    let agents = live_agent_ids(board, Some(&id_of(t))).len();
    object(vec![
        ("id", field(t, "id")),
        ("status", field(t, "status")),
        ("subject", field(t, "subject")),
        ("note", field(t, "note")),
        ("agents", Some(json!(agents))),
        ("flags", field(t, "flags")),
        ("lastSeen", field(t, "lastSeen")),
        ("session", field(t, "session")),
    ])
}

fn item_row(i: &Value) -> Value {
    let criteria = list(&i["ac"]);
    let done = criteria.iter().filter(|c| truthy(c, "done")).count();
    object(vec![
        ("id", field(i, "id")),
        ("status", field(i, "status")),
        ("thread", field(i, "thread")),
        ("subject", field(i, "subject")),
        ("title", field(i, "title")),
        ("ac", Some(json!(format!("{done}/{}", criteria.len())))),
    ])
}

fn test_row(t: &Value) -> Value {
    object(vec![
        ("id", field(t, "id")),
        ("status", field(t, "status")),
        ("mode", field(t, "mode")),
        ("build", Some(or_null(field(t, "build")))),
        ("item", field(t, "item")),
        ("title", field(t, "title")),
    ])
}

fn lock_row(l: &Value) -> Value {
    object(vec![
        ("resource", field(l, "resource")),
        ("holder", field(l, "holder")),
        ("queue", field(l, "queue")),
    ])
}

fn ids_where(docs: &[&Value], kind: Option<&str>, status: &str) -> Vec<Value> {
    docs.iter()
        .filter(|e| kind.is_none_or(|k| e["kind"] == k) && e["status"] == status)
        .filter_map(|e| field(e, "id"))
        .collect()
}

fn waiting(board: &Board) -> Map<String, Value> {
    section(board, "inbox")
        .iter()
        .map(|(k, v)| (k.clone(), list(v).len()))
        .filter(|(_, n)| *n > 0)
        .map(|(k, n)| (k, json!(n)))
        .collect()
}

/// The summary of a board.
#[must_use]
pub fn summary(board: &Board) -> Value {
    let entries = visible(board, "entries");
    let actions = visible(board, "actions");
    let rows = |rows: Vec<Value>| Some(Value::Array(rows));
    object(vec![
        ("project", field(board, "project")),
        ("updatedAt", field(board, "updatedAt")),
        (
            "threads",
            rows(
                visible(board, "threads")
                    .into_iter()
                    .map(|t| thread_row(board, t))
                    .collect(),
            ),
        ),
        (
            "items",
            rows(visible(board, "items").into_iter().map(item_row).collect()),
        ),
        (
            "openQuestions",
            rows(ids_where(&entries, Some("question"), "open")),
        ),
        (
            "pendingDecisions",
            rows(ids_where(&entries, Some("decision"), "pending")),
        ),
        ("handedBuild", Some(or_null(field(board, "handedBuild")))),
        (
            "tests",
            rows(visible(board, "tests").into_iter().map(test_row).collect()),
        ),
        ("pendingActions", rows(ids_where(&actions, None, "pending"))),
        (
            "locks",
            rows(section(board, "locks").values().map(lock_row).collect()),
        ),
        ("inbox", Some(Value::Object(waiting(board)))),
        ("dirty", Some(json!(section(board, "dirty").len()))),
    ])
}
