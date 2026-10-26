//! Apply the owner's new entries to the board, oldest first: replies through [`super::replies`],
//! ops through the same `apply_op` the CLI uses, each stored on the board as an entry with its
//! outcome.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use argus_core::clock::{now_iso, now_ms, parse_ms};
use argus_core::js;
use serde_json::{Map, Value, json};

use super::repeat::is_repeat;
use super::replies::apply_reply;
use super::{is_op_kind, is_reply_kind};
use crate::Board;
use crate::error::BoardError;
use crate::inbox::{Message, message_id, push};
use crate::json::{
    array_mut, display, doc, doc_mut, find_ids, map_mut, nullish, set, shown, slot, strict_eq,
    to_number, truthy,
};
use crate::ops::{OpResult, Status, Who, apply_op, apply_purge, count_line, purge_plan};
use crate::store::{rank_end, remove_doc, touch};

const SPENT: [&str; 4] = ["applied", "cancelled", "duplicate", "dismissed"];
const OP_KEEP_MS: i64 = 24 * 3_600_000;
const EXPECT: [&str; 7] = [
    "threads", "items", "ac", "tests", "entries", "actions", "logLines",
];

/// How many of the owner's ops ended in each state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OpCounts {
    /// Applied.
    pub applied: usize,
    /// Left as a conflict.
    pub conflict: usize,
    /// Refused.
    pub refused: usize,
}

/// What importing the owner's entries did.
#[derive(Debug, Clone, Default)]
pub struct ImportResult {
    /// How many entries were applied (replies and ops; duplicates and cancelled ones not).
    pub imported: usize,
    /// The texts queued for each thread's inbox.
    pub deliveries: Map<String, Value>,
    /// The outcome of the ops.
    pub ops: OpCounts,
    /// Lines to log, for example what a purge did.
    pub notes: Vec<String>,
}

/// The owner's entry becomes a board doc. Returns the stored doc.
fn adopt(board: &mut Board, e: &Value, extra: Map<String, Value>) -> Value {
    let mut entry = e.clone();
    for (key, value) in extra {
        set(&mut entry, &key, value);
    }
    set(&mut entry, "createdBy", "human");
    set(&mut entry, "updatedAt", now_iso());
    set(&mut entry, "updatedBy", "human");
    set(&mut entry, "rev", 1);
    set(&mut entry, "rank", js::number(rank_end(board, "entries")));
    let id = display(&entry["id"]);
    map_mut(board, "entries").insert(id.clone(), entry.clone());
    touch(board, "entries", &id);
    entry
}

fn settled(status: &str) -> Map<String, Value> {
    Map::from_iter([
        ("status".to_owned(), json!(status)),
        ("appliedAt".to_owned(), json!(now_iso())),
    ])
}

/// Where a reply goes: the thread named, else the thread of what it answers, else the lead.
fn route_entry(board: &Board, e: &Value) -> String {
    let reply = nullish(e, "replyTo").unwrap_or_default();
    let item = nullish(e, "item").unwrap_or_default();
    [
        e["thread"].clone(),
        board["entries"][reply.as_str()]["thread"].clone(),
        board["tests"][reply.as_str()]["thread"].clone(),
        board["items"][item.as_str()]["thread"].clone(),
    ]
    .into_iter()
    .find(js::truthy)
    .map_or_else(|| "coordinator".to_owned(), |t| display(&t))
}

/// The thread an op entry is about, so purging that thread takes the entry with it.
fn thread_of_op(board: &Board, e: &Value) -> Value {
    if matches!(e["kind"].as_str(), Some("bulk" | "purge" | "build")) {
        return Value::Null;
    }
    if !e["set"]["thread"].is_null() {
        return e["set"]["thread"].clone();
    }
    let (c, id) = (&e["target"]["c"], &e["target"]["id"]);
    if c == "threads" {
        return id.clone();
    }
    board[display(c).as_str()][display(id).as_str()]["thread"].clone()
}

