//! Asks the owner settled in advance. Owner decision Q-24 (2026-11-23): "Allow coord commands
//! without asking". A Bash call that is one plain `coord` invocation and any tool of the
//! coord-dashboard MCP are allowed at once, with no queue entry and no toast.

use serde_json::Value;

use crate::shadow::SHELL;

const DASHBOARD: &str = "mcp__coord-dashboard__";

/// Whether the owner's standing answer allows this ask without queueing it.
#[must_use]
pub fn auto_allowed(tool: &str, input: &Value) -> bool {
    tool.starts_with(DASHBOARD)
        || (tool == "Bash" && input["command"].as_str().is_some_and(single_coord))
}

fn single_coord(command: &str) -> bool {
    let command = command.trim();
    let named = command == "coord" || command.starts_with("coord ");
    named && !command.contains(SHELL)
}
