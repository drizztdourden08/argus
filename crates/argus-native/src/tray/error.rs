//! The error type of the tray.

use std::fmt;

/// Why the tray could not start or draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayError {
    /// Another tray already holds the icon for this dashboard home.
    AlreadyRunning,
    /// A file of the tray protocol could not be read or written.
    Io(String),
    /// Windows or the icon library refused.
    Platform(String),
}

impl fmt::Display for TrayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyRunning => f.write_str("a tray already runs for this dashboard home"),
            Self::Io(why) => write!(f, "a tray file could not be used: {why}"),
            Self::Platform(why) => write!(f, "the tray could not be shown: {why}"),
        }
    }
}

impl std::error::Error for TrayError {}

impl From<std::io::Error> for TrayError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
