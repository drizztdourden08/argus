//! Why the key store could not do what was asked. The messages never carry the key.

use std::fmt;

/// A failure of the key store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretError {
    /// The key is empty once trimmed.
    Empty,
    /// The folder or the file could not be read or written.
    Io(String),
    /// The platform could not protect or unprotect the value, or has no such facility.
    Platform(String),
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("the key is empty"),
            Self::Io(why) => write!(f, "the key file could not be used: {why}"),
            Self::Platform(why) => write!(f, "the key could not be protected: {why}"),
        }
    }
}

impl std::error::Error for SecretError {}
