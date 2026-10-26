//! The one error type of the native helper.

use std::fmt;

/// Why a toast could not be built or shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotifyError {
    /// The title and the text are both empty once cleaned.
    Empty,
    /// The click address is not a plain `http` or `https` address.
    InvalidUrl(String),
    /// The replace tag is empty or longer than Windows allows.
    InvalidTag(String),
    /// The platform refused or could not show the toast.
    Platform(String),
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("a toast needs a title or a text"),
            Self::InvalidUrl(why) => write!(f, "the toast address is not usable: {why}"),
            Self::InvalidTag(why) => write!(f, "the toast tag is not usable: {why}"),
            Self::Platform(why) => write!(f, "the toast could not be shown: {why}"),
        }
    }
}

impl std::error::Error for NotifyError {}
