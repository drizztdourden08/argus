//! Paths that must stay inside a store: a request path is split into plain names and walked
//! from the store root, and nothing that leaves the root, is a link, or is a half-written file
//! is ever returned.

use std::path::{Path, PathBuf};

/// Endings of a file that is still being written or downloaded.
const UNFINISHED: [&str; 5] = ["tmp", "temp", "part", "partial", "crdownload"];

/// Whether a name is a temp name: hidden, a backup tilde, or an unfinished-file ending.
#[must_use]
pub fn is_temp_name(name: &str) -> bool {
    let ext = Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    name.starts_with('.')
        || name.ends_with('~')
        || ext.is_some_and(|e| UNFINISHED.contains(&e.as_str()))
}

/// Whether a path segment is a plain name: no dots-only, separators, drive or stream colons,
/// control characters, trailing dot or space (Windows reads those as another name), and no
/// temp name.
#[must_use]
pub fn segment_ok(seg: &str) -> bool {
    !seg.is_empty()
        && seg != "."
        && seg != ".."
        && !seg
            .chars()
            .any(|c| c.is_control() || "/\\:*?\"<>|".contains(c))
        && !seg.ends_with(['.', ' '])
        && !is_temp_name(seg)
}

/// The segments of a slash path when every one is a plain name.
#[must_use]
pub fn segments(rel: &str) -> Option<Vec<&str>> {
    let parts: Vec<&str> = rel.split('/').collect();
    parts.iter().all(|s| segment_ok(s)).then_some(parts)
}

/// The regular file `rel` names under `root`: every step is a plain name that is not a link,
/// and the real path is inside the real root. `None` otherwise (a folder included).
#[must_use]
pub fn file_under(root: &Path, rel: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for seg in segments(rel)? {
        path.push(seg);
        if std::fs::symlink_metadata(&path)
            .ok()?
            .file_type()
            .is_symlink()
        {
            return None;
        }
    }
    let real_root = std::fs::canonicalize(root).ok()?;
    let target = std::fs::canonicalize(&path).ok()?;
    (target.starts_with(real_root) && target.is_file()).then_some(path)
}

/// Whether the real path of `file` is inside the real path of `folder`.
#[must_use]
pub fn is_inside(folder: &Path, file: &Path) -> bool {
    match (std::fs::canonicalize(folder), std::fs::canonicalize(file)) {
        (Ok(folder), Ok(file)) => file.starts_with(folder),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        let root = dir.path().join("store");
        std::fs::create_dir_all(root.join("p/c/1.0.0")).expect("folders");
        std::fs::write(root.join("p/c/1.0.0/app.exe"), "bytes").expect("file");
        std::fs::write(dir.path().join("secret.txt"), "no").expect("outside file");
        dir
    }

    #[test]
    fn a_plain_path_resolves_and_a_folder_does_not() {
        let dir = store();
        let root = dir.path().join("store");
        assert!(file_under(&root, "p/c/1.0.0/app.exe").is_some());
        assert!(file_under(&root, "p/c/1.0.0").is_none());
        assert!(file_under(&root, "p/c/1.0.0/missing.exe").is_none());
    }

    #[test]
    fn dots_separators_absolute_paths_and_temp_names_are_refused() {
        let dir = store();
        let root = dir.path().join("store");
        for bad in [
            "../secret.txt",
            "p/../../secret.txt",
            "p/c/./1.0.0/app.exe",
            "/p/c/1.0.0/app.exe",
            "p//c/1.0.0/app.exe",
            "p\\c\\1.0.0\\app.exe",
            "C:/Windows/win.ini",
            "p/c/1.0.0/app.exe:stream",
            "p/c/1.0.0/app.exe.",
            "p/c/1.0.0/.hidden",
            "p/c/1.0.0/app.exe.tmp",
            "",
        ] {
            assert!(file_under(&root, bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn a_link_that_leaves_the_store_is_refused() {
        let dir = store();
        let root = dir.path().join("store");
        let link = root.join("p/c/1.0.0/out.txt");
        let made = make_link(&dir.path().join("secret.txt"), &link);
        if made {
            assert!(file_under(&root, "p/c/1.0.0/out.txt").is_none());
        }
    }

    #[cfg(unix)]
    fn make_link(target: &Path, link: &Path) -> bool {
        std::os::unix::fs::symlink(target, link).is_ok()
    }

    #[cfg(windows)]
    fn make_link(target: &Path, link: &Path) -> bool {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }

    #[test]
    fn temp_names_are_told_apart() {
        for temp in [".x", "a.tmp", "A.PART", "b~", "c.crdownload"] {
            assert!(is_temp_name(temp), "{temp}");
        }
        for plain in ["release.json", "app.exe", "tmp", "latest.yml"] {
            assert!(!is_temp_name(plain), "{plain}");
        }
    }
}
