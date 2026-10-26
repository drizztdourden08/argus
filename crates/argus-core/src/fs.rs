//! The one way Argus writes a state file that must survive a crash: the bytes go to a temp file
//! in the same folder, the temp file is flushed to the disk, then it is renamed over the target.
//! This is `saveBoard` of the Node dashboard after `e9fc73f`: without the flush a power cut after
//! the rename leaves the new name on a file of the right size that holds only zero bytes.
//!
//! Where it differs from Node: the helper creates the folder, retries the rename for every file
//! (Node only for the board), and removes its temp file when the write, the flush or the rename
//! fails (Node leaves the temp file when the write itself fails). Like Node it does not flush the
//! folder entry, only the file.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

const RENAME_ATTEMPTS: u32 = 8;
const RENAME_PAUSE: Duration = Duration::from_millis(25);

/// The temp file of `file`: `<file>.<pid>.tmp` in the same folder, so the rename never crosses a
/// volume.
fn temp_of(file: &Path) -> PathBuf {
    let mut name = file.as_os_str().to_os_string();
    name.push(format!(".{}.tmp", std::process::id()));
    PathBuf::from(name)
}

fn retriable(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::PermissionDenied
        || (cfg!(windows) && matches!(e.raw_os_error(), Some(32 | 33)))
}

fn write_flushed(tmp: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut out = File::create(tmp)?;
    out.write_all(bytes)?;
    out.sync_all()
}

/// On Windows a reader holding the file open (the dashboard, an editor, a virus scan) makes the
/// rename fail for a moment: retry it.
fn rename_with_retry(
    tmp: &Path,
    file: &Path,
    rename: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let mut attempt = 1;
    loop {
        match rename(tmp, file) {
            Err(e) if retriable(&e) && attempt < RENAME_ATTEMPTS => {
                attempt += 1;
                std::thread::sleep(RENAME_PAUSE);
            }
            done => return done,
        }
    }
}

/// [`write_atomic`] with an injectable rename (tests).
///
/// # Errors
/// The folder or the temp file cannot be made or flushed, or the rename keeps failing. The temp
/// file is removed on every error.
pub fn write_atomic_with(
    file: &Path,
    bytes: &[u8],
    rename: &dyn Fn(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    let tmp = temp_of(file);
    write_flushed(&tmp, bytes)
        .and_then(|()| rename_with_retry(&tmp, file, rename))
        .inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
}

/// Writes `bytes` to `file` so a crash leaves the old file or the whole new one, never a hollow
/// one: temp file beside it, `sync_all`, rename over the target (which replaces it on Windows).
///
/// # Errors
/// The folder or the temp file cannot be made or flushed, or the rename keeps failing.
pub fn write_atomic(file: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    write_atomic_with(file, bytes.as_ref(), &|from, to| fs::rename(from, to))
}

/// Flushes a file somebody else wrote (a database copy made by SQLite) to the disk, so a rename
/// of it that follows is safe.
///
/// # Errors
/// The file cannot be opened for writing or flushed.
pub fn flush_file(file: &Path) -> io::Result<()> {
    fs::OpenOptions::new().write(true).open(file)?.sync_all()
}

/// Copies `from` to `to` the way [`write_atomic`] writes: `to` is whole or absent, and flushed.
///
/// # Errors
/// `from` cannot be read or `to` cannot be written.
pub fn copy_atomic(from: &Path, to: &Path) -> io::Result<()> {
    write_atomic(to, fs::read(from)?)
}

/// Writes a file that must not exist yet (a backup with a unique name) and flushes it to the
/// disk before returning, so the caller may go on to delete what the backup holds.
///
/// # Errors
/// The file exists, or cannot be written or flushed.
pub fn write_new(file: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let mut out = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)?;
    out.write_all(bytes.as_ref())?;
    out.sync_all()
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn a_copy_is_whole_and_leaves_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let (from, to) = (
            dir.path().join("a.png"),
            dir.path().join("kept").join("a.png"),
        );
        fs::write(&from, [1, 2, 3]).unwrap();
        copy_atomic(&from, &to).unwrap();
        assert_eq!(fs::read(&to).unwrap(), [1, 2, 3]);
        assert_eq!(names(&dir.path().join("kept")), ["a.png"]);
        assert!(copy_atomic(&dir.path().join("missing"), &to).is_err());
    }

    #[test]
    fn a_new_file_is_written_once_and_never_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("board.v1.json");
        write_new(&file, "first").unwrap();
        let err = write_new(&file, "second").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(&file).unwrap(), "first");
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut all: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        all.sort();
        all
    }

    #[test]
    fn the_bytes_round_trip_and_no_temp_file_is_left() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("board.json");
        write_atomic(&file, b"{\"a\":1}\n").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"{\"a\":1}\n");
        assert_eq!(names(dir.path()), ["board.json"]);
    }

    #[test]
    fn a_second_write_replaces_the_first_and_text_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.json");
        write_atomic(&file, "first, longer text").unwrap();
        write_atomic(&file, String::from("second")).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "second");
        assert_eq!(names(dir.path()), ["config.json"]);
    }

    #[test]
    fn the_folder_is_created() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a").join("b").join("x.json");
        write_atomic(&file, b"x").unwrap();
        assert_eq!(fs::read(&file).unwrap(), b"x");
    }

    #[test]
    fn the_temp_file_sits_beside_the_target_and_names_the_pid() {
        let temp = temp_of(Path::new("/h/board.json"));
        assert_eq!(
            temp,
            PathBuf::from(format!("/h/board.json.{}.tmp", std::process::id()))
        );
    }

    #[test]
    fn a_failed_rename_removes_the_temp_file_and_keeps_the_old_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("board.json");
        fs::write(&file, "old").unwrap();
        let calls = Cell::new(0);
        let denied = |_: &Path, _: &Path| {
            calls.set(calls.get() + 1);
            Err(io::Error::from(io::ErrorKind::PermissionDenied))
        };
        let err = write_atomic_with(&file, b"new", &denied).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(calls.get(), RENAME_ATTEMPTS);
        assert_eq!(names(dir.path()), ["board.json"]);
        assert_eq!(fs::read_to_string(&file).unwrap(), "old");
    }

    #[test]
    fn a_rename_that_is_not_a_sharing_error_is_not_retried() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.json");
        let calls = Cell::new(0);
        let broken = |_: &Path, _: &Path| {
            calls.set(calls.get() + 1);
            Err(io::Error::from(io::ErrorKind::NotFound))
        };
        assert!(write_atomic_with(&file, b"x", &broken).is_err());
        assert_eq!(calls.get(), 1);
        assert_eq!(names(dir.path()), Vec::<String>::new());
    }

    #[test]
    fn a_busy_file_is_retried_until_the_rename_goes_through() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("x.json");
        let calls = Cell::new(0);
        let flaky = |from: &Path, to: &Path| {
            calls.set(calls.get() + 1);
            if calls.get() < 3 {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            } else {
                fs::rename(from, to)
            }
        };
        write_atomic_with(&file, b"ok", &flaky).unwrap();
        assert_eq!(calls.get(), 3);
        assert_eq!(fs::read(&file).unwrap(), b"ok");
        assert_eq!(names(dir.path()), ["x.json"]);
    }

    #[test]
    fn a_folder_that_cannot_be_made_leaves_nothing_and_reports_it() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("blocker");
        fs::write(&blocker, "a file, not a folder").unwrap();
        assert!(write_atomic(&blocker.join("x.json"), b"x").is_err());
        assert_eq!(names(dir.path()), ["blocker"]);
    }
}
