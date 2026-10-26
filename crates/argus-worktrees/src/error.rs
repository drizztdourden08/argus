//! The refusals a removal can end in.

use crate::backup::Backup;
use std::fmt;

/// Why a request was refused. Each code has the HTTP status the server answers with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    NotFound,
    Forbidden,
    Conflict,
    InUse,
    BackupFailed,
    GitFailed,
    BadRequest,
}

impl ErrorCode {
    pub fn status(self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::Forbidden => 403,
            Self::Conflict => 409,
            Self::InUse => 423,
            Self::BackupFailed | Self::GitFailed => 500,
            Self::BadRequest => 400,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Forbidden => "forbidden",
            Self::Conflict => "conflict",
            Self::InUse => "in_use",
            Self::BackupFailed => "backup_failed",
            Self::GitFailed => "git_failed",
            Self::BadRequest => "bad_request",
        }
    }
}

/// A refusal: a code, a plain sentence and, when a backup was already written, where it is.
#[derive(Clone, Debug)]
pub struct WtError {
    pub code: ErrorCode,
    pub message: String,
    pub backup: Option<Box<Backup>>,
}

impl WtError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            backup: None,
        }
    }

    #[must_use]
    pub fn with_backup(mut self, backup: Option<Backup>) -> Self {
        self.backup = backup.map(Box::new);
        self
    }
}

impl fmt::Display for WtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WtError {}

pub type Result<T> = std::result::Result<T, WtError>;

/// A short code for an I/O error, the way a deletion failure is named on a row.
pub fn io_code(err: &std::io::Error) -> String {
    use std::io::ErrorKind;
    match err.kind() {
        ErrorKind::NotFound => "ENOENT".to_string(),
        ErrorKind::PermissionDenied => "EPERM".to_string(),
        ErrorKind::ResourceBusy => "EBUSY".to_string(),
        ErrorKind::DirectoryNotEmpty => "ENOTEMPTY".to_string(),
        _ => err.to_string().chars().take(80).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn codes_carry_their_status() {
        let pairs = [
            (ErrorCode::NotFound, 404),
            (ErrorCode::Forbidden, 403),
            (ErrorCode::Conflict, 409),
            (ErrorCode::InUse, 423),
            (ErrorCode::BackupFailed, 500),
            (ErrorCode::GitFailed, 500),
            (ErrorCode::BadRequest, 400),
        ];
        for (code, status) in pairs {
            assert_eq!(code.status(), status, "{}", code.as_str());
        }
    }

    #[test]
    fn io_errors_get_short_names() {
        assert_eq!(io_code(&Error::from(ErrorKind::ResourceBusy)), "EBUSY");
        assert_eq!(io_code(&Error::from(ErrorKind::NotFound)), "ENOENT");
        assert_eq!(io_code(&Error::other("something odd")), "something odd");
    }
}
