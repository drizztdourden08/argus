//! Folder lookups of `project.mjs`: walking up from a folder, the git root, and the comparable
//! form of a path.

use std::path::{Path, PathBuf};

use crate::cli::resolve::resolve;

/// The nearest folder at or above `start` for which `found` is true.
pub fn walk_up(start: &Path, found: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut dir = resolve(Path::new(""), start);
    loop {
        if found(&dir) {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// The nearest folder at or above `cwd` that holds `name`.
#[must_use]
pub fn holder_of(cwd: &Path, name: &str) -> Option<PathBuf> {
    walk_up(cwd, |d| d.join(name).exists())
}

/// The git checkout holding `cwd`: the nearest folder with a `.git`, else `cwd` itself.
#[must_use]
pub fn git_root(cwd: &Path) -> PathBuf {
    holder_of(cwd, ".git").unwrap_or_else(|| resolve(Path::new(""), cwd))
}

/// A path in the form two paths are compared in: absolute, forward slashes, lower case on
/// Windows.
#[must_use]
pub fn norm(base: &Path, p: &Path) -> String {
    let s = resolve(base, p).to_string_lossy().replace('\\', "/");
    if cfg!(windows) { s.to_lowercase() } else { s }
}

/// Whether `dir` (already normalized) is `root` or inside it.
#[must_use]
pub fn contains(base: &Path, root: &str, dir: &str) -> bool {
    let root = norm(base, Path::new(root));
    dir == root || dir.starts_with(&format!("{root}/"))
}
