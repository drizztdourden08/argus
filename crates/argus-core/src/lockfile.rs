//! The lock-file primitives shared by every writer of a `.lock` file: create it exclusively, and
//! tell whether a lock is old enough to belong to a crash.

use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Creates the lock file. True when this call made it, false while someone else holds it.
///
/// # Errors
/// The file cannot be created for a reason other than being held.
pub fn create_new(path: &Path) -> io::Result<bool> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(_) => Ok(true),
        Err(e)
            if matches!(
                e.kind(),
                io::ErrorKind::AlreadyExists | io::ErrorKind::PermissionDenied
            ) =>
        {
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

/// True when the lock file is older than `limit` or its age cannot be read.
#[must_use]
pub fn is_stale(path: &Path, limit: Duration) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map_or(true, |t| {
            SystemTime::now().duration_since(t).unwrap_or_default() > limit
        })
}
