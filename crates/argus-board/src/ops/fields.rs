//! The editable fields of each collection, each with its check. A value that fails (over its
//! limit, empty where text is required, not in the list) refuses the whole op.

use argus_core::js;
use serde_json::Value;

use super::target::{ITEM_STATUSES, item_arg, one_of, record_arg};
use super::typed;
use super::who::Who;
use crate::Board;
use crate::agents::is_live;
use crate::error::{BoardError, refuse};
use crate::json::{display, truthy};
use crate::limits::{
    OWNER_TEXT_MAX, limit, optional_text, require_text, require_text_max, text_list,
};

const ACTION_STATUSES: [&str; 2] = ["pending", "done"];
const TEST_MODES: [&str; 2] = ["auto", "manual"];
const OWNER_AGENT_STATUS: [&str; 2] = ["working", "stop-requested"];
const THREAD_AGENT_STATUS: [&str; 2] = ["working", "waiting"];

/// What a field check may look at.
pub(crate) struct EditCtx<'a> {
    pub board: &'a Board,
    pub doc: &'a Value,
    pub who: &'a Who,
}

/// Whether a field of a collection can be edited. An item's own fields are edited one by one as
/// `fields.<id>`.
pub(crate) fn editable(coll: &str, field: &str) -> bool {
    matches!(
        (coll, field),
        (
            "items",
            "title"
                | "description"
                | "status"
                | "blocker"
                | "type"
                | "parent"
                | "assignee"
                | "stage"
                | "fields"
                | "blockedBy"
                | "images"
        ) | ("tests", "title" | "covers" | "mode" | "build")
            | ("actions", "text" | "item" | "status" | "assignee")
            | ("entries", "text" | "options")
            | ("threads", "subject" | "brief" | "rank")
            | (
                "agents",
                "note" | "task" | "item" | "branch" | "worktree" | "status"
            )
    ) || (coll == "items" && field.starts_with("fields."))
}

/// Check one value for a field the collection lets through [`editable`].
pub(crate) fn check(coll: &str, field: &str, v: &Value, cx: &EditCtx) -> Result<Value, BoardError> {
    match (coll, field) {
        ("items" | "tests", "title") => text(field, v),
        ("items", "description") => Ok(Value::String(
            optional_text("description", v)?.unwrap_or_default(),
        )),
        ("items", "status") => one_of(&ITEM_STATUSES, v, "status"),
        ("items", "blocker") => optional("note", v),
        ("tests", "mode") => one_of(&TEST_MODES, v, "mode"),
        ("actions", "text") => text("action", v),
        ("actions", "status") => one_of(&ACTION_STATUSES, v, "status"),
        ("entries", "text") => entry_text(v, cx),
        ("entries", "options") => entry_options(v, cx),
        ("threads", "rank") => thread_rank(v),
        ("agents", "status") => agent_status(v, cx),
        ("agents", "note") => optional("agentnote", v),
        ("agents", "task") => text("task", v),
        ("tests", "covers" | "build")
        | ("threads", "subject" | "brief")
        | ("agents", "branch" | "worktree") => optional(field, v),
        ("actions", "assignee") => typed::todo_assignee(cx.board, v),
        ("actions", "item") => record_or_none(v, cx),
        ("agents", "item") => item_or_none(v, cx),
        _ => Err(refuse(format!("{field} cannot be edited on {coll}."))),
    }
}

fn text(kind: &str, v: &Value) -> Result<Value, BoardError> {
    require_text(kind, v).map(Value::String)
}

fn optional(kind: &str, v: &Value) -> Result<Value, BoardError> {
    Ok(optional_text(kind, v)?.map_or(Value::Null, Value::String))
}

fn item_or_none(v: &Value, cx: &EditCtx) -> Result<Value, BoardError> {
    if js::truthy(v) {
        Ok(Value::String(item_arg(cx.board, v)?))
    } else {
        Ok(Value::Null)
    }
}

fn record_or_none(v: &Value, cx: &EditCtx) -> Result<Value, BoardError> {
    if js::truthy(v) {
        Ok(Value::String(record_arg(cx.board, v)?))
    } else {
        Ok(Value::Null)
    }
}

fn entry_text(v: &Value, cx: &EditCtx) -> Result<Value, BoardError> {
    let human = cx.who.is_human();
    if human && cx.doc["author"] != "human" {
        return Err(refuse(
            "the owner never edits a thread's own wording; close, archive or delete it instead.",
        ));
    }
    if cx.doc["kind"] == "note" {
        return typed::note_text(cx.board, cx.doc, v);
    }
    let kind = cx.doc["kind"]
        .as_str()
        .filter(|k| limit(k).is_some())
        .unwrap_or("message");
    let checked = if human {
        require_text_max(kind, v, OWNER_TEXT_MAX)
    } else {
        require_text(kind, v)
    };
    checked.map(Value::String)
}

fn entry_options(v: &Value, cx: &EditCtx) -> Result<Value, BoardError> {
    if cx.doc["kind"] != "question" && !truthy(cx.doc, "options") {
        return Err(refuse("only a question carries options."));
    }
    let options = text_list("option", v, 6)?;
    Ok(Value::Array(
        options.into_iter().map(Value::String).collect(),
    ))
}

fn thread_rank(v: &Value) -> Result<Value, BoardError> {
    if v.is_number() {
        Ok(v.clone())
    } else {
        Err(refuse("rank must be a number."))
    }
}

fn agent_status(v: &Value, cx: &EditCtx) -> Result<Value, BoardError> {
    let allowed: &[&str] = if cx.who.is_human() {
        &OWNER_AGENT_STATUS
    } else {
        &THREAD_AGENT_STATUS
    };
    one_of(allowed, v, "status")?;
    if !is_live(cx.doc) {
        return Err(refuse(format!(
            "{} is {}; only a running agent changes status.",
            display(&cx.doc["id"]),
            display(&cx.doc["status"])
        )));
    }
    Ok(v.clone())
}
