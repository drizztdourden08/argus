//! The result text of the tools and resources compared with the Node tool host over the same
//! fixture, byte for byte. Runs only where the Node dashboard is on disk
//! (`ARGUS_NODE_DASHBOARD`, default `X:/ai-config/dashboard`) and `node` is on the path.
//! E3: result texts against the Node tool host.
// test-tag: e2e

use std::path::PathBuf;

use argus_core::process::hidden_command;

use argus_mcp::{CallContext, RpcHost};
use serde_json::{Value, json};

use crate::support::{Rig, ctx};

const SCRIPT: &str = r"
const { pathToFileURL } = require('node:url');
const dir = process.argv[1];
const plan = JSON.parse(process.argv[2]);
(async () => {
  const tools = await import(pathToFileURL(dir + '/src/mcp/tools.mjs').href);
  const stub = await import(pathToFileURL(dir + '/test/helpers/mcp-stub.mjs').href);
  const host = tools.createToolHost(stub.makeSvc());
  const out = [];
  for (const [name, args, defaultProject] of plan.calls) {
    const r = await host.callTool(name, args, { defaultProject, client: 'tester', baseUrl: 'http://127.0.0.1:47615' });
    out.push(JSON.stringify(r.result) + '|' + r.isError);
  }
  for (const uri of plan.uris) {
    try { out.push(JSON.stringify(await host.readResource(uri))); } catch (e) { out.push('error:' + e.code + ':' + e.message); }
  }
  console.log(JSON.stringify(out));
})();
";

type Call = (&'static str, Value, Option<&'static str>);

fn alpha(extra: &Value) -> Value {
    let mut args = json!({ "project": "alpha" });
    args.as_object_mut()
        .expect("args")
        .extend(extra.as_object().expect("extra").clone());
    args
}

fn reads() -> Vec<Call> {
    vec![
        ("portfolio_overview", json!({}), None),
        ("board_summary", alpha(&json!({})), None),
        ("board_summary", json!({ "project": "beta" }), None),
        ("list_threads", alpha(&json!({})), None),
        (
            "list_threads",
            alpha(&json!({ "includeHidden": true })),
            None,
        ),
        ("list_items", alpha(&json!({})), None),
        ("list_items", alpha(&json!({ "status": "todo" })), None),
        (
            "list_items",
            alpha(&json!({ "thread": "story", "includeArchived": true })),
            None,
        ),
        ("list_items", alpha(&json!({ "limit": 1 })), None),
        ("list_items", json!({}), Some("beta")),
        ("get_item", alpha(&json!({ "id": "I-1" })), None),
        ("get_item", alpha(&json!({ "id": "I-99" })), None),
        ("list_tests", alpha(&json!({})), None),
        ("list_tests", alpha(&json!({ "mode": "manual" })), None),
        (
            "list_tests",
            alpha(&json!({ "build": "b2", "status": "pending" })),
            None,
        ),
        ("get_test", alpha(&json!({ "id": "T-1" })), None),
        ("get_test", alpha(&json!({ "id": "T-2" })), None),
    ]
}

fn lists() -> Vec<Call> {
    vec![
        ("list_agents", alpha(&json!({})), None),
        ("list_agents", alpha(&json!({ "live": true })), None),
        ("list_agents", alpha(&json!({ "thread": "story" })), None),
        ("list_entries", alpha(&json!({})), None),
        ("list_entries", alpha(&json!({ "open": true })), None),
        ("list_entries", alpha(&json!({ "kind": "question" })), None),
        ("list_entries", alpha(&json!({ "kind": "edit" })), None),
        ("search", json!({ "query": "pedestal" }), None),
        (
            "search",
            json!({ "query": "pedestal", "project": "beta" }),
            None,
        ),
        (
            "search",
            json!({ "query": "glow shader", "project": "alpha", "collections": ["log"] }),
            None,
        ),
        ("search", json!({ "query": "Archived scene" }), None),
        ("search", json!({ "query": "scene" }), None),
        ("search", json!({ "query": "the", "limit": 5 }), None),
        ("search", json!({ "query": "zzz nothing" }), None),
        (
            "search",
            json!({ "query": "glow", "project": "nope" }),
            None,
        ),
        ("run_status", alpha(&json!({})), None),
        ("run_status", alpha(&json!({ "status": "running" })), None),
        ("run_status", alpha(&json!({ "run": "R-1" })), None),
        ("run_status", alpha(&json!({ "run": "R-9" })), None),
        ("list_launchers", json!({ "project": "beta" }), None),
        ("run_logs", alpha(&json!({ "run": "R-1" })), None),
        (
            "run_logs",
            alpha(&json!({ "run": "R-1", "after": 1 })),
            None,
        ),
        ("run_logs", alpha(&json!({ "run": "R-9" })), None),
    ]
}

