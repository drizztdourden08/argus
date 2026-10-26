//! The one error the run subsystem returns. `status` and `code` follow the HTTP error table
//! of DESIGN.md section 3 so a route or the MCP layer can pass them straight on.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunError {
    pub status: u16,
    pub code: &'static str,
    pub message: String,
}

impl RunError {
    pub fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, "bad_request", message)
    }

    pub fn unknown_run(message: impl Into<String>) -> Self {
        Self::new(404, "unknown_run", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(409, "conflict", message)
    }

    pub fn too_large(message: impl Into<String>) -> Self {
        Self::new(413, "too_large", message)
    }

    pub fn unsupported_media(message: impl Into<String>) -> Self {
        Self::new(415, "unsupported_media", message)
    }

    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(503, "unavailable", message)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for RunError {}
