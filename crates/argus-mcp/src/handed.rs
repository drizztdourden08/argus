//! The run handed to the owner: a test run ready to start, given to the owner, whose build is
//! still live. The same rule as the automatic ping of `argus-notify`.

use serde_json::{Value, json};

use crate::backend::McpBackend;
use crate::json::{nn, text, truthy};
use crate::project::visible;

fn build_live(backend: &dyn McpBackend, key: &str, run: &Value) -> bool {
    text(&run["fields"], "build")
        .and_then(|name| backend.doc(key, "launchers", name))
        .is_some_and(|launcher| !truthy(&launcher, "retiredAt"))
}

/// The test runs handed to the owner.
pub fn handed_runs(backend: &dyn McpBackend, key: &str) -> Vec<Value> {
    backend
        .docs(key, "items")
        .into_iter()
        .filter(|i| text(i, "type") == Some("test-run") && visible(i))
        .filter(|i| text(i, "stage") == Some("ready") && text(i, "assignee") == Some("owner"))
        .filter(|i| build_live(backend, key, i))
        .collect()
}

/// The handed run changed last, or null: its id, title, build, thread and the time it was handed.
pub fn handed_run(backend: &dyn McpBackend, key: &str) -> Value {
    let newest = handed_runs(backend, key)
        .into_iter()
        .max_by_key(|r| text(r, "updatedAt").map(str::to_owned));
    let Some(run) = newest else {
        return Value::Null;
    };
    json!({
        "id": run["id"],
        "title": nn(&run, "title").cloned().unwrap_or(Value::Null),
        "build": run["fields"]["build"],
        "thread": nn(&run, "thread").cloned().unwrap_or(Value::Null),
        "at": nn(&run, "updatedAt").cloned().unwrap_or(Value::Null),
    })
}
