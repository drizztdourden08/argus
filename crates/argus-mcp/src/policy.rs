//! The permission model of `config.mcp`: `deny` wins, `allow` narrows, `readOnly` refuses every
//! tool that changes data.

use argus_core::config::McpConfig;
use serde_json::Value;

use crate::defs::is_mutating;

fn allowed(allow: &Value, name: &str) -> bool {
    match allow {
        Value::String(s) if s == "*" => true,
        Value::Array(list) => list.iter().any(|v| v == name),
        _ => false,
    }
}

/// Why `name` may not be called under `policy`, or none when it may.
#[must_use]
pub fn refusal(policy: &McpConfig, name: &str) -> Option<String> {
    if policy.deny.iter().any(|d| d == name) {
        return Some(format!("{name} is denied by config.mcp.deny"));
    }
    if !allowed(&policy.allow, name) {
        return Some(format!("{name} is not in config.mcp.allow"));
    }
    if policy.read_only && is_mutating(name) {
        return Some(format!("{name} changes data and config.mcp.readOnly is on"));
    }
    None
}
