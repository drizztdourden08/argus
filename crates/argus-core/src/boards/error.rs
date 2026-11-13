//! What the identity table can refuse.

use std::io;

use thiserror::Error;

/// A failure of the identity table.
#[derive(Debug, Error)]
pub enum BoardsError {
    /// The table or its lock cannot be read or written.
    #[error("boards table: {0}")]
    Io(#[from] io::Error),
    /// The table is not valid JSON of the expected shape.
    #[error("boards table is not valid: {0}")]
    Parse(#[from] serde_json::Error),
    /// The lock stayed held past the wait.
    #[error("boards table is locked")]
    Locked,
    /// No board has this id.
    #[error("unknown board id {0}")]
    Unknown(String),
    /// The key is not a project key, or an id or alias already holds it.
    #[error("board key {0} is not usable")]
    BadKey(String),
    /// The name is empty or longer than 30 characters.
    #[error("board name must be 1 to 30 characters")]
    BadName,
    /// No free id was found.
    #[error("no free board id")]
    Exhausted,
}
