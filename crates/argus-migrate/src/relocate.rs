//! Step 4: one board folder moves from a name to another: the `board.json` is rewritten (temp
//! file, then rename), then the folder is renamed, retrying while a reader holds it. The rerun
//! rule: only the new name = done, only the old name = redo, both = stop and report.

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

/// Moves the board folder `from` to `to`, rewriting its `board.json` with `map` first.
pub fn move_one(home: &Path, from: &str, to: &str, map: &IdMap) -> Result<()> {
    match (home.join(from).is_dir(), home.join(to).is_dir()) {
        (false, true) => Ok(()),
        (true, false) => {
            rewrite_file(&board_file(home, from), map)?;
            rename_retry(&home.join(from), &home.join(to))
        }
        (true, true) => stop(format!(
            "both {from} and {to} exist under the home: stop, nothing was merged; report it"
        )),
        (false, false) => stop(format!("the board folder {from} is gone")),
    }
}
