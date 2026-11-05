//! The board summary and the manual tests still to run, read from the mirrored docs, the same
//! data the pages show.

use serde_json::{Map, Value, json};

use crate::backend::{McpBackend, RunFilter};
use crate::handed::handed_run;
use crate::json::{items, nn, pick, same, text, truthy};
use crate::project::visible;

const LIVE_AGENT: [&str; 3] = ["working", "waiting", "stop-requested"];

fn visible_docs(backend: &dyn McpBackend, key: &str, coll: &str) -> Vec<Value> {
    backend
        .docs(key, coll)
        .into_iter()
        .filter(visible)
        .collect()
}

/// `done/total` of an item's acceptance criteria.
#[must_use]
pub fn ac_progress(item: &Value) -> String {
    let all = items(item, "ac");
    let done = all.iter().filter(|c| truthy(c, "done")).count();
    format!("{done}/{}", all.len())
}

fn thread_row(thread: &Value, agents: &[Value]) -> Value {
    let live = agents.iter().filter(|a| same(a, "thread", thread, "id"));
    let mut row = Map::new();
    pick(&mut row, thread, &["id", "status", "subject", "note"]);
    row.insert("agents".into(), json!(live.count()));
    pick(&mut row, thread, &["flags", "lastSeen", "session"]);
    Value::Object(row)
}

/// The short form of an item: id, status, thread, subject, title and the AC progress.
pub fn item_row(item: &Value) -> Map<String, Value> {
    let mut row = Map::new();
    pick(
        &mut row,
        item,
        &["id", "status", "thread", "subject", "title"],
    );
    row.insert("ac".into(), json!(ac_progress(item)));
    row
}

fn test_row(test: &Value) -> Value {
    let mut row = Map::new();
    pick(&mut row, test, &["id", "status", "mode"]);
    row.insert(
        "build".into(),
        nn(test, "build").cloned().unwrap_or(Value::Null),
    );
    pick(&mut row, test, &["item", "title"]);
    Value::Object(row)
}

fn lock_row(lock: &Value) -> Value {
    let mut row = Map::new();
    pick(&mut row, lock, &["resource", "holder"]);
    let queue = nn(lock, "queue").cloned().unwrap_or_else(|| json!([]));
    row.insert("queue".into(), queue);
    Value::Object(row)
}

fn ids_where(entries: &[Value], kind: &str, status: &str) -> Vec<Value> {
    entries
        .iter()
        .filter(|e| text(e, "kind") == Some(kind) && text(e, "status") == Some(status))
        .filter_map(|e| e.get("id").cloned())
        .collect()
}

fn summary_from_docs(backend: &dyn McpBackend, key: &str) -> Value {
    let threads = visible_docs(backend, key, "threads");
    let entries = visible_docs(backend, key, "entries");
    let meta = backend
        .doc(key, "meta", "board")
        .unwrap_or_else(|| json!({}));
    let agents: Vec<Value> = visible_docs(backend, key, "agents")
        .into_iter()
        .filter(|a| text(a, "status").is_some_and(|s| LIVE_AGENT.contains(&s)))
        .collect();
    let pending: Vec<Value> = visible_docs(backend, key, "actions")
        .iter()
        .filter(|a| text(a, "status") == Some("pending"))
        .filter_map(|a| a.get("id").cloned())
        .collect();
    json!({
        "project": key,
        "updatedAt": nn(&meta, "updatedAt").cloned().unwrap_or(Value::Null),
        "threads": threads.iter().map(|t| thread_row(t, &agents)).collect::<Vec<_>>(),
        "items": visible_docs(backend, key, "items").iter().map(|i| Value::Object(item_row(i))).collect::<Vec<_>>(),
        "openQuestions": ids_where(&entries, "question", "open"),
        "pendingDecisions": ids_where(&entries, "decision", "pending"),
        "handedRun": handed_run(backend, key),
        "tests": visible_docs(backend, key, "tests").iter().map(test_row).collect::<Vec<_>>(),
        "pendingActions": pending,
        "locks": backend.docs(key, "locks").iter().map(lock_row).collect::<Vec<_>>(),
    })
}

/// `{running, queued}` of a project's runs.
#[must_use]
pub fn run_counts(backend: &dyn McpBackend, key: &str) -> Value {
    let filter = RunFilter {
        limit: Some(500),
        ..RunFilter::default()
    };
    let runs = backend.run_list(Some(key), &filter);
    let count = |status: &str| {
        runs.iter()
            .filter(|r| text(r, "status") == Some(status))
            .count()
    };
    json!({ "running": count("running"), "queued": count("queued") })
}

/// The board in one call: the coord summary when the sync service holds the board, else the
/// same shape from the docs, without `artifact`, `dirty` and `inbox`, plus the runs and the run handed to the owner.
#[must_use]
pub fn board_summary(backend: &dyn McpBackend, key: &str) -> Value {
    let base = backend
        .coord_summary(key)
        .unwrap_or_else(|| summary_from_docs(backend, key));
    let Value::Object(mut rest) = base else {
        return base;
    };
    for dropped in ["artifact", "dirty", "inbox"] {
        rest.shift_remove(dropped);
    }
    rest.insert("runs".into(), run_counts(backend, key));
    rest.shift_remove("handedBuild");
    rest.insert("handedRun".into(), handed_run(backend, key));
    Value::Object(rest)
}
