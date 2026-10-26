//! A consistent copy of a live database: the shadow server starts from the Node dashboard's one.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags};

use crate::StoreError;

fn partial_of(to: &Path) -> PathBuf {
    let mut name = to.as_os_str().to_owned();
    name.push(".copying");
    PathBuf::from(name)
}

fn vacuum_into(from: &Path, partial: &Path) -> Result<(), StoreError> {
    let conn = Connection::open_with_flags(from, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute("VACUUM INTO ?1", [partial.to_string_lossy().as_ref()])?;
    Ok(())
}

/// Copies the database `from` to `to` with `VACUUM INTO`, opening `from` read-only (it is never
/// written). The copy is made beside `to` and renamed, so `to` is whole or absent.
///
/// # Errors
/// A missing or unreadable source, a refused `VACUUM`, or a file that cannot be made.
pub fn snapshot(from: &Path, to: &Path) -> Result<(), StoreError> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    let partial = partial_of(to);
    let _ = fs::remove_file(&partial);
    let made = vacuum_into(from, &partial).and_then(|()| Ok(fs::rename(&partial, to)?));
    if made.is_err() {
        let _ = fs::remove_file(&partial);
    }
    made
}
