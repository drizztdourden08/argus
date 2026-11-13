//! `list_dependencies`: the cross-board dependencies of the ledger (the `deps` docs of the
//! portfolio scope), optionally those that start or end on one project, or in one state.

use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use crate::args::arg_str;
use crate::backend::McpBackend;
use crate::json::text;
use crate::project::resolve_project;

fn touches(dep: &Value, project: &str) -> bool {
    ["from", "to"]
        .iter()
        .any(|end| text(&dep[end], "project") == Some(project))
}

/// The dependencies, oldest first as the ledger holds them.
pub fn list_dependencies(backend: &dyn McpBackend, args: &Value) -> Value {
    let resolved = arg_str(args, "project")
        .map(|key| resolve_project(backend, key).unwrap_or_else(|| key.to_owned()));
    let project = resolved.as_deref();
    let state = arg_str(args, "state");
    let deps: Vec<Value> = backend
        .docs(PORTFOLIO, "deps")
        .into_iter()
        .filter(|d| project.is_none_or(|p| touches(d, p)))
        .filter(|d| state.is_none_or(|s| text(d, "state") == Some(s)))
        .collect();
    json!({ "deps": deps })
}
