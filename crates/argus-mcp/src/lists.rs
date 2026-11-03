//! The list and get tools over threads, items, tests and entries.

use serde_json::{Map, Value, json};

use crate::args::{arg_str, arg_u64, flag};
use crate::backend::McpBackend;
use crate::error::ToolError;
use crate::json::{items, nn, pick, text, truthy};
use crate::project::{clip, need, sorted_docs, visible};
use crate::summary::item_row;

const OP_KINDS: [&str; 15] = [
    "create",
    "edit",
    "move",
    "ac",
    "archive",
    "unarchive",
    "delete",
    "restore",
    "bulk",
    "purge",
    "build",
    "avatar",
    "plan-run-start",
    "plan-run-record",
    "plan-run-end",
];

fn is_op(entry: &Value) -> bool {
    text(entry, "kind").is_some_and(|k| OP_KINDS.contains(&k))
}

fn is_text(doc: &Value, key: &str, wanted: &str) -> bool {
    text(doc, key) == Some(wanted)
}

fn slim_item(item: &Value) -> Value {
    let mut row = item_row(item);
    pick(&mut row, item, &["rev", "updatedAt"]);
    if truthy(item, "archivedAt") {
        row.insert("archived".into(), json!(true));
    }
    Value::Object(row)
}

fn slim_test(test: &Value) -> Value {
    let mut row = Map::new();
    pick(
        &mut row,
        test,
        &["id", "item", "thread", "title", "covers", "mode", "status"],
    );
    row.insert(
        "build".into(),
        nn(test, "build").cloned().unwrap_or(Value::Null),
    );
    row.insert("proofs".into(), json!(items(test, "proofs").len()));
    row.insert(
        "result".into(),
        nn(test, "result").cloned().unwrap_or(Value::Null),
    );
    Value::Object(row)
}

fn hidden_thread(t: &Value) -> bool {
    is_text(t, "status", "archived") || is_text(t, "status", "paused")
}

/// `list_threads`.
pub fn list_threads(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let all = flag(args, "includeHidden");
    let rows = sorted_docs(backend, key, "threads", |t| {
        !truthy(t, "deletedAt") && (all || (!truthy(t, "archivedAt") && !hidden_thread(t)))
    });
    json!({ "threads": rows })
}

fn matches_arg(doc: &Value, args: &Value, key: &str) -> bool {
    arg_str(args, key).is_none_or(|want| is_text(doc, key, want))
}

/// `list_items`.
pub fn list_items(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let archived = flag(args, "includeArchived");
    let rows = sorted_docs(backend, key, "items", |i| {
        !truthy(i, "deletedAt")
            && (archived || !truthy(i, "archivedAt"))
            && ["status", "thread", "subject"]
                .iter()
                .all(|k| matches_arg(i, args, k))
    });
    let slim: Vec<Value> = clip(rows, arg_u64(args, "limit"))
        .iter()
        .map(slim_item)
        .collect();
    json!({ "items": slim })
}

fn about_item(doc: &Value, id: &str) -> bool {
    is_text(doc, "item", id) && visible(doc)
}

/// `get_item`.
pub fn get_item(backend: &dyn McpBackend, key: &str, args: &Value) -> Result<Value, ToolError> {
    let id = arg_str(args, "id").unwrap_or_default();
    let item = need(
        backend.doc(key, "items", id),
        &format!("item {id} in {key}"),
    )?;
    let tests: Vec<Value> = sorted_docs(backend, key, "tests", |t| about_item(t, id))
        .iter()
        .map(slim_test)
        .collect();
    let entries = sorted_docs(backend, key, "entries", |e| about_item(e, id) && !is_op(e));
    let actions = sorted_docs(backend, key, "actions", |x| about_item(x, id));
    Ok(json!({ "item": item, "tests": tests, "entries": entries, "actions": actions }))
}

/// `list_tests`.
pub fn list_tests(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let rows = sorted_docs(backend, key, "tests", |t| {
        visible(t)
            && ["item", "status", "build", "mode"]
                .iter()
                .all(|k| matches_arg(t, args, k))
    });
    let slim: Vec<Value> = clip(rows, arg_u64(args, "limit"))
        .iter()
        .map(slim_test)
        .collect();
    json!({ "tests": slim })
}

/// A url the dashboard serves (`/files/...`) made absolute on `base`; other values stay.
pub fn absolute(url: &Value, base: &str) -> Value {
    match url.as_str() {
        Some(u) if u.starts_with('/') => json!(format!("{base}{u}")),
        _ => url.clone(),
    }
}

fn public_test(test: &Value, base: &str) -> Value {
    let proofs: Vec<Value> = items(test, "proofs")
        .iter()
        .map(|p| {
            let mut proof = p.clone();
            if let (Some(map), Some(url)) = (proof.as_object_mut(), p.get("url")) {
                map.insert("url".into(), absolute(url, base));
            }
            proof
        })
        .collect();
    let mut out = test.clone();
    if let Some(map) = out.as_object_mut() {
        map.insert("proofs".into(), Value::Array(proofs));
    }
    out
}

/// `get_test`, with proof urls made absolute on `base`.
pub fn get_test(
    backend: &dyn McpBackend,
    key: &str,
    args: &Value,
    base: &str,
) -> Result<Value, ToolError> {
    let id = arg_str(args, "id").unwrap_or_default();
    let test = need(
        backend.doc(key, "tests", id),
        &format!("test {id} in {key}"),
    )?;
    Ok(json!({ "test": public_test(&test, base) }))
}

/// `list_entries`: the conversation, without the owner's page edits.
pub fn list_entries(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let kind = arg_str(args, "kind");
    let open = flag(args, "open");
    let rows = sorted_docs(backend, key, "entries", |e| {
        visible(e)
            && kind.map_or(!is_op(e), |k| is_text(e, "kind", k))
            && (!open || is_text(e, "status", "open") || is_text(e, "status", "pending"))
    });
    json!({ "entries": clip(rows, arg_u64(args, "limit")) })
}
