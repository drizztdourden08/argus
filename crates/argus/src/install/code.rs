//! The `.mcp.json` entry of DESIGN 7.3 for a folder ai-config does not render: the one every
//! rendered project has, which runs the `dashboard` shim.

use std::fs;
use std::path::Path;

use argus_core::ids::is_key;
use serde_json::{Value, json};

use super::desktop::SERVER_NAME;
use super::env::Platform;

/// The entry for project `key`.
#[must_use]
pub fn code_entry(key: &str, platform: Platform) -> Value {
    let server = if platform == Platform::Windows {
        json!({ "command": "cmd", "args": ["/c", "dashboard", "mcp", "--project", key] })
    } else {
        json!({ "command": "dashboard", "args": ["mcp", "--project", key] })
    };
    json!({ "mcpServers": { SERVER_NAME: server } })
}

/// The `projectKey` of the folder's `.ai-config.json`, when it has one.
#[must_use]
pub fn manifest_key(dir: &Path) -> Option<String> {
    let text = fs::read_to_string(dir.join(".ai-config.json")).ok()?;
    let manifest: Value = serde_json::from_str(&text).ok()?;
    manifest["projectKey"].as_str().map(str::to_owned)
}

/// The project key to use: the one given, else the folder's manifest; none when it is not a key.
#[must_use]
pub fn resolve_key(given: Option<&str>, dir: &Path) -> Option<String> {
    let key = given.map(str::to_owned).or_else(|| manifest_key(dir))?;
    is_key(&key).then_some(key)
}
