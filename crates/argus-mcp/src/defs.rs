//! The tool definitions: names, titles, descriptions, input schemas and annotations, loaded from
//! `tools.json`. That file is the Node `tools/list` answer, written once and compared with the
//! Node server in a test, so a Claude session sees the same tools from either server.

use std::sync::LazyLock;

use serde_json::{Value, json};

static DEFS: LazyLock<Vec<Value>> =
    LazyLock::new(|| serde_json::from_str(include_str!("../tools.json")).unwrap_or_default());

/// The 23 tool definitions, in the order the Node server lists them.
#[must_use]
pub fn tool_defs() -> &'static [Value] {
    &DEFS
}

/// One definition by tool name.
#[must_use]
pub fn tool_def(name: &str) -> Option<&'static Value> {
    DEFS.iter().find(|d| d["name"] == name)
}

/// Whether the tool changes data (`annotations.readOnlyHint` is false).
#[must_use]
pub fn is_mutating(name: &str) -> bool {
    tool_def(name).is_some_and(|d| d["annotations"]["readOnlyHint"] != true)
}

/// The `serverInfo` of `initialize`. The name stays `coord-dashboard`, so every rendered
/// `.mcp.json` and permission rule keeps working.
#[must_use]
pub fn server_info() -> Value {
    json!({ "name": "coord-dashboard", "version": env!("CARGO_PKG_VERSION") })
}
