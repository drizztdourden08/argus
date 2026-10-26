//! Store errors and their API codes.

use argus_core::{ApiError, ErrorCode};

/// What the store refuses or fails at.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Bad input (scope, collection, query, change).
    #[error("{0}")]
    BadRequest(String),
    /// The database stayed locked past `busy_timeout`.
    #[error("database is busy, retry shortly")]
    Busy,
    /// The database was written by a newer server.
    #[error("database schema {have} is newer than this dashboard ({known})")]
    Newer {
        /// Its `user_version`.
        have: i64,
        /// The migrations this build knows.
        known: usize,
    },
    /// Any other SQLite failure.
    #[error(transparent)]
    Sqlite(rusqlite::Error),
    /// A stored doc is not JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// The folder could not be made.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl StoreError {
    /// A 400 error.
    #[must_use]
    pub fn bad(message: impl Into<String>) -> Self {
        StoreError::BadRequest(message.into())
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        let busy = matches!(
            e.sqlite_error_code(),
            Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked)
        );
        if busy {
            StoreError::Busy
        } else {
            StoreError::Sqlite(e)
        }
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::BadRequest(m) => ApiError::bad_request(m),
            StoreError::Busy => ApiError::new(ErrorCode::DbBusy, StoreError::Busy.to_string()),
            _ => ApiError::internal(),
        }
    }
}
