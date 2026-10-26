//! The API error: a code from the Node table, a message and optional details.

use std::fmt;

use serde_json::{Map, Value, json};

/// The error codes of DESIGN 3 and the status each maps to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// 400.
    BadRequest,
    /// 401.
    Unauthorized,
    /// 403.
    Forbidden,
    /// 421.
    BadHost,
    /// 403.
    ReadOnly,
    /// 404.
    NotFound,
    /// 404.
    UnknownRun,
    /// 405.
    MethodNotAllowed,
    /// 409.
    Conflict,
    /// 413.
    TooLarge,
    /// 415.
    UnsupportedMedia,
    /// 503.
    BoardLocked,
    /// 503.
    DbBusy,
    /// 503.
    Unavailable,
    /// 500.
    Internal,
}

impl ErrorCode {
    /// The wire code.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCode::BadRequest => "bad_request",
            ErrorCode::Unauthorized => "unauthorized",
            ErrorCode::Forbidden => "forbidden",
            ErrorCode::BadHost => "bad_host",
            ErrorCode::ReadOnly => "read_only",
            ErrorCode::NotFound => "not_found",
            ErrorCode::UnknownRun => "unknown_run",
            ErrorCode::MethodNotAllowed => "method_not_allowed",
            ErrorCode::Conflict => "conflict",
            ErrorCode::TooLarge => "too_large",
            ErrorCode::UnsupportedMedia => "unsupported_media",
            ErrorCode::BoardLocked => "board_locked",
            ErrorCode::DbBusy => "db_busy",
            ErrorCode::Unavailable => "unavailable",
            ErrorCode::Internal => "internal",
        }
    }

    /// The HTTP status.
    #[must_use]
    pub fn status(self) -> u16 {
        match self {
            ErrorCode::BadRequest => 400,
            ErrorCode::Unauthorized => 401,
            ErrorCode::Forbidden | ErrorCode::ReadOnly => 403,
            ErrorCode::NotFound | ErrorCode::UnknownRun => 404,
            ErrorCode::MethodNotAllowed => 405,
            ErrorCode::Conflict => 409,
            ErrorCode::TooLarge => 413,
            ErrorCode::UnsupportedMedia => 415,
            ErrorCode::BadHost => 421,
            ErrorCode::Internal => 500,
            ErrorCode::BoardLocked | ErrorCode::DbBusy | ErrorCode::Unavailable => 503,
        }
    }
}

/// An error a route answers with.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    /// The code.
    pub code: ErrorCode,
    /// The message.
    pub message: String,
    /// Optional details.
    pub details: Option<Value>,
}

impl ApiError {
    /// A new error.
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// 400 `bad_request`.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }

    /// 404 `not_found`.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, message)
    }

    /// 503 `unavailable`.
    #[must_use]
    pub fn unavailable(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unavailable, message)
    }

    /// 500 `internal`, with the message the client sees.
    #[must_use]
    pub fn internal() -> Self {
        Self::new(ErrorCode::Internal, "internal error")
    }

    /// The same error with details.
    #[must_use]
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// `{"error": {"code", "message", "details"?}}`.
    #[must_use]
    pub fn body(&self) -> Value {
        let mut error = Map::new();
        error.insert("code".into(), json!(self.code.as_str()));
        error.insert("message".into(), json!(self.message));
        if let Some(d) = &self.details {
            error.insert("details".into(), d.clone());
        }
        json!({ "error": error })
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for ApiError {}
