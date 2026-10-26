//! The service errors as the tools report them: the wire code and the message.

use argus_core::ApiError;
use argus_launchers::RunError;
use argus_mcp::ToolError;

/// A route-style error as a tool error, details kept.
pub fn api(e: ApiError) -> ToolError {
    let error = ToolError::new(e.code.as_str(), e.message);
    match e.details {
        Some(details) => error.with_details(details),
        None => error,
    }
}

/// A run service refusal as a tool error.
pub fn run(e: &RunError) -> ToolError {
    ToolError::new(e.code, e.message.clone())
}

/// A write refused because the server is in shadow mode.
pub fn read_only(message: &str) -> ToolError {
    ToolError::new("read_only", message)
}

/// An internal failure whose cause stays in the server log.
pub fn internal() -> ToolError {
    ToolError::new("internal", "internal error")
}
