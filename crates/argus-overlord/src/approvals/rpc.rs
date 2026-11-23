//! The JSON-RPC of the private endpoint: `initialize`, `ping`, `tools/list` and the one tool,
//! `permission_prompt`. Only a `permission_prompt` call needs more than a canned answer.

use serde_json::{Value, json};

use super::auto::auto_allowed;
use crate::tools::{self, Call, parse_call, text_result};

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
    /// A question the owner settled in advance (I-198): allow it at once, queue nothing.
    Allowed(Prompt),
    /// A call of one of the Overlord's own tools; `id` is the request id to answer.
    Tool { id: Value, call: Call },
}

fn result(id: &Value, body: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": body })
}

fn failure(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn listing() -> Value {
    let mut all = vec![json!({
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
    })];
    all.extend(tools::specs());
    json!({ "tools": all })
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
    if params["name"].as_str() != Some(TOOL) {
        return own_tool(id, params["name"].as_str().unwrap_or_default(), args);
    }
    let Some(tool_name) = args["tool_name"].as_str() else {
        return Parsed::Now(failure(id, -32602, "unknown tool or missing tool_name"));
    };
    let prompt = Prompt {
        id: id.clone(),
        tool_name: tool_name.to_owned(),
        input: args.get("input").cloned().unwrap_or_else(|| json!({})),
        tool_use_id: args["tool_use_id"].as_str().map(str::to_owned),
    };
    if auto_allowed(&prompt.tool_name, &prompt.input) {
        Parsed::Allowed(prompt)
    } else {
        Parsed::Ask(prompt)
    }
}

fn own_tool(id: &Value, name: &str, args: &Value) -> Parsed {
    match parse_call(name, args) {
        Some(Ok(call)) => Parsed::Tool {
            id: id.clone(),
            call,
        },
        Some(Err(sentence)) => Parsed::Now(text_result(id, &sentence, true)),
        None => Parsed::Now(failure(id, -32602, "unknown tool or missing tool_name")),
    }
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
        Some("initialize") => Parsed::Now(result(&id, &initialize())),
        Some("ping") => Parsed::Now(result(&id, &json!({}))),
        Some("tools/list") => Parsed::Now(result(&id, &listing())),
        Some("tools/call") => ask(&id, &message["params"]),
        _ => Parsed::Now(failure(&id, -32601, "method not found")),
    }
}

/// The JSON-RPC answer of a `permission_prompt` call: the reply as the tool's text content.
#[must_use]
pub fn tool_result(id: &Value, reply: &Value) -> Value {
    result(
        id,
        &json!({ "content": [{ "type": "text", "text": reply.to_string() }] }),
    )
}
