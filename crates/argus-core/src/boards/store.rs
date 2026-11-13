//! Reading and writing `<home>/boards.json`.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::error::BoardsError;
use super::lock::acquire;
use super::model::Boards;
use crate::fs::write_atomic;

fn file(home: &Path) -> PathBuf {
    home.join("boards.json")
}

/// Reads the table; a home without one (not migrated) gives an empty table.
///
/// # Errors
/// The file exists but cannot be read or parsed.
pub fn load(home: &Path) -> Result<Boards, BoardsError> {
    match fs::read_to_string(file(home)) {
        Ok(text) => Ok(serde_json::from_str(&text)?),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Boards::default()),
        Err(e) => Err(e.into()),
    }
}

fn write(home: &Path, boards: &Boards) -> Result<(), BoardsError> {
    let text = serde_json::to_string_pretty(boards)?;
    write_atomic(&file(home), text + "\n")?;
    Ok(())
}

/// Writes the table atomically under `boards.lock`.
///
/// # Errors
/// The lock stays held or the file cannot be written.
pub fn save(home: &Path, boards: &Boards) -> Result<(), BoardsError> {
    let _lock = acquire(home)?;
    write(home, boards)
}

/// Loads, changes and writes the table under one hold of the lock.
///
/// # Errors
/// What `f`, the lock, the read or the write refuse.
pub fn update<T>(
    home: &Path,
    f: impl FnOnce(&mut Boards) -> Result<T, BoardsError>,
) -> Result<T, BoardsError> {
    let _lock = acquire(home)?;
    let mut boards = load(home)?;
    let out = f(&mut boards)?;
    write(home, &boards)?;
    Ok(out)
}
