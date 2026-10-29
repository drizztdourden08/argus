//! Deleting inside the cache root and nowhere else. Every path is checked against the root first; a folder is moved
//! out of sight into `.tmp/` before it is deleted, so a reader never meets half an entry; a link is never followed.

use crate::segment::{join, parent};
use crate::tmp::TMP_DIR;
use argus_worktrees::{Fs, clean_path, path_key};
use std::io;

/// Why a delete did not happen.
#[derive(Debug, thiserror::Error)]
pub enum RemoveError {
    /// The path is not strictly inside the cache root, or is spelled with a `.` or `..` segment.
    #[error("{0} is outside the cache root")]
    Outside(String),
    /// The path is a link; a link is never followed.
    #[error("{0} is a link")]
    Link(String),
    #[error(transparent)]
    Io(#[from] io::Error),
}

fn folded(path: &str) -> String {
    path_key(path, cfg!(windows))
}

/// The root has a name of its own: `/` and `C:/` are not a cache.
fn is_usable_root(root: &str) -> bool {
    let clean = clean_path(root);
    clean
        .split('/')
        .filter(|s| !s.is_empty() && !s.ends_with(':'))
        .count()
        >= 2
}

/// Is `path` strictly below `root`, with no `.` or `..` in it?
pub fn inside(root: &str, path: &str) -> bool {
    let clean = clean_path(path);
    let plain = !clean.split('/').any(|s| s == ".." || s == ".");
    let prefix = format!("{}/", folded(root).trim_end_matches('/'));
    is_usable_root(root) && plain && folded(&clean).starts_with(&prefix)
}

/// Inside the root, and neither the path nor a folder between the root and it is a link: a junction on the way (a
/// `.tmp` or a version folder made a link) would lead the delete outside the cache.
fn checked(fs: &dyn Fs, root: &str, path: &str) -> Result<(), RemoveError> {
    if !inside(root, path) {
        return Err(RemoveError::Outside(path.to_owned()));
    }
    let base = clean_path(root).trim_end_matches('/').to_owned();
    let clean = clean_path(path);
    let rest = clean.get(base.len() + 1..).unwrap_or_default();
    let mut at = base;
    for segment in rest.split('/') {
        at = join(&at, segment);
        if fs.lstat(&at)?.is_link {
            return Err(RemoveError::Link(at));
        }
    }
    Ok(())
}

/// Deletes a folder of the cache and everything in it.
pub fn delete_tree(fs: &dyn Fs, root: &str, path: &str) -> Result<(), RemoveError> {
    checked(fs, root, path)?;
    fs.remove_tree(path)?;
    Ok(())
}

/// Moves a folder of the cache to `.tmp/<name>` in one rename and answers its new path.
pub fn park(fs: &dyn Fs, root: &str, path: &str, name: &str) -> Result<String, RemoveError> {
    checked(fs, root, path)?;
    let tmp = join(root, TMP_DIR);
    std::fs::create_dir_all(&tmp)?;
    checked(fs, root, &tmp)?;
    let parked = join(&tmp, name);
    if !inside(root, &parked) {
        return Err(RemoveError::Outside(parked));
    }
    fs.rename(path, &parked)?;
    Ok(parked)
}

/// Removes the folder above `path` when it holds nothing, as a version or name folder of a deleted entry.
pub fn drop_if_empty(fs: &dyn Fs, root: &str, folder: &str) -> bool {
    let empty = fs.read_dir(folder).is_ok_and(|entries| entries.is_empty());
    empty && inside(root, folder) && fs.remove_dir(folder).is_ok()
}

/// The version folder and the name folder above an entry, nearest first.
pub fn parents_of(entry_path: &str) -> Vec<String> {
    let version = parent(entry_path);
    let name = version.and_then(parent);
    [version, name]
        .into_iter()
        .flatten()
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use argus_worktrees::RealFs;
    use std::fs;

    fn folders(dir: &tempfile::TempDir) -> (String, String) {
        let base = clean_path(&dir.path().to_string_lossy());
        (format!("{base}/a/cache"), format!("{base}/a/other"))
    }

    #[test]
    fn only_a_path_strictly_below_the_root_is_inside() {
        let root = "C:/Users/me/.brock/cache/build-cache";
        assert!(inside(root, &format!("{root}/package/a/1.0.0/node-aa")));
        assert!(!inside(root, root));
        assert!(!inside(
            root,
            "C:/Users/me/.brock/cache/build-cache-other/x"
        ));
        assert!(!inside(root, "C:/Users/me/other"));
        assert!(!inside(root, &format!("{root}/package/../../x")));
        assert!(!inside(root, &format!("{root}/./package")));
    }

    #[test]
    fn a_drive_or_filesystem_root_is_no_cache() {
        assert!(!inside("C:/", "C:/Windows"));
        assert!(!inside("/", "/etc"));
        assert!(!inside("C:/temp", "C:/temp/x"));
    }

    #[test]
    fn a_folder_outside_the_root_is_refused_and_left_alone() {
        let dir = tempfile::tempdir().expect("temp dir");
        let (root, other) = folders(&dir);
        fs::create_dir_all(&root).expect("root");
        fs::create_dir_all(&other).expect("other");
        fs::write(format!("{other}/keep.txt"), "x").expect("file");
        let deleted = delete_tree(&RealFs, &root, &other);
        assert!(matches!(deleted, Err(RemoveError::Outside(_))));
        let parked = park(&RealFs, &root, &other, "taken");
        assert!(matches!(parked, Err(RemoveError::Outside(_))));
        assert!(fs::exists(format!("{other}/keep.txt")).expect("exists"));
        assert!(!fs::exists(format!("{root}/.tmp/taken")).expect("exists"));
    }

    #[test]
    fn a_folder_is_parked_whole_and_its_empty_parents_go() {
        let dir = tempfile::tempdir().expect("temp dir");
        let (root, _) = folders(&dir);
        let entry = format!("{root}/package/n/1.0.0/node-aa");
        fs::create_dir_all(format!("{entry}/files")).expect("entry");
        fs::write(format!("{entry}/files/a.js"), "x").expect("file");
        let parked = park(&RealFs, &root, &entry, "p-1").expect("park");
        assert_eq!(parked, format!("{root}/.tmp/p-1"));
        assert!(fs::exists(format!("{parked}/files/a.js")).expect("exists"));
        for folder in parents_of(&entry) {
            assert!(drop_if_empty(&RealFs, &root, &folder));
        }
        assert!(!fs::exists(format!("{root}/package/n")).expect("exists"));
        assert!(fs::exists(format!("{root}/package")).expect("exists"));
    }
}
