//! The MCP streamable HTTP transport as pure functions, stateless: `POST /mcp` takes one
//! JSON-RPC message and answers `application/json`, a notification gets 202, and there is no
//! server-initiated stream, so `GET` and `DELETE` are 405. The server mounts these behind its
//! own loopback, Host, Origin, content-type and bearer checks.

use argus_core::ids::is_key;
use serde_json::{Value, json};

use crate::context::CallContext;
use crate::error::ToolError;
use crate::rpc::{RpcHost, SUPPORTED_VERSIONS, code, handle_rpc, rpc_error};

/// The largest body the endpoint reads (design section 8).
pub const MAX_BYTES: usize = 2 * 1024 * 1024;

/// What the endpoint needs of one `POST /mcp`.
#[derive(Debug, Clone, Copy, Default)]
pub struct McpRequest<'a> {
    /// `MCP-Protocol-Version`.
    pub protocol_version: Option<&'a str>,
    /// `X-Coord-Default-Project`.
    pub default_project: Option<&'a str>,
    /// `X-Coord-Client`.
    pub client: Option<&'a str>,
    /// `User-Agent`, the client name when `X-Coord-Client` is absent.
    pub user_agent: Option<&'a str>,
    /// `Host`, the base of proof urls.
    pub host: Option<&'a str>,
    /// The raw body.
    pub body: &'a [u8],
}

/// The answer to one request.
#[derive(Debug, Clone, PartialEq)]
pub enum McpReply {
    /// A JSON-RPC response with its status.
    Json {
        /// 200, or 400 for a body that is not one JSON-RPC message.
        status: u16,
        /// The JSON-RPC response.
        body: Value,
    },
    /// A notification: 202 and no body.
    Accepted,
    /// A refusal outside JSON-RPC: the standard error body with its status.
    Refused {
        /// The HTTP status.
        status: u16,
        /// The error to write as `{error: {code, message, details?}}`.
        error: ToolError,
        /// The `Allow` header, for a 405.
        allow: Option<&'static str>,
    },
}

impl McpReply {
    /// The HTTP status.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            McpReply::Json { status, .. } | McpReply::Refused { status, .. } => *status,
            McpReply::Accepted => 202,
        }
    }

    /// The JSON body, none for a 202.
    #[must_use]
    pub fn body(&self) -> Option<Value> {
        match self {
            McpReply::Json { body, .. } => Some(body.clone()),
            McpReply::Refused { error, .. } => Some(error.result()),
            McpReply::Accepted => None,
        }
    }
}

fn client_name(req: &McpRequest) -> String {
    let named = req.client.filter(|c| !c.is_empty());
    let agent = req
        .user_agent
        .and_then(|ua| ua.split(|c: char| c.is_whitespace() || c == '/').next())
        .filter(|c| !c.is_empty());
    named.or(agent).unwrap_or("client").to_owned()
}

fn context(req: &McpRequest) -> CallContext {
    CallContext {
        default_project: req.default_project.filter(|p| is_key(p)).map(str::to_owned),
        client: client_name(req),
        base_url: req
            .host
            .filter(|h| !h.is_empty())
            .map(|h| format!("http://{h}")),
    }
}

fn unsupported_version(version: &str) -> McpReply {
    let error = ToolError::bad_request(format!("unsupported MCP-Protocol-Version {version}"))
        .with_details(json!({ "supported": SUPPORTED_VERSIONS }));
    McpReply::Refused {
        status: 400,
        error,
        allow: None,
    }
}

fn bad_body(code: i64, message: &str) -> McpReply {
    McpReply::Json {
        status: 400,
        body: rpc_error(None, code, message),
    }
}

/// Answer one `POST /mcp`.
pub async fn handle_post(host: &dyn RpcHost, req: &McpRequest<'_>) -> McpReply {
    if let Some(v) = req.protocol_version.filter(|v| !v.is_empty()) {
        if !SUPPORTED_VERSIONS.contains(&v) {
            return unsupported_version(v);
        }
    }
    if req.body.len() > MAX_BYTES {
        return McpReply::Refused {
            status: 413,
            error: ToolError::new("too_large", "body too large"),
            allow: None,
        };
    }
    let text = String::from_utf8_lossy(req.body);
    let Ok(msg) = serde_json::from_str::<Value>(&text) else {
        return bad_body(code::PARSE, "body is not valid JSON");
    };
    if msg.is_array() {
        return bad_body(code::INVALID, "send one JSON-RPC message per request");
    }
    match handle_rpc(host, &msg, &context(req)).await {
        None => McpReply::Accepted,
        Some(body) => McpReply::Json { status: 200, body },
    }
}

/// The answer to `GET` and `DELETE`: 405, `Allow: POST`.
#[must_use]
pub fn handle_other() -> McpReply {
    McpReply::Refused {
        status: 405,
        error: ToolError::bad_request(
            "this MCP endpoint takes POST only (no server-initiated stream)",
        ),
        allow: Some("POST"),
    }
}