fn run_purge(
    board: &mut Board,
    entry: &Value,
    notes: &mut Vec<String>,
    home: Option<&Path>,
) -> Result<OpResult, BoardError> {
    if !strict_eq(&entry["confirm"], &board["project"]) {
        return Ok(OpResult::refused(
            "the confirm text must equal the project key",
        ));
    }
    if !entry["expect"].is_object() {
        return Ok(OpResult::refused("board changed, review again"));
    }
    let criteria = if entry["criteria"].is_null() {
        json!({})
    } else {
        entry["criteria"].clone()
    };
    let plan = match purge_plan(board, &criteria) {
        Ok(plan) => plan,
        Err(e) => return Ok(OpResult::refused(e.to_string())),
    };
    let stale = EXPECT.iter().any(|key| {
        let planned = plan.counts.get(*key).map_or(0.0, |n| to_number(Some(n)));
        let limit = match &entry["expect"][*key] {
            Value::Null => 0.0,
            v => to_number(Some(v)),
        };
        planned > limit
    });
    if stale {
        return Ok(OpResult::refused("board changed, review again"));
    }
    let Some(home) = home else {
        return Ok(OpResult::refused("purging needs the coord home"));
    };
    let done = apply_purge(home, board, &plan, &Who::human())?;
    let outcome = format!("purged {}; backup {}", count_line(&done.counts), done.file);
    notes.push(format!("owner {outcome}"));
    Ok(OpResult::applied(outcome))
}

fn run_op(
    board: &mut Board,
    entry: &Value,
    notes: &mut Vec<String>,
    home: Option<&Path>,
) -> OpResult {
    if entry["kind"] == "purge" {
        run_purge(board, entry, notes, home).unwrap_or_else(|e| OpResult::refused(e.to_string()))
    } else {
        apply_op(board, entry, &Who::human(), home)
    }
}

fn write_outcome(board: &mut Board, id: &str, entry: &Value, result: &OpResult) {
    let fallback = board[display(&entry["target"]["c"]).as_str()]
        [display(&entry["target"]["id"]).as_str()]["updatedBy"]
        .clone();
    let Some(stored) = doc_mut(board, "entries", id) else {
        return;
    };
    set(stored, "status", result.status.as_str());
    set(stored, "outcome", result.outcome.as_str());
    if let Some(results) = &result.results {
        set(stored, "results", results.clone());
    }
    if result.status == Status::Applied {
        set(stored, "appliedAt", now_iso());
    }
    if result.status == Status::Conflict {
        set(
            stored,
            "current",
            result.current.clone().unwrap_or_else(|| json!({})),
        );
        let by = result.by.clone().map_or(fallback, Value::String);
        set(stored, "by", by);
    }
}

fn queue_notices(
    board: &mut Board,
    id: &str,
    kind: &str,
    result: &OpResult,
    queued: &HashMap<String, usize>,
) {
    for (i, notice) in result.notify.iter().enumerate() {
        if notice.to.is_empty() || notice.text.is_empty() {
            continue;
        }
        let skip = queued.get(&notice.to).copied().unwrap_or(0);
        let inbox = array_mut(slot(board, "inbox"), &notice.to);
        if inbox
            .iter()
            .skip(skip)
            .any(|m| m["text"] == notice.text.as_str())
        {
            continue;
        }
        let message = Message {
            id: message_id(&format!("{id}-{i}")),
            from: "human",
            kind,
            text: &notice.text,
            reference: Some(id),
        };
        push(board, &notice.to, &message);
    }
}

fn apply_op_entry(board: &mut Board, e: &Value, out: &mut ImportResult, home: Option<&Path>) {
    let extra = Map::from_iter([("thread".to_owned(), thread_of_op(board, e))]);
    let entry = adopt(board, e, extra);
    let id = display(&entry["id"]);
    let queued: HashMap<String, usize> = board["inbox"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(to, list)| (to.clone(), list.as_array().map_or(0, Vec::len)))
        .collect();
    let result = run_op(board, &entry, &mut out.notes, home);
    write_outcome(board, &id, &entry, &result);
    match result.status {
        Status::Applied => out.ops.applied += 1,
        Status::Conflict => out.ops.conflict += 1,
        Status::Refused => out.ops.refused += 1,
    }
    queue_notices(
        board,
        &id,
        entry["kind"].as_str().unwrap_or_default(),
        &result,
        &queued,
    );
    touch(board, "entries", &id);
}

fn reply_target(board: &Board, entry: &Value) -> String {
    let kind = entry["kind"].as_str().unwrap_or_default();
    let key = shown(entry.get("replyTo"));
    let routed = matches!(kind, "done" | "result")
        .then(|| {
            nullish(&board["actions"][key.as_str()], "thread")
                .or_else(|| nullish(&board["tests"][key.as_str()], "thread"))
        })
        .flatten();
    routed
        .or_else(|| nullish(entry, "thread"))
        .unwrap_or_else(|| "undefined".to_owned())
}

