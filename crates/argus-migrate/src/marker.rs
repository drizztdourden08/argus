//! The `MIGRATING` marker: while it exists the coord CLI refuses board writes and the server does
//! not start. It holds the pid and start time of the run that wrote it.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::now_iso;
use argus_core::fs::write_atomic;
use serde_json::{Value, json};

use crate::error::{Error, Result, stop};
use crate::options::Options;

fn path(home: &Path) -> PathBuf {
    home.join("MIGRATING")
}

fn owner(home: &Path) -> Option<u32> {
    let text = fs::read_to_string(path(home)).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    u32::try_from(value["pid"].as_u64()?).ok()
}

/// Whether a marker is there.
pub fn exists(home: &Path) -> bool {
    path(home).is_file()
}

/// Writes the marker into `dir` (the home, or a copy about to become it).
pub fn put(dir: &Path) -> Result<()> {
    let marker = json!({ "pid": std::process::id(), "start": now_iso() });
    write_atomic(&path(dir), format!("{marker}\n"))?;
    Ok(())
}

/// Ends a run that refused before it changed anything. The marker goes unless `keep` (the home is
/// half migrated, or a marker was already there): then it stays and the message says so.
pub fn release(home: &Path, keep: bool, e: Error) -> Error {
    if !keep {
        clear(home);
        return e;
    }
    Error::Stop(format!(
        "{e}\nthe MIGRATING marker stays: the home is mid-migration, so writers stay refused until a rerun finishes"
    ))
}

/// Writes the marker; refuses while another live process holds one.
pub fn take(opts: &Options) -> Result<()> {
    let other = owner(&opts.home).filter(|p| *p != std::process::id() && (opts.pid_alive)(*p));
    if let Some(pid) = other {
        return stop(format!("another migration is running (pid {pid})"));
    }
    put(&opts.home)
}

/// Removes the marker.
pub fn clear(home: &Path) {
    fs::remove_file(path(home)).ok();
}
