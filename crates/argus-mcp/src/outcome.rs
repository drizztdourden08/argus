//! The answer of a tool call before it is framed as JSON-RPC.

use serde_json::Value;

use crate::error::ToolError;

/// A tool's result and whether it counts as an error (`status` refused or error, or a failure).
#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutcome {
    /// The result object.
    pub result: Value,
    /// `isError` of the MCP result.
    pub is_error: bool,
}

impl ToolOutcome {
    /// A failed call: `{error: {...}}` with `isError` set.
    #[must_use]
    pub fn failed(error: &ToolError) -> Self {
        Self {
            result: error.result(),
            is_error: true,
        }
    }

    /// A result: an error only when the board refused the post or reported an error status.
    #[must_use]
    pub fn of(result: Value) -> Self {
        let is_error = matches!(result["status"].as_str(), Some("refused" | "error"));
        Self { result, is_error }
    }
}
