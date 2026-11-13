//! Checking a backup (or a restored home) against its manifest, and the database with
//! `PRAGMA integrity_check`. Any mismatch stops the run before the first change.

use std::fs;
use std::path::Path;

use super::db;
use super::manifest::{Manifest, NAME, hash_file};
use crate::error::{Result, stop};

/// Reads the manifest of a backup.
pub fn read_manifest(backup: &Path) -> Result<Manifest> {
    let text = fs::read_to_string(backup.join(NAME))?;
    Ok(serde_json::from_str(&text)?)
}

fn mismatch(root: &Path, path: &str, size: u64, sha256: &str) -> bool {
    match hash_file(&root.join(path)) {
        Ok((got_size, got_hash)) => got_size != size || got_hash != sha256,
        Err(_) => true,
    }
}

/// Every file of `manifest` under `root` has its size and hash, and the database is intact.
pub fn against(root: &Path, manifest: &Manifest) -> Result<()> {
    let bad: Vec<&str> = manifest
        .files
        .iter()
        .filter(|f| mismatch(root, &f.path, f.size, &f.sha256))
        .map(|f| f.path.as_str())
        .take(5)
        .collect();
    if !bad.is_empty() {
        return stop(format!(
            "the backup does not match its manifest ({}): nothing was changed",
            bad.join(", ")
        ));
    }
    let file = root.join(db::REL);
    if file.is_file() && !db::is_intact(&file)? {
        return stop("the database of the backup fails its integrity check: nothing was changed");
    }
    Ok(())
}

/// Checks a backup folder against its own manifest.
pub fn backup(path: &Path) -> Result<()> {
    against(path, &read_manifest(path)?)
}
