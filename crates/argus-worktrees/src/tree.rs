//! Deleting folder trees without ever following a link.
//!
//! Every folder is looked at with `lstat` before it is entered. A symlink or junction is removed as a
//! link, so what it points to is never touched.

use crate::fs::Fs;
use crate::paths::join_path;
use std::io;

/// Remove a link as a link: unlink it, else remove it as a folder junction.
pub fn unlink_link(fs: &dyn Fs, path: &str) -> io::Result<()> {
    match fs.remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(_) => fs.remove_dir(path),
    }
}

/// Every link below `dir` (pnpm's `node_modules` is full of junctions) is removed as a link, walking real folders only.
/// `.git` is skipped.
pub fn drop_links(fs: &dyn Fs, dir: &str) -> io::Result<()> {
    for entry in fs.read_dir(dir)? {
        if entry.name == ".git" || !(entry.is_dir || entry.is_link) {
            continue;
        }
        let path = join_path(dir, &entry.name);
        let Ok(stat) = fs.lstat(&path) else {
            continue;
        };
        if stat.is_link {
            unlink_link(fs, &path)?;
        } else if stat.is_dir {
            drop_links(fs, &path)?;
        }
    }
    Ok(())
}

/// Delete a folder and everything in it. A link inside is unlinked, never entered.
pub fn remove_tree(fs: &dyn Fs, path: &str) -> io::Result<()> {
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
            remove_tree(fs, &child)?;
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
