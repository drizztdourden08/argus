//! The permission model of `config.mcp`: deny wins, allow narrows, readOnly refuses writes.

use argus_core::config::McpConfig;
use argus_mcp::tool_defs;
use serde_json::{Map, Value, json};

use crate::support::Rig;

fn policy(allow: Value, deny: &[&str], read_only: bool) -> McpConfig {
    McpConfig {
        allow,
        deny: deny.iter().map(|d| (*d).to_owned()).collect(),
        read_only,
        extra: Map::new(),
    }
}

fn message(error: &Value) -> &str {
    error["message"].as_str().unwrap_or_default()
}

#[tokio::test]
async fn deny_wins_and_allow_narrows() {
    let rig = &Rig::new();
    rig.backend
        .set_policy(policy(json!("*"), &["search"], false));
    let denied = rig
        .err("search", json!({ "query": "pedestal" }), "forbidden")
        .await;
    assert_eq!(message(&denied), "search is denied by config.mcp.deny");
    rig.ok("health", json!({})).await;

    rig.backend
        .set_policy(policy(json!(["health", "board_summary"]), &[], false));
    let narrowed = rig
        .err("list_items", json!({ "project": "alpha" }), "forbidden")
        .await;
    assert_eq!(message(&narrowed), "list_items is not in config.mcp.allow");
    rig.ok("board_summary", json!({ "project": "alpha" })).await;
}

#[tokio::test]
async fn read_only_refuses_every_mutating_tool_and_none_of_the_reads() {
    let rig = &Rig::new();
    rig.backend.set_policy(policy(json!("*"), &[], true));
    for def in tool_defs() {
        let name = def["name"].as_str().expect("a name");
        let out = rig.call(name, json!({ "project": "alpha" })).await;
        let code = out.result["error"]["code"].clone();
        if def["annotations"]["readOnlyHint"] == true {
            assert_ne!(code, "forbidden", "{name}");
        } else {
            assert!(out.is_error, "{name}");
            assert_eq!(code, "forbidden", "{name}");
            let want = format!("{name} changes data and config.mcp.readOnly is on");
            assert_eq!(message(&out.result["error"]), want);
        }
    }
    let calls = rig.backend.calls();
    assert_eq!(
        calls.post.len() + calls.run_start.len() + calls.send.len(),
        0,
        "nothing was written"
    );
}

#[tokio::test]
async fn the_policy_is_read_on_every_call() {
    let rig = &Rig::new();
    rig.backend.set_policy(policy(json!("*"), &[], true));
    let e = rig
        .err(
            "send_to_lead",
            json!({ "project": "beta", "text": "blocked" }),
            "forbidden",
        )
        .await;
    assert!(message(&e).contains("config.mcp.readOnly"));
    rig.backend.set_policy(policy(json!("*"), &[], false));
    rig.ok(
        "send_to_lead",
        json!({ "project": "beta", "text": "now fine" }),
    )
    .await;
}
