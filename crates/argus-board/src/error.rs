//! The ways a board call fails.

/// A board call failed.
#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    /// The board lock stayed taken by another coord call until the wait ran out.
    #[error("board {0} is locked by another coord call; retry.")]
    Locked(String),
    /// The project has no board file and the call may not create one.
    #[error("no board for project \"{0}\". Run `coord board init` first (coordinator).")]
    Missing(String),
    /// A validation or ownership rule refused the change; the text says why.
    #[error("{0}")]
    Refused(String),
    /// A file could not be read or written.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// A board file is not valid JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

/// A refusal with its message.
pub(crate) fn refuse(text: impl Into<String>) -> BoardError {
    BoardError::Refused(text.into())
}
