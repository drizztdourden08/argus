//! How a verb ends: the error type of every verb and its map to an exit code.
//!
//! | Code | Meaning |
//! |---|---|
//! | 0 | success |
//! | 1 | a refusal: stderr `coord: <message>` |
//! | any | [`CoordError::Exit`]: the verb printed its own lines and picked the code (2 for a heard wake, 3 for a dashboard that will not come up, a run's own code) |
//! | 120 | [`NOT_PORTED`]: the verb is not served here, the shim runs Node |

use std::fmt;
use std::io::Write;

use argus_board::BoardError;

use crate::client::ClientError;

/// The code that tells the shim "not ported here": returned before any read of stdin or any file.
pub const NOT_PORTED: i32 = 120;

/// Why a verb ended without success.
#[derive(Debug)]
pub enum CoordError {
    /// A refusal: the router prints `coord: <message>` on stderr and exits 1.
    Refused(String),
    /// The verb printed what it had to say and ends with this code, silently.
    Exit(i32),
}

impl CoordError {
    /// A refusal with its message.
    #[must_use]
    pub fn refuse(message: impl Into<String>) -> Self {
        CoordError::Refused(message.into())
    }
}

impl fmt::Display for CoordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoordError::Refused(message) => f.write_str(message),
            CoordError::Exit(code) => write!(f, "exit {code}"),
        }
    }
}

impl std::error::Error for CoordError {}

impl From<BoardError> for CoordError {
    fn from(e: BoardError) -> Self {
        CoordError::Refused(e.to_string())
    }
}

impl From<ClientError> for CoordError {
    fn from(e: ClientError) -> Self {
        CoordError::Refused(e.to_string())
    }
}

impl From<std::io::Error> for CoordError {
    fn from(e: std::io::Error) -> Self {
        CoordError::Refused(e.to_string())
    }
}

/// A verb's result: nothing to return, everything printed on the way.
pub type Outcome = Result<(), CoordError>;

/// Prints a refusal as `coord: <message>` on `stderr` and returns the exit code of an error.
pub fn report(err: &CoordError, stderr: &mut dyn Write) -> i32 {
    match err {
        CoordError::Refused(message) => {
            writeln!(stderr, "coord: {message}").ok();
            1
        }
        CoordError::Exit(code) => *code,
    }
}
