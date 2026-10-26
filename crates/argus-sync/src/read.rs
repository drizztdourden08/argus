//! Reading a board file for an export: no lock (saves are renames); a busy file or a half
//! written one is retried a few times.

use std::io::ErrorKind;
use std::path::Path;
use std::time::Duration;

use argus_board::store::{board_path, normalize};
use argus_board::{Board, BoardError};

const ATTEMPTS: u32 = 4;
const PAUSE: Duration = Duration::from_millis(40);

fn read_once(home: &Path, key: &str) -> Result<Option<Board>, BoardError> {
    match std::fs::read_to_string(board_path(home, key)) {
        Ok(text) => {
            let mut board: Board = serde_json::from_str(&text)?;
            normalize(&mut board);
            Ok(Some(board))
        }
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn retryable(e: &BoardError) -> bool {
    match e {
        BoardError::Io(io) => {
            matches!(
                io.kind(),
                ErrorKind::PermissionDenied | ErrorKind::ResourceBusy
            ) || io.raw_os_error() == Some(32)
        }
        BoardError::Json(_) => true,
        _ => false,
    }
}

/// The board parsed and normalized; `None` when there is none.
///
/// # Errors
/// The file stayed unreadable or invalid after the retries.
pub async fn read_board_file(home: &Path, key: &str) -> Result<Option<Board>, BoardError> {
    let mut attempt = 1;
    loop {
        match read_once(home, key) {
            Err(e) if retryable(&e) && attempt < ATTEMPTS => {
                attempt += 1;
                tokio::time::sleep(PAUSE).await;
            }
            other => return other,
        }
    }
}
