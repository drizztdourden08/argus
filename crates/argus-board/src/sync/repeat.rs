//! The same post twice within two minutes (a double click) is one post: the copy is recorded as
//! a duplicate so the page stops showing it as pending, and nothing else. A failed op posted
//! again is a retry, and so is the same op after something else happened to its target.

use argus_core::clock::parse_ms;
use serde_json::{Value, json};

use super::is_op_kind;
use crate::Board;
use crate::json::{display, section};

const REPEAT_MS: i64 = 2 * 60_000;
const RETRIED: [&str; 4] = ["duplicate", "cancelled", "refused", "conflict"];

/// Only what the page sends: never outcome, status, appliedAt, by, current or results, which
/// applying writes onto the stored entry afterwards. A `result` reply is the one kind whose
/// page-sent outcome (pass or fail) is part of what was posted.
fn signature(e: &Value) -> String {
    let get = |key: &str| e[key].clone();
    let or = |key: &str, default: Value| match &e[key] {
        Value::Null => default,
        v => v.clone(),
    };
    let outcome = if e["kind"].as_str().is_some_and(is_op_kind) {
        Value::Null
    } else {
        get("outcome")
    };
    let text = or("text", json!(""));
    let parts = json!([
        get("kind"),
        get("replyTo"),
        text,
        outcome,
        get("target"),
        get("set"),
        get("to"),
        get("do"),
        get("n"),
        get("m"),
        get("ops"),
        get("criteria"),
        get("undoes"),
        get("reason"),
        get("confirm"),
        get("expect"),
        or("force", json!(false)),
        get("handed"),
        get("avatar"),
        get("icon"),
        get("kanban"),
        get("titleMax"),
    ]);
    parts.to_string()
}

/// A build or avatar entry has no target: hand, unhand and hand again are all about the one handed
/// build, and every avatar or icon post is about the one avatar or icon of the board.
fn subject_of(e: &Value) -> Option<String> {
    let named = [&e["target"]["id"], &e["replyTo"]]
        .into_iter()
        .find(|v| !v.is_null());
    match named {
        Some(v) => Some(display(v)),
        None => match e["kind"].as_str() {
            Some(kind @ ("build" | "avatar" | "icon" | "kanban-config" | "title-max")) => {
                Some(kind.to_owned())
            }
            _ => None,
        },
    }
}

fn created_at(e: &Value) -> Option<&str> {
    e["createdAt"].as_str()
}

fn within(x: &Value, e: &Value) -> bool {
    let at = |v: &Value| created_at(v).and_then(parse_ms);
    match (at(x), at(e)) {
        (Some(a), Some(b)) => (a - b).abs() < REPEAT_MS,
        _ => false,
    }
}

/// Whether an owner entry repeats an earlier one of the owner's within two minutes.
pub(crate) fn is_repeat(board: &Board, e: &Value) -> bool {
    let sig = signature(e);
    let humans: Vec<&Value> = section(board, "entries")
        .values()
        .filter(|x| {
            x["author"] == "human"
                && x["id"] != e["id"]
                && !x["status"].as_str().is_some_and(|s| RETRIED.contains(&s))
        })
        .collect();
    let later_on_same_subject = |x: &Value| {
        let subject = subject_of(x);
        humans.iter().any(|y| {
            let other = subject_of(y);
            other.as_deref().is_some_and(|s| !s.is_empty())
                && other == subject
                && matches!((created_at(y), created_at(x), created_at(e)), (Some(yc), Some(xc), Some(ec)) if yc > xc && yc <= ec)
        })
    };
    humans
        .iter()
        .any(|x| signature(x) == sig && within(x, e) && !later_on_same_subject(x))
}
