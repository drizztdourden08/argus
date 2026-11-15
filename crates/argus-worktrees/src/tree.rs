//! Deleting folder trees without ever following a link.
//!
//! Every folder is looked at with `lstat` before it is entered. A symlink or junction is removed as a
//! link, so what it points to is never touched.

use crate::fs::Fs;
use crate::paths::join_path;
use std::io;

/// Remove a link as a link, never what it points to (see [`Fs::remove_link`]).
pub fn unlink_link<F: Fs + ?Sized>(fs: &F, path: &str) -> io::Result<()> {
    fs.remove_link(path)
}

/// Every link below `dir` (pnpm's `node_modules` is full of junctions) is removed as a link, walking real folders only.
/// Only the worktree's own top-level `.git` (a file or a folder, not a link) is left alone; a link anywhere else, a
/// `.git` named link included, goes as a link. The answer is how many links went.
pub fn drop_links(fs: &dyn Fs, dir: &str) -> io::Result<usize> {
    drop_links_in(fs, dir, true)
}

fn drop_links_in(fs: &dyn Fs, dir: &str, top: bool) -> io::Result<usize> {
    let entries = fs.read_dir(dir)?;
    if fs.lstat(dir)?.is_link {
        return Err(io::Error::other(format!(
            "{dir} became a link while it was being read"
        )));
    }
    let mut dropped = 0;
    for entry in entries {
        if (top && entry.name == ".git" && !entry.is_link) || !(entry.is_dir || entry.is_link) {
            continue;
        }
        let path = join_path(dir, &entry.name);
        let Ok(stat) = fs.lstat(&path) else {
            continue;
        };
        if stat.is_link {
            unlink_link(fs, &path)?;
            dropped += 1;
        } else if stat.is_dir {
            dropped += drop_links_in(fs, &path, false)?;
        }
    }
    Ok(dropped)
}

/// Delete a folder and everything in it. A link inside is unlinked, never entered. The file system does the walk
/// (`RealFs` uses the standard library's handle-based delete, which never follows a link).
pub fn remove_tree(fs: &dyn Fs, path: &str) -> io::Result<()> {
    fs.remove_tree(path)
}

/// The walk behind [`Fs::remove_tree`] for a file system with no native delete: `lstat` before every descent, the
/// folder itself included (a folder that is a link goes as a link).
pub fn walk_remove<F: Fs + ?Sized>(fs: &F, path: &str) -> io::Result<()> {
    if fs.lstat(path)?.is_link {
        return unlink_link(fs, path);
    }
    for entry in fs.read_dir(path)? {
        let child = join_path(path, &entry.name);
        let stat = match fs.lstat(&child) {
            Ok(stat) => stat,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };
        if stat.is_link {
            unlink_link(fs, &child)?;
        } else if stat.is_dir {
            walk_remove(fs, &child)?;
        } else {
            fs.remove_file(&child)?;
        }
    }
    fs.remove_dir(path)
}

/// One top-level entry of a folder being deleted: a link as a link, a folder after its inner links, a file.
pub fn remove_entry(fs: &dyn Fs, path: &str, drop_links_first: bool) -> io::Result<()> {
    let stat = match fs.lstat(path) {
        Ok(stat) => stat,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    if stat.is_link {
        return unlink_link(fs, path);
    }
    if !stat.is_dir {
        return fs.remove_file(path);
    }
    if drop_links_first {
        drop_links(fs, path)?;
    }
    remove_tree(fs, path)
}

// test-tag: standard
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::RealFs;
    use crate::testkit::{exists_itself, make_link, put, root_of};
    use tempfile::tempdir;

    #[test]
    fn a_tree_goes_and_the_target_of_a_link_inside_it_survives() {
        let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
        let (root, target) = (root_of(&dir), root_of(&outside));
        put(&target, "keep.txt", "precious");
        put(&target, "deeper/more.txt", "also");
        put(&root, "victim/a.txt", "a");
        put(&root, "victim/node_modules/pkg/index.js", "x");
        make_link(&join_path(&root, "victim/node_modules/ext"), &target);
        make_link(&join_path(&root, "victim/top"), &target);
        remove_tree(&RealFs, &join_path(&root, "victim")).unwrap();
        assert!(!exists_itself(&join_path(&root, "victim")));
        assert_eq!(
            std::fs::read_to_string(join_path(&target, "keep.txt")).unwrap(),
            "precious"
        );
        assert!(exists_itself(&join_path(&target, "deeper/more.txt")));
    }

    #[test]
    fn drop_links_removes_the_links_and_nothing_else() {
        let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
        let (root, target) = (root_of(&dir), root_of(&outside));
        put(&target, "keep.txt", "precious");
        put(&root, "wt/node_modules/.pnpm/pkg@1/index.js", "x");
        put(&root, "wt/.git", "gitdir: x");
        make_link(&join_path(&root, "wt/node_modules/ext"), &target);
        make_link(&join_path(&root, "wt/lnk-top"), &target);
        drop_links(&RealFs, &join_path(&root, "wt")).unwrap();
        assert!(!exists_itself(&join_path(&root, "wt/node_modules/ext")));
        assert!(!exists_itself(&join_path(&root, "wt/lnk-top")));
        assert!(exists_itself(&join_path(
            &root,
            "wt/node_modules/.pnpm/pkg@1/index.js"
        )));
        assert!(exists_itself(&join_path(&root, "wt/.git")));
        assert_eq!(
            std::fs::read_to_string(join_path(&target, "keep.txt")).unwrap(),
            "precious"
        );
    }

    #[test]
    fn only_the_worktrees_own_top_level_git_is_left_a_git_named_link_deeper_or_at_the_top_goes() {
        let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
        let (root, target) = (root_of(&dir), root_of(&outside));
        put(&target, "keep.txt", "precious");
        put(&root, "wt/.git", "gitdir: x");
        make_link(&join_path(&root, "wt/sub/.git"), &target);
        drop_links(&RealFs, &join_path(&root, "wt")).unwrap();
        assert!(
            exists_itself(&join_path(&root, "wt/.git")),
            "the worktree's own .git stays"
        );
        assert!(
            !exists_itself(&join_path(&root, "wt/sub/.git")),
            "a .git link deeper goes as a link"
        );
        put(&root, "linked/a.txt", "a");
        make_link(&join_path(&root, "linked/.git"), &target);
        drop_links(&RealFs, &join_path(&root, "linked")).unwrap();
        assert!(
            !exists_itself(&join_path(&root, "linked/.git")),
            "a top-level .git that is a link goes as a link"
        );
        assert!(exists_itself(&join_path(&target, "keep.txt")));
    }

    #[test]
    fn remove_entry_unlinks_a_top_level_link_and_survives_a_missing_path() {
        let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
        let (root, target) = (root_of(&dir), root_of(&outside));
        put(&target, "keep.txt", "precious");
        let link = join_path(&root, "lnk");
        make_link(&link, &target);
        remove_entry(&RealFs, &link, true).unwrap();
        remove_entry(&RealFs, &join_path(&root, "nothing"), true).unwrap();
        assert!(!exists_itself(&link));
        assert!(exists_itself(&join_path(&target, "keep.txt")));
    }
}
