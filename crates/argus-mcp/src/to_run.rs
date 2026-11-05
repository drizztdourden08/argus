//! The manual tests still to run, grouped by the test run that holds them.

use std::collections::HashMap;

use serde_json::{Map, Value, json};

use crate::backend::McpBackend;
use crate::handed::{handed_run, handed_runs};
use crate::json::{items, nn, pick, text};
use crate::project::{by_id, visible};

fn open_runs(backend: &dyn McpBackend, key: &str) -> Vec<Value> {
    let mut runs: Vec<Value> = backend
        .docs(key, "items")
        .into_iter()
        .filter(|i| text(i, "type") == Some("test-run") && visible(i))
        .filter(|i| matches!(text(i, "stage"), Some("ready" | "running")))
        .collect();
    runs.sort_by(by_id);
    runs
}

fn answered(run: &Value) -> Vec<&str> {
    items(&run["fields"], "results")
        .iter()
        .filter(|r| text(r, "status").is_some_and(|s| s != "pending"))
        .filter_map(|r| text(r, "test"))
        .collect()
}

fn test_row(test: &Value) -> Value {
    let mut row = Map::new();
    pick(&mut row, test, &["id", "item", "title"]);
    row.insert(
        "covers".into(),
        nn(test, "covers").cloned().unwrap_or(Value::Null),
    );
    Value::Object(row)
}

/// The manual tests and the test plans of a board, by id.
struct Known {
    tests: HashMap<String, Value>,
    plans: HashMap<String, Value>,
}

fn known(backend: &dyn McpBackend, key: &str) -> Known {
    let tests = backend
        .docs(key, "tests")
        .into_iter()
        .filter(|t| visible(t) && text(t, "mode") == Some("manual"))
        .filter_map(|t| Some((text(&t, "id")?.to_owned(), t)))
        .collect();
    let plans = backend
        .docs(key, "items")
        .into_iter()
        .filter(|i| text(i, "type") == Some("test-plan"))
        .filter_map(|i| Some((text(&i, "id")?.to_owned(), i)))
        .collect();
    Known { tests, plans }
}

fn group(run: &Value, known: &Known, handed: &[Value]) -> Option<Value> {
    let plan = text(&run["fields"], "plan").and_then(|p| known.plans.get(p))?;
    let done = answered(run);
    let rows: Vec<Value> = items(&plan["fields"], "cases")
        .iter()
        .filter_map(Value::as_str)
        .filter(|id| !done.contains(id))
        .filter_map(|id| known.tests.get(id))
        .map(test_row)
        .collect();
    let is_handed = handed.iter().any(|h| h["id"] == run["id"]);
    (!rows.is_empty()).then(|| {
        json!({
            "run": run["id"], "title": nn(run, "title").cloned().unwrap_or(Value::Null),
            "build": nn(&run["fields"], "build").cloned().unwrap_or(Value::Null),
            "handed": is_handed, "tests": rows,
        })
    })
}

/// Manual tests not run yet, grouped by test run, the handed run first.
#[must_use]
pub fn tests_to_run(backend: &dyn McpBackend, key: &str) -> Value {
    let known = known(backend, key);
    let handed = handed_runs(backend, key);
    let mut groups: Vec<Value> = open_runs(backend, key)
        .iter()
        .filter_map(|run| group(run, &known, &handed))
        .collect();
    groups.sort_by_key(|g| std::cmp::Reverse(g["handed"] == true));
    json!({ "handedRun": handed_run(backend, key), "groups": groups })
}
