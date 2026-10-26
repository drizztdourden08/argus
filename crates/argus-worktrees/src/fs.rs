//! The file system as the crate sees it. A link (symlink or junction) is always a leaf: never followed.

use crate::clock::system_ms;
use crate::paths::join_path;
use std::fs::{self, Metadata};
use std::io;
use std::thread;
use std::time::Duration;

/// What `lstat` says: the entry itself, never what a link points to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stat {
    pub is_link: bool,
    pub is_dir: bool,
    pub mtime_ms: Option<i64>,
    pub size: u64,
}

/// One entry of a folder, from its own type (a link is not a folder).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub is_link: bool,
}

/// The file system. Tests wrap it to fail on cue or to be a virtual tree.
pub trait Fs: Send + Sync {
    /// Does the path exist (a link is followed)?
    fn exists(&self, path: &str) -> bool;
    /// Is the path a folder (a link is followed)?
    fn is_dir(&self, path: &str) -> bool;
    /// The names of the sub-folders (a link to a folder counts).
    fn dirs(&self, path: &str) -> io::Result<Vec<String>>;
    /// The modification time of a path (a link is followed), in milliseconds.
    fn mtime_ms(&self, path: &str) -> Option<i64>;
    fn lstat(&self, path: &str) -> io::Result<Stat>;
    fn read_dir(&self, path: &str) -> io::Result<Vec<Entry>>;
    /// The text of a small file (a git bookkeeping file).
    fn read_text(&self, path: &str) -> io::Result<String> {
        fs::read_to_string(path)
    }
    fn rename(&self, from: &str, to: &str) -> io::Result<()>;
    /// Remove a file, or a link as a link.
    fn remove_file(&self, path: &str) -> io::Result<()>;
    /// Remove an empty folder, or a junction as a link.
    fn remove_dir(&self, path: &str) -> io::Result<()>;
    /// Delete a folder and everything in it. A link inside is removed as a link, never followed.
    fn remove_tree(&self, path: &str) -> io::Result<()> {
        crate::tree::walk_remove(self, path)
    }
}

#[cfg(windows)]
fn is_reparse(meta: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    meta.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_reparse(_meta: &Metadata) -> bool {
    false
}

/// A symlink or a junction (any reparse point on Windows).
pub fn meta_is_link(meta: &Metadata) -> bool {
    meta.file_type().is_symlink() || is_reparse(meta)
}

fn stat_of(meta: &Metadata) -> Stat {
    let is_link = meta_is_link(meta);
    Stat {
        is_link,
        is_dir: !is_link && meta.is_dir(),
        mtime_ms: meta.modified().ok().map(system_ms),
        size: meta.len(),
    }
}

const RETRIES: u32 = 3;

fn transient(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::PermissionDenied | io::ErrorKind::ResourceBusy
    )
}

fn retried(path: &str, op: impl Fn() -> io::Result<()>) -> io::Result<()> {
    let mut last = op();
    for _ in 0..RETRIES {
        match &last {
            Err(err) if transient(err) => {
                clear_readonly(path);
                thread::sleep(Duration::from_millis(50));
                last = op();
            }
            _ => break,
        }
    }
    last
}

fn clear_readonly(path: &str) {
    if let Ok(meta) = fs::symlink_metadata(path) {
        let mut permissions = meta.permissions();
        if permissions.readonly() && !meta_is_link(&meta) {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

/// The real file system.
pub struct RealFs;

impl Fs for RealFs {
    fn exists(&self, path: &str) -> bool {
        fs::metadata(path).is_ok()
    }

    fn is_dir(&self, path: &str) -> bool {
        fs::metadata(path).is_ok_and(|m| m.is_dir())
    }

    fn dirs(&self, path: &str) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            let folder = kind.is_dir()
                || (kind.is_symlink()
                    && self.is_dir(&join_path(path, &entry.file_name().to_string_lossy())));
            if folder {
                names.push(entry.file_name().to_string_lossy().into_owned());
            }
        }
        Ok(names)
    }

    fn mtime_ms(&self, path: &str) -> Option<i64> {
        fs::metadata(path).ok()?.modified().ok().map(system_ms)
    }

    fn lstat(&self, path: &str) -> io::Result<Stat> {
        fs::symlink_metadata(path).map(|m| stat_of(&m))
    }

    fn read_dir(&self, path: &str) -> io::Result<Vec<Entry>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            out.push(Entry {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: kind.is_dir(),
                is_link: kind.is_symlink(),
            });
        }
        Ok(out)
    }

    fn rename(&self, from: &str, to: &str) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn remove_file(&self, path: &str) -> io::Result<()> {
        retried(path, || fs::remove_file(path))
    }

    fn remove_dir(&self, path: &str) -> io::Result<()> {
        retried(path, || fs::remove_dir(path))
    }

    fn remove_tree(&self, path: &str) -> io::Result<()> {
        retried(path, || fs::remove_dir_all(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{exists_itself, make_link, put};
    use tempfile::tempdir;

    #[test]
    fn a_link_is_a_leaf_in_lstat_and_read_dir_and_dirs_still_lists_it() {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let root = dir.path().to_string_lossy().into_owned();
        put(&root, "own.txt", "own");
        put(&root, "real/a.txt", "a");
        let link = join_path(&root, "lnk");
        make_link(&link, &outside.path().to_string_lossy());
        let fs = RealFs;
        let stat = fs.lstat(&link).unwrap();
        assert!(stat.is_link && !stat.is_dir);
        let mut entries = fs.read_dir(&root).unwrap();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let shape: Vec<(&str, bool, bool)> = entries
            .iter()
            .map(|e| (e.name.as_str(), e.is_dir, e.is_link))
            .collect();
        assert_eq!(
            shape,
            [
                ("lnk", false, true),
                ("own.txt", false, false),
                ("real", true, false)
            ]
        );
        let mut dirs = fs.dirs(&root).unwrap();
        dirs.sort();
        assert_eq!(dirs, ["lnk", "real"]);
        assert!(fs.exists(&link) && fs.is_dir(&link));
    }

    #[test]
    fn removing_a_link_leaves_its_target_alone() {
        let dir = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let root = dir.path().to_string_lossy().into_owned();
        let target = outside.path().to_string_lossy().into_owned();
        put(&target, "keep.txt", "precious");
        let link = join_path(&root, "lnk");
        make_link(&link, &target);
        let fs = RealFs;
        fs.remove_file(&link)
            .or_else(|_| fs.remove_dir(&link))
            .unwrap();
        assert!(!exists_itself(&link));
        assert_eq!(
            std::fs::read_to_string(join_path(&target, "keep.txt")).unwrap(),
            "precious"
        );
    }
}
