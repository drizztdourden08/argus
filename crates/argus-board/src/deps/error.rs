//! How a dependency call fails, and the exact refusal texts of the contract.

use crate::error::BoardError;

/// A dependency call failed. The server answers 400, 404 or 409 with the message.
#[derive(Debug, thiserror::Error)]
pub enum DepError {
    /// The input is wrong (400).
    #[error("{0}")]
    Bad(String),
    /// Something named does not exist (404).
    #[error("{0}")]
    Unknown(String),
    /// A rule is broken (409).
    #[error("{0}")]
    Rule(String),
    /// A board could not be locked, read or written.
    #[error(transparent)]
    Board(#[from] BoardError),
}

impl DepError {
    /// The HTTP status the server answers with.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            DepError::Bad(_) => 400,
            DepError::Unknown(_) | DepError::Board(BoardError::Missing(_)) => 404,
            DepError::Rule(_) | DepError::Board(BoardError::Refused(_)) => 409,
            DepError::Board(_) => 500,
        }
    }
}

/// A refusal of a text check is bad input; any other board error stays what it is.
pub(crate) fn bad(e: BoardError) -> DepError {
    match e {
        BoardError::Refused(message) => DepError::Bad(message),
        other => DepError::Board(other),
    }
}

/// `dependencies are off (deps.enabled is false in config.json)`; the server answers 404.
#[must_use]
pub fn off() -> DepError {
    DepError::Unknown("dependencies are off (deps.enabled is false in config.json)".to_owned())
}

pub(crate) fn not_a_board(target: &str, known: &[String]) -> DepError {
    let listed = if known.is_empty() {
        "none".to_owned()
    } else {
        known.join(", ")
    };
    DepError::Unknown(format!("{target} is not a board here: known {listed}"))
}

pub(crate) fn same_project() -> DepError {
    DepError::Rule("a dependency stays inside one project: use coord item add --for".to_owned())
}

pub(crate) fn not_open(node: &str, status: &str) -> DepError {
    DepError::Rule(format!(
        "{node} is {status}: only an open item can wait on another board"
    ))
}

pub(crate) fn already_waits(node: &str, other: &str, dep: &str) -> DepError {
    DepError::Rule(format!(
        "{node} already waits on {other} ({dep}): cancel it first"
    ))
}

pub(crate) fn already_requested(node: &str, by: &str, dep: &str) -> DepError {
    DepError::Rule(format!("{node} is already requested by {by} ({dep})"))
}

pub(crate) fn closes_loop(path: &str) -> DepError {
    DepError::Rule(format!("this would close a loop: {path}"))
}

pub(crate) fn no_lead(target: &str) -> DepError {
    DepError::Rule(format!(
        "{target} has no live lead; name a thread with --to"
    ))
}

pub(crate) fn thread_not_live(thread: &str, target: &str, live: &[String]) -> DepError {
    DepError::Rule(format!(
        "thread {thread} is not live on {target}: live threads {}",
        live.join(", ")
    ))
}

pub(crate) fn blocked_needs_reason() -> DepError {
    DepError::Bad("--as blocked needs --reason".to_owned())
}

pub(crate) fn wrong_state(dep: &str, state: &str, verb: &str, needs: [&str; 2]) -> DepError {
    DepError::Rule(format!(
        "{dep} is {state}: {verb} needs it {} or {}",
        needs[0], needs[1]
    ))
}

pub(crate) fn not_a_dependency_of(dep: &str, project: &str) -> DepError {
    DepError::Unknown(format!("{dep} is not a dependency of {project}"))
}
