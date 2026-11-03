//! What each reply kind of owner entry does to the board. Each returns the inbox text for the
//! thread, or `None` when the thread is not told.

use argus_core::clock::now_iso;
use serde_json::Value;

use super::is_op_kind;
use crate::Board;
use crate::error::BoardError;
use crate::json::{doc, doc_mut, nullish, set, shown, text, truthy};
use crate::ops::refresh_needs_human;
use crate::pause::{pause_thread, resume_thread};
use crate::results::{Outcome, record_result};
use crate::store::{LogLine, append_log, stamp};

const HUMAN: &str = "human";
const OWNER: &str = "the owner";

/// `e.replyTo ?? ''`, the key of the record a reply answers.
fn reply_key(e: &Value) -> String {
    nullish(e, "replyTo").unwrap_or_default()
}

fn set_status(board: &mut Board, coll: &str, id: &str, status: &str) -> Result<(), BoardError> {
    if let Some(d) = doc_mut(board, coll, id) {
        set(d, "status", status);
    }
    stamp(board, coll, id, HUMAN)
}

/// The sync flavour of the needs-human flag: no archive or trash filter, always touched.
fn pause(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let key = reply_key(e);
    if pause_thread(board, &key, OWNER)? {
        stamp(board, "threads", &key, HUMAN)?;
    }
    Ok(None)
}

fn resume(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let key = reply_key(e);
    if resume_thread(board, &key, OWNER) {
        stamp(board, "threads", &key, HUMAN)?;
    }
    Ok(None)
}

fn followup(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let key = reply_key(e);
    let answerable = doc(board, "entries", &key)
        .is_some_and(|t| matches!(t["kind"].as_str(), Some("question" | "message")));
    if answerable {
        set_status(board, "entries", &key, "answered")?;
    }
    let on = if truthy(e, "replyTo") {
        format!(" on {}", shown(e.get("replyTo")))
    } else {
        String::new()
    };
    Ok(Some(format!(
        "{} follow-up{on}: {}",
        shown(e.get("id")),
        shown(e.get("text"))
    )))
}

fn decision(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let key = reply_key(e);
    if doc(board, "entries", &key).is_some_and(|t| t["kind"] == "question") {
        set_status(board, "entries", &key, "answered")?;
    }
    let id = shown(e.get("id"));
    if let Some(stored) = doc_mut(board, "entries", &id) {
        set(stored, "status", "pending");
    }
    let on = if truthy(e, "replyTo") {
        format!(" on {}", shown(e.get("replyTo")))
    } else {
        String::new()
    };
    let item = if truthy(e, "item") {
        format!(" ({})", shown(e.get("item")))
    } else {
        String::new()
    };
    Ok(Some(format!(
        "{id} decision{on}{item}: {}. Run coord entry pick {id} when you start on it.",
        shown(e.get("text"))
    )))
}

fn ack(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let key = reply_key(e);
    let Some(target) = doc(board, "entries", &key) else {
        return Ok(None);
    };
    let spent = target["kind"].as_str().is_some_and(is_op_kind)
        && !matches!(target["status"].as_str(), Some("conflict" | "refused"));
    if !spent {
        set_status(board, "entries", &key, "dismissed")?;
    }
    Ok(None)
}

fn done(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let key = reply_key(e);
    let Some(action) = doc(board, "actions", &key).cloned() else {
        return Ok(None);
    };
    if action["status"] == "done" {
        return Ok(None);
    }
    if let Some(a) = doc_mut(board, "actions", &key) {
        set(a, "status", "done");
        let at = nullish(e, "createdAt").unwrap_or_else(now_iso);
        set(a, "doneAt", at);
    }
    stamp(board, "actions", &key, HUMAN)?;
    refresh_needs_human(board, &shown(action.get("thread")));
    Ok(Some(format!(
        "{} done by the owner: {}",
        shown(action.get("id")),
        shown(action.get("text"))
    )))
}

/// The test a reply is about, with its key.
fn find_test(board: &Board, e: &Value) -> Option<(String, Value)> {
    let key = reply_key(e);
    doc(board, "tests", &key).cloned().map(|t| (key, t))
}

fn result(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let Some((key, test)) = find_test(board, e) else {
        return Ok(None);
    };
    let note = if truthy(e, "text") {
        shown(e.get("text"))
    } else {
        String::new()
    };
    let outcome = Outcome {
        status: &e["outcome"],
        text: if note.is_empty() {
            "recorded on the board"
        } else {
            &note
        },
        by: HUMAN,
    };
    record_result(board, &key, &outcome)?;
    stamp(board, "tests", &key, HUMAN)?;
    Ok(Some(format!(
        "{} {} (run by the owner): {note}",
        shown(test.get("id")),
        shown(e.get("outcome"))
    )))
}

/// The owner runs a test again: its status goes back to pending, the runs history and
/// everything else stay. Already pending changes nothing. One audit line, no inbox message.
fn rerun(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    let Some((key, test)) = find_test(board, e) else {
        return Ok(None);
    };
    if truthy(&test, "deletedAt") || test["status"] == "pending" {
        return Ok(None);
    }
    set_status(board, "tests", &key, "pending")?;
    let line = format!(
        "owner set {} back to pending to run it again (was {})",
        shown(test.get("id")),
        shown(test.get("status"))
    );
    let item = nullish(&test, "item");
    append_log(
        board,
        "audit",
        &LogLine {
            thread: HUMAN,
            item: item.as_deref(),
            text: &line,
        },
    );
    Ok(None)
}

/// Apply a reply entry (already stored on the board) and return the text for the thread's inbox.
pub(crate) fn apply_reply(board: &mut Board, e: &Value) -> Result<Option<String>, BoardError> {
    match text(e, "kind").unwrap_or_default() {
        "pause" => pause(board, e),
        "resume" => resume(board, e),
        "followup" => followup(board, e),
        "decision" => decision(board, e),
        "ack" => ack(board, e),
        "done" => done(board, e),
        "result" => result(board, e),
        "rerun" => rerun(board, e),
        _ => Ok(None),
    }
}
