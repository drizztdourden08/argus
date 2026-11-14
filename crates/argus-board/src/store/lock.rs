//! The board lock file, `<project>/board.lock`, and the two ways to wait for it: blocking (the
//! CLI) and polling with timers (the server). A lock older than 15 s belongs to a crashed call.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use argus_core::clock::now_iso;
use argus_core::lockfile;

use super::io::{read_board, save_board};
use super::{board_dir, empty_board};
use crate::Board;
use crate::error::BoardError;
use crate::json::set;

const LOCK_WAIT: Duration = Duration::from_secs(5);
const LOCK_STALE: Duration = Duration::from_secs(15);
const BLOCKING_PAUSE: Duration = Duration::from_millis(50);
const FIRST_PAUSE: Duration = Duration::from_millis(25);
const MAX_PAUSE: Duration = Duration::from_millis(200);

/// A held board lock; dropping it releases the lock.
#[derive(Debug)]
pub struct BoardLock {
    path: PathBuf,
}

impl Drop for BoardLock {
    fn drop(&mut self) {
        fs::remove_file(&self.path).ok();
    }
}

fn take(path: &Path) -> Result<Option<BoardLock>, BoardError> {
    Ok(lockfile::create_new(path)?.then(|| BoardLock {
        path: path.to_path_buf(),
    }))
}

fn age_of(path: &Path) -> io::Result<Duration> {
    let modified = fs::metadata(path)?.modified()?;
    Ok(SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default())
}

/// One attempt at the lock of a project folder: the lock, or `None` while someone else holds it.
/// A lock older than 15 s belongs to a crashed call and is cleared first.
///
/// # Errors
/// The folder or the lock file cannot be created.
pub fn try_lock(dir: &Path) -> Result<Option<BoardLock>, BoardError> {
    fs::create_dir_all(dir)?;
    let path = dir.join("board.lock");
    if let Some(lock) = take(&path)? {
        return Ok(Some(lock));
    }
    match age_of(&path) {
        Err(_) => take(&path),
        Ok(age) if age <= LOCK_STALE => Ok(None),
        Ok(_) => {
            fs::remove_file(&path).ok();
            take(&path)
        }
    }
}

fn mutate<T>(
    home: &Path,
    key: &str,
    create: bool,
    f: impl FnOnce(&mut Board) -> Result<T, BoardError>,
) -> Result<T, BoardError> {
    let mut board = match read_board(home, key)? {
        Some(board) => board,
        None if create => empty_board(key),
        None => return Err(BoardError::Missing(key.to_owned())),
    };
    let result = f(&mut board)?;
    set(&mut board, "updatedAt", now_iso());
    save_board(home, &board)?;
    Ok(result)
}

/// Run a mutation under the lock, waiting for it by blocking (up to 5 s, 50 ms apart): the CLI
/// flavour. `create` allows the first write (board init).
///
/// # Errors
/// [`BoardError::Locked`] when the lock stays taken, [`BoardError::Missing`] without a board
/// and without `create`, or whatever `f` or the save fails with.
pub fn with_board<T>(
    home: &Path,
    key: &str,
    create: bool,
    f: impl FnOnce(&mut Board) -> Result<T, BoardError>,
) -> Result<T, BoardError> {
    let deadline = Instant::now() + LOCK_WAIT;
    let lock = loop {
        if let Some(lock) = try_lock(&board_dir(home, key))? {
            break lock;
        }
        if Instant::now() > deadline {
            return Err(BoardError::Locked(key.to_owned()));
        }
        std::thread::sleep(BLOCKING_PAUSE);
    };
    let result = mutate(home, key, create, f);
    drop(lock);
    result
}

/// How long [`with_board_async`] waits for the lock, and whether it may create the board.
#[derive(Debug, Clone, Copy)]
pub struct LockWait {
    /// The longest wait for the lock.
    pub wait: Duration,
    /// Allow the first write of a board.
    pub create: bool,
}

impl Default for LockWait {
    fn default() -> Self {
        LockWait {
            wait: Duration::from_secs(10),
            create: false,
        }
    }
}

/// The server flavour: wait for the lock with timers (25 ms doubling to 200 ms) so the runtime
/// keeps serving, then read, mutate and save synchronously so nothing interleaves.
///
/// # Errors
/// [`BoardError::Locked`] after the wait, [`BoardError::Missing`] without a board and without
/// `create`, or whatever `f` or the save fails with.
pub async fn with_board_async<T>(
    home: &Path,
    key: &str,
    opts: LockWait,
    f: impl FnOnce(&mut Board) -> Result<T, BoardError>,
) -> Result<T, BoardError> {
    let deadline = Instant::now() + opts.wait;
    let mut pause = FIRST_PAUSE;
    loop {
        if let Some(lock) = try_lock(&board_dir(home, key))? {
            let result = mutate(home, key, opts.create, f);
            drop(lock);
            return result;
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(BoardError::Locked(key.to_owned()));
        }
        tokio::time::sleep(pause.min(left.max(Duration::from_millis(1)))).await;
        pause = (pause * 2).min(MAX_PAUSE);
    }
}
