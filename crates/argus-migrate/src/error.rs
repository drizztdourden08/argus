//! What a migration run can refuse.

use std::io;

use argus_core::boards::BoardsError;
use thiserror::Error;

/// A failure of the migration.
#[derive(Debug, Error)]
pub enum Error {
    /// A file or folder cannot be read, written or renamed.
    #[error("{0}")]
    Io(#[from] io::Error),
    /// A file is not valid JSON.
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    /// The database refused.
    #[error("database: {0}")]
    Sql(#[from] rusqlite::Error),
    /// The identity table refused.
    #[error("{0}")]
    Boards(#[from] BoardsError),
    /// A precondition or a check failed; nothing was changed past this point.
    #[error("{0}")]
    Stop(String),
    /// A fault injected by a test.
    #[error("injected fault: {0}")]
    Fault(String),
}

/// The result of a migration step.
pub type Result<T> = std::result::Result<T, Error>;

/// Ends the run with a message.
pub fn stop<T>(message: impl Into<String>) -> Result<T> {
    Err(Error::Stop(message.into()))
}
