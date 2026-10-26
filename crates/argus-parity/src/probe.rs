//! A probe: one read-only question put to both servers. The only requests that exist are a GET
//! of a path and the two MCP calls that read (`initialize` and `tools/list`), so nothing here
//! can write.

use serde_json::{Value, json};

use crate::kind::Kind;

/// The MCP calls the comparison makes. Neither changes anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpCall {
    /// `initialize`.
    Initialize,
    /// `tools/list`.
    ToolsList,
}

impl McpCall {
    /// The JSON-RPC method.
    #[must_use]
    pub fn method(self) -> &'static str {
        match self {
            McpCall::Initialize => "initialize",
            McpCall::ToolsList => "tools/list",
        }
    }

    /// The JSON-RPC request.
    #[must_use]
    pub fn body(self) -> Value {
        let params = match self {
            McpCall::Initialize => json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "argus-parity", "version": "0" },
            }),
            McpCall::ToolsList => json!({}),
        };
        json!({ "jsonrpc": "2.0", "id": 1, "method": self.method(), "params": params })
    }
}

/// What a probe sends.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// A GET of a path and query.
    Get(String),
    /// A read-only MCP call on `/mcp`.
    Mcp(McpCall),
}

/// One question and the rules its two answers are compared by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Probe {
    /// The kind, which picks the rules.
    pub kind: Kind,
    /// The request.
    pub request: Request,
}

impl Probe {
    /// A GET probe.
    #[must_use]
    pub fn get(kind: Kind, path: impl Into<String>) -> Probe {
        Probe {
            kind,
            request: Request::Get(path.into()),
        }
    }

    /// The name of the route in a report.
    #[must_use]
    pub fn label(&self) -> String {
        match &self.request {
            Request::Get(path) => format!("GET {path}"),
            Request::Mcp(call) => format!("POST /mcp {}", call.method()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{McpCall, Probe};
    use crate::kind::Kind;

    #[test]
    fn labels_name_the_method_and_the_path() {
        assert_eq!(
            Probe::get(Kind::Health, "/api/health").label(),
            "GET /api/health"
        );
    }

    #[test]
    fn the_mcp_calls_are_the_two_reads() {
        assert_eq!(McpCall::Initialize.body()["method"], "initialize");
        assert_eq!(McpCall::ToolsList.body()["method"], "tools/list");
    }
}
