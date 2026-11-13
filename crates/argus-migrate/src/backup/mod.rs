//! Step 2: the backup, a sibling folder `<home>-backup-<stamp>` outside the home so no scan of
//! the home sees it: a copy of every file, the database through the SQLite backup API, and a
//! `manifest.json` the copy is checked against.

mod check;
mod copy;
mod db;
mod manifest;

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::{file_stamp, now_iso};

use crate::error::Result;
use crate::fault;
use crate::jsonio;
use crate::options::{Fault, Options};

pub use check::{against, backup as check_folder, read_manifest};
pub use copy::copy_tree;
pub use db::{REL as DB_REL, is_intact};
pub use manifest::{Manifest, NAME as MANIFEST};

/// A new sibling folder of the home: `<home>-<label>-<stamp>`.
pub fn sibling(home: &Path, label: &str) -> PathBuf {
    let base = home
        .file_name()
        .map_or_else(|| "coord".into(), std::ffi::OsStr::to_os_string);
    let stamp = file_stamp();
    let mut n = 1;
    loop {
        let mut name = base.clone();
        name.push(if n == 1 {
            format!("-{label}-{stamp}")
        } else {
            format!("-{label}-{stamp}-{n}")
        });
        let path = home.with_file_name(name);
        if !path.exists() {
            return path;
        }
        n += 1;
    }
}

fn damage(backup: &Path, manifest: &Manifest) -> Result<()> {
    if let Some(first) = manifest.files.first() {
        let file = backup.join(&first.path);
        let mut bytes = fs::read(&file)?;
        bytes.push(b'!');
        fs::write(&file, bytes)?;
    }
    Ok(())
}

/// Copies the home, writes the manifest, checks the copy; the backup folder.
pub fn make(opts: &Options) -> Result<PathBuf> {
    let home = &opts.home;
    let path = sibling(home, "backup");
    let mut files = copy_tree(home, &path, &copy::is_skipped)?;
    files.extend(db::copy(home, &path)?);
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let manifest = Manifest {
        created: now_iso(),
        source: home.display().to_string(),
        files,
    };
    jsonio::write(&path.join(MANIFEST), &serde_json::to_value(&manifest)?)?;
    if fault::armed(opts, Fault::CorruptBackup) {
        damage(&path, &manifest)?;
    }
    against(&path, &manifest)?;
    Ok(path)
}
