//! The errors the monitor answers with, each carrying the HTTP-style code the server maps.

/// What kind of refusal this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    NotFound,
    Forbidden,
    Conflict,
    Internal,
}

impl ErrorCode {
    /// The code as the dashboard names it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::NotFound => "not_found",
            Self::Forbidden => "forbidden",
            Self::Conflict => "conflict",
            Self::Internal => "internal",
        }
    }
}

/// A refusal with its code and a message fit for the owner.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct ProcsError {
    code: ErrorCode,
    message: String,
}

impl ProcsError {
    fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }

    /// The kind of refusal.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// The message, without the code.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
