//! The error a tool answers with: a code, a message and optional details, as the Node
//! `ToolError` carried them.

use serde_json::{Value, json};

/// A failed tool call or resource read.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolError {
    /// The wire code (`bad_request`, `not_found`, `unavailable`, ...).
    pub code: String,
    /// The message the client reads.
    pub message: String,
    /// Optional details.
    pub details: Option<Value>,
}

impl ToolError {
    /// A new error.
    #[must_use]
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_owned(),
            message: message.into(),
            details: None,
        }
    }

    /// `bad_request`.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("bad_request", message)
    }

    /// `not_found`.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }

    /// `unavailable`: the service behind the tool is not running.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new("unavailable", message)
    }

    /// The same error with details.
    #[must_use]
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// `{"error": {"code", "message", "details"?}}`, the result of a failed call.
    #[must_use]
    pub fn result(&self) -> Value {
        let mut error = json!({ "code": self.code, "message": self.message });
        if let (Some(details), Some(map)) = (&self.details, error.as_object_mut()) {
            map.insert("details".into(), details.clone());
        }
        json!({ "error": error })
    }
}

impl std::fmt::Display for ToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ToolError {}
