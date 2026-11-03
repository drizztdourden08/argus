//! The tool list and the `initialize` text compared with the Node server itself. The test
//! runs only where the Node dashboard is on disk (`ARGUS_NODE_DASHBOARD`, default
//! `X:/ai-config/dashboard`) and `node` is on the path.
//! E3: the tool list and initialize text against the Node server.
// test-tag: e2e

use std::path::PathBuf;
use std::process::Command;

use argus_mcp::{DEP_TOOLS, INSTRUCTIONS, PROTOCOL_VERSION, SUPPORTED_VERSIONS, tool_defs};
use serde_json::Value;

/// The tool definitions the Node server has: ours without the two dependency tools.
fn node_tools() -> Vec<&'static Value> {
    let ours = |d: &&Value| !d["name"].as_str().is_some_and(|n| DEP_TOOLS.contains(&n));
    tool_defs().iter().filter(ours).collect()
}

const SCRIPT: &str = r"
const { pathToFileURL } = require('node:url');
const dir = process.argv[1];
(async () => {
  const tools = await import(pathToFileURL(dir + '/src/mcp/tools.mjs').href);
  const proto = await import(pathToFileURL(dir + '/src/mcp/protocol.mjs').href);
  const view = (d) => ({ name: d.name, title: d.title, description: d.description, inputSchema: d.inputSchema, ...(d.outputSchema ? { outputSchema: d.outputSchema } : {}), annotations: d.annotations });
  console.log(JSON.stringify({ tools: tools.TOOL_DEFS.map(view), instructions: proto.INSTRUCTIONS, protocol: proto.PROTOCOL_VERSION, supported: proto.SUPPORTED_VERSIONS }));
})();
";

fn node_answer() -> Option<Value> {
    let dir = std::env::var("ARGUS_NODE_DASHBOARD")
        .unwrap_or_else(|_| "X:/ai-config/dashboard".to_owned());
    if !PathBuf::from(&dir).join("src/mcp/tools.mjs").exists() {
        return None;
    }
    let out = Command::new("node")
        .args(["-e", SCRIPT, &dir])
        .output()
        .ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

#[test]
#[ignore = "e2e"]
fn the_tool_list_and_the_initialize_text_are_the_node_ones() {
    let Some(node) = node_answer() else {
        return;
    };
    let ours = serde_json::to_string(&node_tools()).expect("JSON");
    assert_eq!(
        ours,
        node["tools"].to_string(),
        "tools/list differs from the Node server"
    );
    assert_eq!(INSTRUCTIONS, node["instructions"]);
    assert_eq!(PROTOCOL_VERSION, node["protocol"]);
    assert_eq!(serde_json::json!(SUPPORTED_VERSIONS), node["supported"]);
}

fn permissions() -> Option<Value> {
    let dir = std::env::var("ARGUS_NODE_DASHBOARD")
        .unwrap_or_else(|_| "X:/ai-config/dashboard".to_owned());
    let file = PathBuf::from(dir).join("../shared/permissions.json");
    serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()
}

fn rules(list: &Value) -> Vec<String> {
    let mut names: Vec<String> = list
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            r.as_str()?
                .strip_prefix("mcp__coord-dashboard__")
                .map(str::to_owned)
        })
        .collect();
    names.sort();
    names
}

fn names_where(keep: impl Fn(&Value) -> bool) -> Vec<String> {
    let mut names: Vec<String> = node_tools()
        .into_iter()
        .filter(|d| keep(d))
        .filter_map(|d| d["name"].as_str().map(str::to_owned))
        .collect();
    names.sort();
    names
}

#[test]
#[ignore = "e2e"]
fn the_permission_rules_allow_the_reads_and_the_ping_and_ask_for_the_writes() {
    let Some(perms) = permissions() else {
        return;
    };
    let ping = |d: &Value| d["name"] == "notify_owner";
    let read_or_ping = names_where(|d| d["annotations"]["readOnlyHint"] == true || ping(d));
    assert_eq!(
        rules(&perms["allow"]),
        read_or_ping,
        "allow rules are exactly the read-only tools and the ping"
    );
    let writes = names_where(|d| d["annotations"]["readOnlyHint"] != true && !ping(d));
    assert_eq!(
        rules(&perms["ask"]),
        writes,
        "ask rules are exactly the mutating tools that act as the owner"
    );
}
