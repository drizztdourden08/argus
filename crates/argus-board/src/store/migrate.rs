//! The two in-memory migrations of a loaded board: schema 1 to 2, and embedded sub-agents to
//! agents docs. Neither loses anything: ids, counters and versions stay.

use std::collections::HashSet;

use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

use super::docs::{create_doc, touch};
use crate::Board;
use crate::json::{
    array_mut, default_field, doc, doc_mut, ids, map_mut, nullish, section, set, shown, unset,
};

const COLLECTIONS: [&str; 5] = ["threads", "items", "tests", "entries", "actions"];
const RANK_STEP: i64 = 1024;

fn created_of(doc: &Value) -> String {
    nullish(doc, "createdAt")
        .or_else(|| nullish(doc, "registeredAt"))
        .unwrap_or_default()
}

fn fill_bookkeeping(doc: &mut Value, coll: &str, index: i64, stamped: &str) {
    default_field(doc, "rev", 1);
    let author = nullish(doc, "author")
        .or_else(|| nullish(doc, "thread"))
        .unwrap_or_else(|| "unknown".to_owned());
    default_field(doc, "createdBy", author);
    let by = doc["createdBy"].clone();
    default_field(doc, "updatedBy", by);
    let created = created_of(doc);
    let updated = if created.is_empty() {
        stamped
    } else {
        &created
    };
    default_field(doc, "updatedAt", updated);
    default_field(doc, "rank", (index + 1) * RANK_STEP);
    if coll == "threads" && doc["status"] == "archived" {
        let at = doc["updatedAt"].clone();
        default_field(doc, "archivedAt", at);
    }
}

fn migrate_collection(board: &mut Board, coll: &str, stamped: &str) {
    let mut order: Vec<(String, String)> = section(board, coll)
        .iter()
        .map(|(id, d)| (id.clone(), created_of(d)))
        .collect();
    order.sort_by(|a, b| a.1.cmp(&b.1));
    for (index, (id, _)) in (0_i64..).zip(&order) {
        if let Some(d) = doc_mut(board, coll, id) {
            fill_bookkeeping(d, coll, index, stamped);
        }
        touch(board, coll, id);
    }
}

fn number_log_lines(board: &mut Board) {
    for subject in ids(board, "log") {
        let key = format!("log:{subject}");
        let mut seq = board["counters"][key.as_str()].as_i64().unwrap_or(0);
        if let Some(log) = doc_mut(board, "log", &subject) {
            for line in array_mut(log, "entries") {
                if line["seq"].is_null() {
                    seq += 1;
                    set(line, "seq", seq);
                }
            }
        }
        map_mut(board, "counters").insert(key, json!(seq));
        touch(board, "log", &subject);
    }
}

/// Schema 1 to 2: every doc gains its bookkeeping fields, rank follows creation order, log lines
/// get a sequence number and every doc is marked dirty.
pub(crate) fn migrate_to_2(board: &mut Board) {
    let stamped = now_iso();
    for coll in COLLECTIONS {
        migrate_collection(board, coll, &stamped);
    }
    number_log_lines(board);
    for resource in ids(board, "locks") {
        touch(board, "locks", &resource);
    }
    set(board, "schema", 2);
    unset(board, "version");
}

fn seen_agents(board: &Board) -> HashSet<String> {
    section(board, "agents")
        .values()
        .map(|d| agent_key(d.get("thread"), d.get("name"), d.get("startedAt")))
        .collect()
}

fn agent_key(thread: Option<&Value>, name: Option<&Value>, at: Option<&Value>) -> String {
    format!("{}|{}|{}", shown(thread), shown(name), shown(at))
}

fn agent_fields(thread: &str, a: &Value, at: &str) -> Map<String, Value> {
    let or = |key: &str, default: Value| match &a[key] {
        Value::Null => default,
        v => v.clone(),
    };
    let fields = json!({
        "name": a["name"], "thread": thread, "task": or("task", json!("")),
        "model": or("model", Value::Null), "kind": "build", "item": null, "branch": null,
        "worktree": null, "status": "working", "note": null, "result": null,
        "startedAt": at, "createdAt": at, "updatedAt": at, "finishedAt": null,
    });
    match fields {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

fn convert_thread(board: &mut Board, thread_id: &str) {
    let Some(t) = doc(board, "threads", thread_id).filter(|t| t.get("agents").is_some()) else {
        return;
    };
    let tid = nullish(t, "id").unwrap_or_else(|| thread_id.to_owned());
    let list = t["agents"].as_array().cloned().unwrap_or_default();
    let mut seen = seen_agents(board);
    for a in &list {
        let at = nullish(a, "startedAt").unwrap_or_else(now_iso);
        set(board, "agentsMigrated", true);
        let key = agent_key(Some(&json!(tid)), a.get("name"), Some(&json!(at)));
        if seen.insert(key) {
            create_doc(board, "agents", "A", agent_fields(&tid, a, &at), &tid).ok();
        }
    }
    if let Some(t) = doc_mut(board, "threads", thread_id) {
        unset(t, "agents");
    }
    touch(board, "threads", &tid);
}

/// Embedded `thread.agents[]` entries become agents docs, still working, created when they
/// started; the array goes. Idempotent: an entry whose thread, name and start already have a doc
/// is dropped, not doubled.
pub(crate) fn convert_agents(board: &mut Board) {
    for thread_id in ids(board, "threads") {
        convert_thread(board, &thread_id);
    }
}
