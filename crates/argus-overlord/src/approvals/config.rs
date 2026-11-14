//! The `--mcp-config` file of the child: the private endpoint and its bearer.

use serde_json::json;

/// The JSON naming the one server, `argus-overlord`, at `endpoint`, with the bearer header.
#[must_use]
pub fn mcp_config_json(endpoint: &str, bearer: &str) -> String {
    json!({ "mcpServers": { "argus-overlord": {
        "type": "http",
        "url": endpoint,
        "headers": { "Authorization": format!("Bearer {bearer}") }
    } } })
    .to_string()
}
