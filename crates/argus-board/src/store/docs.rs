//! The bookkeeping every doc shares: ids, who made and last changed it, the revision, the dirty
//! mark, and the work log.

use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::rank::rank_end;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{array_mut, display, doc_mut, map_mut, section, set, slot};

const LOG_KEEP: usize = 200;

/// Mark a doc as changed. Kept for older tools reading the board.
pub fn touch(board: &mut Board, coll: &str, id: &str) {
    map_mut(board, "dirty").insert(format!("{coll}/{id}"), Value::Bool(true));
}

/// The value of a counter, 0 when unset.
fn counter(board: &Board, key: &str) -> i64 {
    board["counters"][key].as_i64().unwrap_or(0)
}

fn bump(board: &mut Board, key: &str) -> i64 {
    let n = counter(board, key) + 1;
    map_mut(board, "counters").insert(key.to_owned(), json!(n));
    n
}

/// The next id of a prefix: `I-1`, `I-2`, ... Ids are never reused.
pub fn next_id(board: &mut Board, prefix: &str) -> String {
    let n = bump(board, prefix);
    format!("{prefix}-{n}")
}

/// Create a doc with its bookkeeping set: created and updated by `actor` now, revision 1, a rank
/// at the end, and the dirty mark. The id is `fields.id` when named, else the next
/// `<prefix>-<n>`. Fields given win over the defaults. Returns the id.
///
/// # Errors
/// Refused when the collection already holds the id.
pub fn create_doc(
    board: &mut Board,
    coll: &str,
    prefix: &str,
    fields: Map<String, Value>,
    actor: &str,
) -> Result<String, BoardError> {
    let id = match fields.get("id").filter(|v| !v.is_null()) {
        Some(named) => display(named),
        None => next_id(board, prefix),
    };
    if section(board, coll).contains_key(&id) {
        return Err(refuse(format!("{coll} already has \"{id}\".")));
    }
    let at = now_iso();
    let mut doc = Map::new();
    doc.insert("id".into(), json!(id));
    doc.insert("createdAt".into(), json!(at));
    doc.insert("createdBy".into(), json!(actor));
    doc.insert("updatedAt".into(), json!(at));
    doc.insert("updatedBy".into(), json!(actor));
    doc.insert("rank".into(), js::number(rank_end(board, coll)));
    doc.extend(fields);
    doc.insert("rev".into(), json!(1));
    map_mut(board, coll).insert(id.clone(), Value::Object(doc));
    touch(board, coll, &id);
    Ok(id)
}

/// Record a change: who and when, one more revision, and the dirty mark.
///
/// # Errors
/// Refused when the collection has no such doc.
pub fn stamp(board: &mut Board, coll: &str, id: &str, actor: &str) -> Result<(), BoardError> {
    let Some(doc) = doc_mut(board, coll, id) else {
        return Err(refuse(format!("no {coll} doc \"{id}\" to stamp.")));
    };
    let rev = doc["rev"].as_i64().unwrap_or(0) + 1;
    set(doc, "updatedAt", now_iso());
    set(doc, "updatedBy", actor);
    set(doc, "rev", rev);
    touch(board, coll, id);
    Ok(())
}

/// Delete a doc for good. The dashboard finds it gone from the board and deletes its copy.
pub fn remove_doc(board: &mut Board, coll: &str, id: &str) {
    if let Some(m) = board.get_mut(coll).and_then(Value::as_object_mut) {
        m.shift_remove(id);
    }
    touch(board, coll, id);
}

/// One line of a work log.
#[derive(Debug, Clone, Copy)]
pub struct LogLine<'a> {
    /// Who wrote it.
    pub thread: &'a str,
    /// The item it is about.
    pub item: Option<&'a str>,
    /// The text.
    pub text: &'a str,
}

/// Add a line to a subject's work log: numbered per subject (never reused), the latest 200
/// kept. Returns the line's number.
pub fn append_log(board: &mut Board, subject: &str, line: &LogLine) -> i64 {
    push_log_line(board, subject, line, Some(LOG_KEEP))
}

/// Add a line to a subject's log, keeping only the latest `keep` lines when given.
pub(crate) fn push_log_line(
    board: &mut Board,
    subject: &str,
    line: &LogLine,
    keep: Option<usize>,
) -> i64 {
    let at = now_iso();
    let seq = bump(board, &format!("log:{subject}"));
    let doc = slot(slot(board, "log"), subject);
    if doc.is_null() {
        *doc = json!({"subject": subject, "entries": [], "updatedAt": at});
    }
    let entries = array_mut(doc, "entries");
    entries.push(
        json!({"seq": seq, "ts": at, "thread": line.thread, "item": line.item, "text": line.text}),
    );
    if let Some(keep) = keep.filter(|k| entries.len() > *k) {
        let extra = entries.len() - keep;
        entries.drain(..extra);
    }
    set(doc, "updatedAt", at);
    touch(board, "log", subject);
    seq
}