fn apply_reply_entry(
    board: &mut Board,
    e: &Value,
    out: &mut ImportResult,
) -> Result<(), BoardError> {
    let extra = Map::from_iter([
        ("thread".to_owned(), json!(route_entry(board, e))),
        ("appliedAt".to_owned(), json!(now_iso())),
    ]);
    let entry = adopt(board, e, extra);
    let text = apply_reply(board, &entry)?;
    out.imported += 1;
    let Some(text) = text else { return Ok(()) };
    let to = reply_target(board, &entry);
    let id = display(&entry["id"]);
    let message = Message {
        id: message_id(&id),
        from: "human",
        kind: entry["kind"].as_str().unwrap_or_default(),
        text: &text,
        reference: Some(&id),
    };
    push(board, &to, &message);
    if let Value::Array(list) = out.deliveries.entry(to).or_insert_with(|| json!([])) {
        list.push(json!(text));
    }
    Ok(())
}

fn is_fresh(board: &Board, e: &Value, known: &dyn Fn(&str) -> bool) -> bool {
    let Some(id) = e["id"].as_str() else {
        return false;
    };
    e["author"] == "human"
        && doc(board, "entries", id).is_none()
        && !truthy(&board["dirty"], &format!("entries/{id}"))
        && !known(id)
        && e["kind"]
            .as_str()
            .is_some_and(|k| is_reply_kind(k) || is_op_kind(k))
}

/// An undo posted in the same batch as the entry it undoes: neither is applied.
fn cancelled_ids(fresh: &[Value]) -> HashSet<String> {
    let batch: HashSet<&str> = fresh.iter().filter_map(|e| e["id"].as_str()).collect();
    let mut cancelled = HashSet::new();
    for e in fresh {
        let undoes = e["undoes"].as_str().filter(|u| !u.is_empty());
        if let Some(undoes) = undoes.filter(|u| batch.contains(u) && !cancelled.contains(*u)) {
            cancelled.insert(display(&e["id"]));
            cancelled.insert(undoes.to_owned());
        }
    }
    cancelled
}

/// An op entry that is done with stays a day for the page to show its outcome, then goes.
fn remove_spent(board: &mut Board) {
    let spent = |e: &Value| {
        let at = nullish(e, "appliedAt")
            .or_else(|| nullish(e, "createdAt"))
            .and_then(|t| parse_ms(&t));
        e["author"] == "human"
            && e["kind"].as_str().is_some_and(is_op_kind)
            && e["status"].as_str().is_some_and(|s| SPENT.contains(&s))
            && at.is_some_and(|ms| now_ms() - ms > OP_KEEP_MS)
    };
    for id in find_ids(board, "entries", spent) {
        remove_doc(board, "entries", &id);
    }
}

fn process(
    board: &mut Board,
    e: &Value,
    cancelled: &HashSet<String>,
    (out, home): (&mut ImportResult, Option<&Path>),
) {
    let id = display(&e["id"]);
    if cancelled.contains(&id) {
        adopt(board, e, settled("cancelled"));
    } else if is_repeat(board, e) {
        adopt(board, e, settled("duplicate"));
    } else if e["kind"].as_str().is_some_and(is_op_kind) {
        apply_op_entry(board, e, out, home);
        out.imported += 1;
    } else if let Err(err) = apply_reply_entry(board, e, out) {
        out.notes
            .push(format!("{} {id}: {err}", display(&e["kind"])));
    }
}

fn import_with(
    home: Option<&Path>,
    board: &mut Board,
    mut entries: Vec<Value>,
    known: &dyn Fn(&str) -> bool,
) -> ImportResult {
    entries.sort_by_cached_key(|e| shown(e.get("createdAt")));
    let fresh: Vec<Value> = entries
        .into_iter()
        .filter(|e| is_fresh(board, e, known))
        .collect();
    let cancelled = cancelled_ids(&fresh);
    let mut out = ImportResult::default();
    for e in &fresh {
        process(board, e, &cancelled, (&mut out, home));
    }
    remove_spent(board);
    out
}

/// Apply the owner's new entries to the board, oldest first (the dashboard calls it for each
/// post, under the board lock). `known(id)` lets the caller skip ids it has already handled.
/// Known ids, unknown kinds and entries already on the board are skipped. A `purge` op is refused
/// here because it needs the coord home; use [`import_entries_at`] for it.
pub fn import_entries(
    board: &mut Board,
    entries: Vec<Value>,
    known: &dyn Fn(&str) -> bool,
) -> ImportResult {
    import_with(None, board, entries, known)
}

/// [`import_entries`] with the coord home, so a `purge` op can write its backup under it.
pub fn import_entries_at(
    home: &Path,
    board: &mut Board,
    entries: Vec<Value>,
    known: &dyn Fn(&str) -> bool,
) -> ImportResult {
    import_with(Some(home), board, entries, known)
}
