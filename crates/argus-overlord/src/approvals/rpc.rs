//! The JSON-RPC of the private endpoint: `initialize`, `ping`, `tools/list` and the one tool,
//! `permission_prompt`. Only a `permission_prompt` call needs more than a canned answer.

use serde_json::{Value, json};

const PROTOCOL: &str = "2025-06-18";
const TOOL: &str = "permission_prompt";

/// What the CLI sends to `permission_prompt`.
#[derive(Debug, Clone, PartialEq)]
pub struct Prompt {
    pub id: Value,
    pub tool_name: String,
    pub input: Value,
    pub tool_use_id: Option<String>,
}

/// What one request body asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Parsed {
    /// Answer at once with this JSON-RPC message.
    Now(Value),
    /// A notification: answer 202 and nothing.
    Quiet,
    /// A permission question to put to the owner.
    Ask(Prompt),
}

fn result(id: &Value, body: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": body })
}

fn failure(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn listing() -> Value {
    json!({ "tools": [{
        "name": TOOL,
        "description": "Asks the owner whether a tool call may run.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tool_name": { "type": "string" },
                "input": { "type": "object" },
                "tool_use_id": { "type": "string" }
            },
            "required": ["tool_name", "input"]
        }
    }] })
}

fn initialize() -> Value {
    json!({
        "protocolVersion": PROTOCOL,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "argus-overlord", "version": "1" }
    })
}

fn ask(id: &Value, params: &Value) -> Parsed {
    let args = &params["arguments"];
    let named = params["name"].as_str() == Some(TOOL);
    let Some(tool_name) = args["tool_name"].as_str().filter(|_| named) else {
        return Parsed::Now(failure(id, -32602, "unknown tool or missing tool_name"));
    };
    Parsed::Ask(Prompt {
        id: id.clone(),
        tool_name: tool_name.to_owned(),
        input: args.get("input").cloned().unwrap_or_else(|| json!({})),
        tool_use_id: args["tool_use_id"].as_str().map(str::to_owned),
    })
}

/// Reads one request body.
#[must_use]
pub fn parse(body: &[u8]) -> Parsed {
    let Ok(message) = serde_json::from_slice::<Value>(body) else {
        return Parsed::Now(failure(&Value::Null, -32700, "not JSON"));
    };
    let Some(id) = message.get("id").cloned() else {
        return Parsed::Quiet;
    };
    match message["method"].as_str() {
        Some("initialize") => Parsed::Now(result(&id, initialize())),
        Some("ping") => Parsed::Now(result(&id, json!({}))),
        Some("tools/list") => Parsed::Now(result(&id, listing())),
        Some("tools/call") => ask(&id, &message["params"]),
        _ => Parsed::Now(failure(&id, -32601, "method not found")),
    }
}

/// The JSON-RPC answer of a `permission_prompt` call: the reply as the tool's text content.
#[must_use]
pub fn tool_result(id: &Value, reply: &Value) -> Value {
    result(
        id,
        json!({ "content": [{ "type": "text", "text": reply.to_string() }] }),
    )
}
