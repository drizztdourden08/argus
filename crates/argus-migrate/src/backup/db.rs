//! The database goes into the backup through the SQLite backup API, so a copy made while a
//! writer waits is still whole.

use std::fs;
use std::path::Path;
use std::time::Duration;

use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};

use super::manifest::{Entry, hash_file};
use crate::error::Result;

/// The database of the dashboard home, relative to the coord home.
pub const REL: &str = "dashboard/dashboard.db";

/// Copies `<home>/dashboard/dashboard.db` into the backup; its manifest entry, if it exists.
pub fn copy(home: &Path, backup: &Path) -> Result<Option<Entry>> {
    let from = home.join(REL);
    if !from.is_file() {
        return Ok(None);
    }
    let to = backup.join(REL);
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    let source = Connection::open_with_flags(&from, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut target = Connection::open(&to)?;
    {
        let run = Backup::new(&source, &mut target)?;
        run.run_to_completion(256, Duration::from_millis(5), None)?;
    }
    drop(target);
    let (size, sha256) = hash_file(&to)?;
    Ok(Some(Entry {
        path: REL.to_owned(),
        size,
        sha256,
    }))
}

/// `PRAGMA integrity_check` of a database file: whether it says `ok`.
pub fn is_intact(file: &Path) -> Result<bool> {
    let conn = Connection::open_with_flags(file, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let answer: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    Ok(answer == "ok")
}