fn errors() -> Vec<Call> {
    vec![
        ("list_items", json!({}), None),
        ("list_items", json!({ "project": "gamma" }), None),
        ("list_items", json!({ "project": "Bad Key" }), None),
        (
            "list_items",
            alpha(&json!({ "limit": 9999, "status": "weird", "surprise": 1 })),
            None,
        ),
        ("get_item", alpha(&json!({})), None),
        ("get_item", alpha(&json!({ "id": "x" })), None),
        ("no_such_tool", json!({}), None),
    ]
}

fn writes() -> Vec<Call> {
    vec![
        ("post_entry", alpha(&json!({ "kind": "followup" })), None),
        ("post_entry", alpha(&json!({ "kind": "pause" })), None),
        (
            "answer",
            alpha(&json!({ "entry": "Q-77", "text": "x" })),
            None,
        ),
        (
            "record_test_result",
            alpha(&json!({ "test": "T-1", "outcome": "fail" })),
            None,
        ),
        (
            "record_test_result",
            alpha(&json!({ "test": "T-99", "outcome": "pass" })),
            None,
        ),
        ("run_start", alpha(&json!({ "name": "nope" })), None),
        ("run_stop", alpha(&json!({ "run": "R-2" })), None),
        (
            "send_to_lead",
            json!({ "project": "beta", "text": "hello" }),
            None,
        ),
        ("send_to_lead", json!({ "text": "x" }), None),
        (
            "send_to_lead",
            json!({ "project": "zzz", "text": "x" }),
            None,
        ),
        (
            "attach_proof",
            alpha(&json!({ "test": "T-1", "caption": "c" })),
            None,
        ),
        (
            "attach_proof",
            alpha(&json!({ "test": "T-1", "caption": "c", "base64": "###" })),
            None,
        ),
        (
            "attach_proof",
            alpha(&json!({ "caption": "c", "base64": "aGVsbG8=" })),
            None,
        ),
        (
            "attach_proof",
            alpha(&json!({ "test": "T-1", "caption": "c", "base64": "aGVsbG8=" })),
            None,
        ),
        (
            "attach_proof",
            alpha(&json!({ "test": "T-1", "caption": "c", "path": "relative.png" })),
            None,
        ),
        ("notify_owner", json!({ "text": "hi" }), None),
    ]
}

fn calls() -> Vec<Call> {
    [reads(), lists(), errors(), writes()].concat()
}

const URIS: [&str; 8] = [
    "portfolio://overview",
    "board://alpha/summary",
    "board://alpha/tests/to-run",
    "runs://alpha/active",
    "runs://alpha/R-1/log",
    "board://nope/summary",
    "board://alpha/other",
    "garbage",
];

fn node_output(calls: &[(&str, Value, Option<&str>)]) -> Option<Vec<String>> {
    let dir = std::env::var("ARGUS_NODE_DASHBOARD")
        .unwrap_or_else(|_| "X:/ai-config/dashboard".to_owned());
    if !PathBuf::from(&dir)
        .join("test/helpers/mcp-stub.mjs")
        .exists()
    {
        return None;
    }
    let plan = json!({ "calls": calls, "uris": URIS });
    let out = hidden_command("node")
        .args(["-e", SCRIPT, &dir, &plan.to_string()])
        .output()
        .ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

#[tokio::test]
#[ignore = "e2e"]
async fn every_result_text_is_the_node_one() {
    let plan = calls();
    let Some(node) = node_output(&plan) else {
        return;
    };
    let rig = Rig::new();
    let mut ours = Vec::new();
    for (name, args, default_project) in &plan {
        let context = CallContext {
            default_project: default_project.map(str::to_owned),
            ..ctx()
        };
        let out = rig.call_as(name, args.clone(), &context).await;
        ours.push(format!("{}|{}", out.result, out.is_error));
    }
    for uri in URIS {
        ours.push(match rig.host.read_resource(uri, &ctx()).await {
            Ok(v) => v.to_string(),
            Err(e) => format!("error:{}:{}", e.code, e.message),
        });
    }
    assert_eq!(ours.len(), node.len());
    for (i, (mine, theirs)) in ours.iter().zip(&node).enumerate() {
        let label = plan.get(i).map_or_else(
            || URIS[i - plan.len()].to_owned(),
            |c| format!("{} {}", c.0, c.1),
        );
        assert_eq!(mine, theirs, "{label}");
    }
}
