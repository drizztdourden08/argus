//! Copying the home into the backup: every file, hashed while it is copied. Links and junctions
//! are not followed (a worktree junction would copy a whole checkout), locks and the migration's
//! own files are left out, and the database goes through the SQLite backup API instead.

use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use super::manifest::{Entry, copy_hashed};

/// What a backup of the home never copies, by path under the home.
pub fn is_skipped(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.ends_with(".lock")
        || matches!(
            rel,
            "MIGRATING"
                | "board-ids.plan.json"
                | "board-ids.journal.json"
                | "dashboard/dashboard.db"
                | "dashboard/dashboard.db-wal"
                | "dashboard/dashboard.db-shm"
        )
}

#[cfg(windows)]
fn is_link(meta: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_type().is_symlink() || meta.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_link(meta: &Metadata) -> bool {
    meta.file_type().is_symlink()
}

fn rel_of(rel: &Path, name: &str) -> String {
    if rel.as_os_str().is_empty() {
        name.to_owned()
    } else {
        format!("{}/{name}", rel.to_string_lossy().replace('\\', "/"))
    }
}

/// Copies the tree of `from` into `to`, skipping what `skip` names, and lists the files.
pub fn copy_tree(from: &Path, to: &Path, skip: &dyn Fn(&str) -> bool) -> io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut stack: Vec<PathBuf> = vec![PathBuf::new()];
    fs::create_dir_all(to)?;
    while let Some(rel) = stack.pop() {
        for item in fs::read_dir(from.join(&rel))? {
            let item = item?;
            let name = item.file_name().to_string_lossy().into_owned();
            let path = rel_of(&rel, &name);
            let meta = fs::symlink_metadata(item.path())?;
            if is_link(&meta) || skip(&path) {
                continue;
            }
            if meta.is_dir() {
                fs::create_dir_all(to.join(&path))?;
                stack.push(PathBuf::from(&path));
            } else {
                let (size, sha256) = copy_hashed(&item.path(), &to.join(&path))?;
                entries.push(Entry { path, size, sha256 });
            }
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}
