//! Copying the home into the backup: every file, hashed while it is copied. Links and junctions
//! are not followed (a worktree junction would copy a whole checkout), locks and the migration's
//! own files are left out, and the database goes through the SQLite backup API instead.

use std::fs::{self, Metadata};
use std::io;
use std::path::{Path, PathBuf};

use super::manifest::{Entry, Link, copy_hashed};

/// A lock file, whatever the case of its extension.
fn is_lock(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("lock"))
}

/// What a backup of the home never copies, by path under the home.
pub fn is_skipped(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    is_lock(name)
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

/// What a copy made and what it left out.
#[derive(Default)]
pub struct Copied {
    /// Every file copied.
    pub files: Vec<Entry>,
    /// Every link or junction not followed.
    pub links: Vec<Link>,
    /// How many lock files were skipped.
    pub locks: usize,
}

fn link_of(path: &str, item: &Path) -> Link {
    let target = fs::read_link(item).map(|t| t.display().to_string());
    Link {
        path: path.to_owned(),
        target: target.unwrap_or_default(),
    }
}

/// Copies the tree of `from` into `to`, skipping what `skip` names; lists files, links and locks.
pub fn copy_tree(from: &Path, to: &Path, skip: &dyn Fn(&str) -> bool) -> io::Result<Copied> {
    let mut done = Copied::default();
    let mut stack: Vec<PathBuf> = vec![PathBuf::new()];
    fs::create_dir_all(to)?;
    while let Some(rel) = stack.pop() {
        for item in fs::read_dir(from.join(&rel))? {
            let item = item?;
            let name = item.file_name().to_string_lossy().into_owned();
            let path = rel_of(&rel, &name);
            let meta = fs::symlink_metadata(item.path())?;
            if is_link(&meta) {
                done.links.push(link_of(&path, &item.path()));
                continue;
            }
            if skip(&path) {
                done.locks += usize::from(is_lock(&name));
                continue;
            }
            if meta.is_dir() {
                fs::create_dir_all(to.join(&path))?;
                stack.push(PathBuf::from(&path));
            } else {
                let (size, sha256) = copy_hashed(&item.path(), &to.join(&path))?;
                done.files.push(Entry { path, size, sha256 });
            }
        }
    }
    done.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(done)
}
