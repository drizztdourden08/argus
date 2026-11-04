//! Links for the delete safety tests: a directory junction (a symlink off Windows), a file
//! symlink (Windows allows one only in Developer Mode or an elevated shell), and a sibling
//! folder full of files that a link points at and that must survive every delete.

use std::fs;
use std::path::Path;

/// The files of a sibling folder and what each holds.
const SIBLING_FILES: [(&str, &str); 3] = [
    ("keep.txt", "precious"),
    ("deep/more.txt", "also precious"),
    ("deep/deeper/last.txt", "the last one"),
];

/// A directory junction at `link` pointing to `target` (`mklink /J`); a symlink off Windows.
///
/// # Panics
/// When the link cannot be made.
pub fn junction(link: &Path, target: &Path) {
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent).expect("the folder of the link");
    }
    make_junction(link, target);
}

#[cfg(windows)]
fn make_junction(link: &Path, target: &Path) {
    use crate::process::hidden_command;

    let native = |p: &Path| p.to_string_lossy().replace('/', "\\");
    let status = hidden_command("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(native(link))
        .arg(native(target))
        .stdout(std::process::Stdio::null())
        .status()
        .expect("cmd runs");
    assert!(
        status.success(),
        "mklink /J {} {}",
        link.display(),
        target.display()
    );
}

#[cfg(not(windows))]
fn make_junction(link: &Path, target: &Path) {
    std::os::unix::fs::symlink(target, link).expect("a symlink");
}

/// A file symlink at `link` pointing to `target`. False when this machine does not allow one.
#[must_use]
pub fn file_symlink(link: &Path, target: &Path) -> bool {
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_file(target, link);
    #[cfg(not(windows))]
    let made = std::os::unix::fs::symlink(target, link);
    if let Err(err) = &made {
        eprintln!(
            "skipped: a file symlink cannot be made on this machine ({err}); Windows needs Developer Mode or an elevated shell"
        );
    }
    made.is_ok()
}

/// Fills `dir` with the files of a sibling folder.
///
/// # Panics
/// When a file cannot be written.
pub fn fill_sibling(dir: &Path) {
    for (rel, text) in SIBLING_FILES {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().expect("a parent")).expect("the folders");
        fs::write(path, text).expect("the file");
    }
}

/// Panics unless every file [`fill_sibling`] wrote is still there with its text.
///
/// # Panics
/// When a file of the sibling folder is gone or changed.
pub fn assert_sibling_intact(dir: &Path) {
    for (rel, text) in SIBLING_FILES {
        let got = fs::read_to_string(dir.join(rel));
        assert_eq!(
            got.ok().as_deref(),
            Some(text),
            "{} must survive the delete",
            dir.join(rel).display()
        );
    }
}

/// Does the path exist as itself, link or not (a link is not followed)?
#[must_use]
pub fn exists_itself(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}
