//! Shared pieces of the MCP tests: the fixture boards of the Node `mcp-stub`, a rig of one
//! backend and one tool host, and the call helpers.

pub mod stub;

use std::sync::Arc;

use argus_mcp::{CallContext, MemoryBackend, RpcHost, ToolHost, ToolOutcome};
use base64::Engine;
use serde_json::{Value, json};

const T0: &str = "2026-10-22T10:00:00.000Z";

/// A 1x1 PNG, enough for the magic bytes.
pub fn png() -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==")
        .expect("a valid PNG")
}

/// The PNG as base64, the way a client sends it.
pub fn png_base64() -> String {
    base64::engine::general_purpose::STANDARD.encode(png())
}

/// The fixture boards `alpha`, `beta` and the portfolio cards.
pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixture.json")).expect("fixture.json is JSON")
}

fn run_row(
    id: &str,
    name: &str,
    status: &str,
    agent: Option<&str>,
    origin: &str,
    by: &str,
) -> Value {
    json!({ "id": id, "scope": "alpha", "name": name, "kind": "task", "status": status,
        "exitCode": if status == "exited" { json!(0) } else { Value::Null },
        "agent": agent, "origin": origin, "by": by, "cmd": "node", "cwd": "/x" })
}

/// The two runs of the stub: R-1 exited, R-2 running.
pub fn run_rows() -> Vec<Value> {
    vec![
        run_row("R-1", "hello", "exited", Some("A-1"), "cli", "coordinator"),
        run_row("R-2", "slow", "running", None, "page", "human"),
    ]
}

/// The two log lines every run shares.
pub fn run_lines() -> Vec<Value> {
    ["hi\n", "there\n"]
        .iter()
        .zip(1..)
        .map(|(text, seq)| json!({ "seq": seq, "stream": "out", "at": T0, "text": text }))
        .collect()
}

/// The run definition the sync service lists, with a secret env value.
pub fn run_defs() -> Vec<Value> {
    vec![
        json!({ "name": "hello", "cmd": "node -e \"1\"", "cwd": "/x", "kind": "task",
        "env": { "SECRET_TOKEN": "abc" }, "hand": false, "visible": false,
        "timeoutSec": 0, "parallel": 1 }),
    ]
}

/// The fixture backend with the runs service up.
pub fn memory() -> MemoryBackend {
    MemoryBackend::new(fixture())
        .with_runs(run_rows(), run_lines())
        .with_run_defs(run_defs())
}

/// The context of the Node tool tests: client `tester`, proof urls on port 47615.
pub fn ctx() -> CallContext {
    CallContext {
        default_project: None,
        client: "tester".to_owned(),
        base_url: Some("http://127.0.0.1:47615".to_owned()),
    }
}

/// One backend and one host over it.
pub struct Rig {
    pub backend: Arc<MemoryBackend>,
    pub host: ToolHost,
}

impl Rig {
    pub fn new() -> Self {
        Self::over(memory())
    }

    pub fn over(backend: MemoryBackend) -> Self {
        let backend = Arc::new(backend);
        let host = ToolHost::new(backend.clone());
        Self { backend, host }
    }

    pub async fn call(&self, name: &str, args: Value) -> ToolOutcome {
        self.call_as(name, args, &ctx()).await
    }

    pub async fn call_as(&self, name: &str, args: Value, ctx: &CallContext) -> ToolOutcome {
        self.host.call_tool(name, &args, ctx).await
    }

    pub async fn ok(&self, name: &str, args: Value) -> Value {
        self.ok_as(name, args, &ctx()).await
    }

    pub async fn ok_as(&self, name: &str, args: Value, ctx: &CallContext) -> Value {
        let out = self.call_as(name, args, ctx).await;
        assert!(!out.is_error, "{name} failed: {}", out.result);
        out.result
    }

    pub async fn err(&self, name: &str, args: Value, code: &str) -> Value {
        self.err_as(name, args, code, &ctx()).await
    }

    pub async fn err_as(&self, name: &str, args: Value, code: &str, ctx: &CallContext) -> Value {
        let out = self.call_as(name, args, ctx).await;
        assert!(out.is_error, "{name} should fail");
        let error = out.result["error"].clone();
        assert_eq!(error["code"], code, "{name}: {}", error["message"]);
        error
    }
}

/// The `id` of every row of `value[field]`.
pub fn ids(value: &Value, field: &str) -> Vec<String> {
    value[field]
        .as_array()
        .expect("an array")
        .iter()
        .map(|row| row["id"].as_str().expect("an id").to_owned())
        .collect()
}

/// The same ids in sorted order.
pub fn sorted_ids(value: &Value, field: &str) -> Vec<String> {
    let mut all = ids(value, field);
    all.sort();
    all
}

/// A `tools/call` request.
pub fn call(id: u32, name: &str, arguments: Value) -> Value {
    let mut msg =
        json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": name } });
    msg["params"]["arguments"] = arguments;
    msg
}
