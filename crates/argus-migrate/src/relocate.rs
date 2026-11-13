//! Step 4: one board folder moves from a name to another: the `board.json` is rewritten (temp
//! file, then rename), then the folder is renamed, retrying while a reader holds it. The rerun
//! rule: only the new name = done (its `board.json` is still rewritten, it is idempotent, in case
//! the rewrite ran and the rename failed), only the old name = redo, both = stop and report.
//! An old writer waiting on `board.lock` may recreate the old folder with that one file in it:
//! such a folder is removed, anything more stops the run.

use std::fs;
use std::io;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crate::error::{Result, stop};
use crate::map::IdMap;
use crate::plan::board_file;
use crate::rewrite::board::rewrite_file;

const ATTEMPTS: u32 = 40;
const PAUSE: Duration = Duration::from_millis(250);
const LOCK: &str = "board.lock";

fn is_sharing(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::PermissionDenied || matches!(e.raw_os_error(), Some(5 | 32 | 33))
}

/// Renames a folder, retrying on a sharing error.
pub fn rename_retry(from: &Path, to: &Path) -> Result<()> {
    let mut attempt = 1;
    loop {
        match fs::rename(from, to) {
            Err(e) if is_sharing(&e) && attempt < ATTEMPTS => {
                attempt += 1;
                sleep(PAUSE);
            }
            done => return Ok(done?),
        }
    }
}

/// Whether the folder exists with no `board.json` and nothing but a `board.lock` (or nothing).
pub fn is_stray(home: &Path, name: &str) -> bool {
    let dir = home.join(name);
    if dir.join("board.json").exists() {
        return false;
    }
    fs::read_dir(&dir)
        .is_ok_and(|items| items.filter_map(io::Result::ok).all(|i| i.file_name() == LOCK))
}

fn drop_stray(home: &Path, name: &str) -> Result<()> {
    if !is_stray(home, name) {
        return Ok(());
    }
    let dir = home.join(name);
    match fs::remove_file(dir.join(LOCK)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e.into()),
        _ => {}
    }
    Ok(fs::remove_dir(&dir)?)
}

fn after_move(home: &Path, from: &str) -> Result<()> {
    drop_stray(home, from)?;
    if home.join(from).exists() {
        return stop(format!(
            "{from} was recreated beside the moved board and holds more than a lock: stop, report it"
        ));
    }
    Ok(())
}

/// Moves the board folder `from` to `to`, rewriting its `board.json` with `map` first.
pub fn move_one(home: &Path, from: &str, to: &str, map: &IdMap) -> Result<()> {
    drop_stray(home, from)?;
    drop_stray(home, to)?;
    match (home.join(from).is_dir(), home.join(to).is_dir()) {
        (false, true) => rewrite_file(&board_file(home, to), map),
        (true, false) => {
            rewrite_file(&board_file(home, from), map)?;
            rename_retry(&home.join(from), &home.join(to))?;
            after_move(home, from)
        }
        (true, true) => stop(format!(
            "both {from} and {to} exist under the home: stop, nothing was merged; report it"
        )),
        (false, false) => stop(format!("the board folder {from} is gone")),
    }
}
