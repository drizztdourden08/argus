//! Reading a board file and saving it atomically: the temp file and rename of `saveBoard`, the
//! rename retry for Windows sharing errors, and the backup of a board that still needed
//! converting.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::Duration;

use argus_core::clock::file_stamp;
use serde_json::Value;

use super::normalize::{normalize, stale_schema};
use super::{board_dir, board_path};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, section, truthy};

const RENAME_ATTEMPTS: u32 = 8;
const RENAME_PAUSE: Duration = Duration::from_millis(25);

type Unmigrated = HashMap<PathBuf, (String, &'static str)>;

static UNMIGRATED: LazyLock<Mutex<Unmigrated>> = LazyLock::new(Mutex::default);

fn unmigrated() -> std::sync::MutexGuard<'static, Unmigrated> {
    UNMIGRATED.lock().unwrap_or_else(PoisonError::into_inner)
}

fn has_embedded_agents(board: &Board) -> bool {
    section(board, "threads")
        .values()
        .any(|t| t["agents"].as_array().is_some_and(|a| !a.is_empty()))
}

fn note_unmigrated(file: &Path, raw: String, board: &Board) {
    let mut map = unmigrated();
    if stale_schema(board) {
        map.insert(file.to_path_buf(), (raw, "board.v1"));
    } else if has_embedded_agents(board) && !truthy(board, "agentsMigrated") {
        map.insert(file.to_path_buf(), (raw, "board.pre-agents"));
    } else {
        map.remove(file);
    }
}

/// Read and normalize the board of a project. `None` when it has no board file. A board that
/// still needs converting is remembered, and its first save writes the old text beside it as
/// `board.v1.<timestamp>.json` or `board.pre-agents.<timestamp>.json`.
///
/// # Errors
/// The file cannot be read or is not a JSON object.
pub fn read_board(home: &Path, key: &str) -> Result<Option<Board>, BoardError> {
    let file = board_path(home, key);
    let raw = match fs::read_to_string(&file) {
        Ok(raw) => raw,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let mut board: Value = serde_json::from_str(&raw)?;
    if !board.is_object() {
        return Err(refuse(format!(
            "the board of \"{key}\" is not a JSON object."
        )));
    }
    note_unmigrated(&file, raw, &board);
    normalize(&mut board);
    Ok(Some(board))
}

fn write_backup(dir: &Path, file: &Path) -> Result<(), BoardError> {
    let Some((raw, name)) = unmigrated().get(file).cloned() else {
        return Ok(());
    };
    let backup = dir.join(format!("{name}.{}.json", file_stamp()));
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(backup)?;
    out.write_all(raw.as_bytes())?;
    unmigrated().remove(file);
    Ok(())
}

fn retriable(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::PermissionDenied
        || (cfg!(windows) && matches!(e.raw_os_error(), Some(32 | 33)))
}

/// On Windows a reader holding the board open makes the rename fail for a moment: retry, and
/// never leave the temp file behind.
fn rename_with_retry(
    tmp: &Path,
    file: &Path,
    rename: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> Result<(), BoardError> {
    for attempt in 1..=RENAME_ATTEMPTS {
        match rename(tmp, file) {
            Ok(()) => return Ok(()),
            Err(e) if !retriable(&e) || attempt >= RENAME_ATTEMPTS => {
                fs::remove_file(tmp).ok();
                return Err(e.into());
            }
            Err(_) => std::thread::sleep(RENAME_PAUSE),
        }
    }
    Ok(())
}

/// Save a board with an injectable rename (tests): the unmigrated backup first, then two-space
/// JSON and a newline into `board.json.<pid>.tmp`, then the rename, retried on a sharing error.
///
/// # Errors
/// A file cannot be written or the rename keeps failing.
pub fn save_board_with(
    home: &Path,
    board: &Board,
    rename: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> Result<(), BoardError> {
    let project = display(&board["project"]);
    let dir = board_dir(home, &project);
    fs::create_dir_all(&dir)?;
    let file = board_path(home, &project);
    write_backup(&dir, &file)?;
    let tmp = dir.join(format!("board.json.{}.tmp", std::process::id()));
    fs::write(&tmp, format!("{}\n", serde_json::to_string_pretty(board)?))?;
    rename_with_retry(&tmp, &file, rename)
}

/// Save a board atomically.
///
/// # Errors
/// A file cannot be written or the rename keeps failing.
pub fn save_board(home: &Path, board: &Board) -> Result<(), BoardError> {
    save_board_with(home, board, &|from, to| fs::rename(from, to))
}
