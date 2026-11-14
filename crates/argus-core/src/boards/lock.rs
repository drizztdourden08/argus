//! `boards.lock`: one writer of the table at a time. A lock older than 15 s belongs to a crash.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::error::BoardsError;
use crate::lockfile;

const WAIT: Duration = Duration::from_secs(5);
const STALE: Duration = Duration::from_secs(15);
const PAUSE: Duration = Duration::from_millis(25);

/// A held lock; dropping it releases it.
#[derive(Debug)]
pub struct TableLock(PathBuf);

impl Drop for TableLock {
    fn drop(&mut self) {
        fs::remove_file(&self.0).ok();
    }
}

fn take(path: &Path) -> Result<Option<TableLock>, BoardsError> {
    Ok(lockfile::create_new(path)?.then(|| TableLock(path.to_path_buf())))
}

/// Waits up to 5 s for the lock of `home`.
///
/// # Errors
/// The lock stays held, or its file cannot be created.
pub fn acquire(home: &Path) -> Result<TableLock, BoardsError> {
    fs::create_dir_all(home)?;
    let path = home.join("boards.lock");
    let start = Instant::now();
    loop {
        if let Some(lock) = take(&path)? {
            return Ok(lock);
        }
        if lockfile::is_stale(&path, STALE) {
            fs::remove_file(&path).ok();
        } else if start.elapsed() > WAIT {
            return Err(BoardsError::Locked);
        } else {
            sleep(PAUSE);
        }
    }
}
