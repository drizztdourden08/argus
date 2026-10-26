//! The one error type of the crate. Services log it and carry on; none of them stops on a fault.

/// What a port can fail with.
#[derive(Debug, thiserror::Error)]
pub enum AgentsError {
    /// The board of a project could not be read or written.
    #[error("board {0}: {1}")]
    Board(String, String),
    /// A live agents doc could not be read or set.
    #[error("agents doc {0}: {1}")]
    Docs(String, String),
    /// A wake could not be handed to the engine.
    #[error("wake not sent: {0}")]
    Wake(String),
}
