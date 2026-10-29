//! Puts a new copy of the binary in place of the installed one, also while a process runs it.
//!
//! On Windows a running exe cannot be overwritten or deleted, but it can be renamed. So when the
//! atomic replace fails, the current target is renamed aside to `<name>.old-<UTC stamp>` in the
//! same folder and the new copy takes its place. Running processes keep the old image, new starts
//! pick up the new file. The set-aside copies are deleted on a later install, as far as they are
//! no longer in use. The folder never lacks its binary: when the new copy cannot be put in place
//! after the rename aside, the old one is renamed back. When the rename aside fails too, nothing
//! was changed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use argus_core::fs::write_atomic_with;

type Rename<'a> = &'a dyn Fn(&Path, &Path) -> io::Result<()>;

/// How the new copy got in place.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Replaced {
    /// The atomic replace worked.
    InPlace,
    /// The old copy was renamed to this path first, because it was in use.
    SetAside(PathBuf),
}

fn refusal(target: &Path, why: &io::Error) -> String {
    format!(
        "{} cannot be replaced ({why}). It is most likely in use by a running argus: run `argus stop`, then `argus self-install` again. The installed copy was left as it was.",
        target.display()
    )
}

fn aside_of(target: &Path, stamp: &str) -> PathBuf {
    let mut name = target.as_os_str().to_os_string();
    name.push(format!(".old-{stamp}"));
    PathBuf::from(name)
}

/// Copies `source` over `target`, or sets the in-use `target` aside and puts the copy in its
/// place, or leaves `target` as it was and says why.
pub(super) fn replace(source: &Path, target: &Path, stamp: &str) -> Result<Replaced, String> {
    replace_with(source, target, stamp, &|from, to| fs::rename(from, to))
}

fn replace_with(
    source: &Path,
    target: &Path,
    stamp: &str,
    rename: Rename<'_>,
) -> Result<Replaced, String> {
    let bytes =
        fs::read(source).map_err(|e| format!("{} cannot be read: {e}", source.display()))?;
    match write_atomic_with(target, &bytes, rename) {
        Ok(()) => Ok(Replaced::InPlace),
        Err(blocked) => set_aside(target, &bytes, &aside_of(target, stamp), &blocked, rename),
    }
}

fn set_aside(
    target: &Path,
    bytes: &[u8],
    aside: &Path,
    blocked: &io::Error,
    rename: Rename<'_>,
) -> Result<Replaced, String> {
    rename(target, aside).map_err(|_| refusal(target, blocked))?;
    let Err(failed) = write_atomic_with(target, bytes, rename) else {
        return Ok(Replaced::SetAside(aside.to_path_buf()));
    };
    match rename(aside, target) {
        Ok(()) => Err(refusal(target, &failed)),
        Err(back) => Err(format!(
            "{} could not be replaced ({failed}) and the old copy could not be renamed back ({back}); it is at {}",
            target.display(),
            aside.display()
        )),
    }
}

/// Deletes the copies an earlier install set aside. A copy still in use stays for next time.
pub(super) fn remove_old_copies(dir: &Path, file_name: &str) {
    let prefix = format!("{file_name}.old-");
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAMP: &str = "20261028T101500Z";

    fn names(dir: &Path) -> Vec<String> {
        let mut all: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        all.sort();
        all
    }

    fn denied() -> io::Error {
        io::Error::from(io::ErrorKind::PermissionDenied)
    }

    /// A temp folder with the new build, and an installed `argus.exe` holding "running".
    fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("new.bin");
        fs::write(&source, b"new build").unwrap();
        let stable = dir.path().join("stable");
        fs::create_dir_all(&stable).unwrap();
        let target = stable.join("argus.exe");
        fs::write(&target, b"running").unwrap();
        (dir, source, target)
    }

    fn aside_name() -> String {
        format!("argus.exe.old-{STAMP}")
    }

    #[test]
    fn a_free_target_is_replaced_in_place() {
        let (_dir, source, target) = setup();
        let done = replace(&source, &target, STAMP).unwrap();
        assert_eq!(done, Replaced::InPlace);
        assert_eq!(fs::read(&target).unwrap(), b"new build");
        assert_eq!(names(target.parent().unwrap()), ["argus.exe"]);
    }

    /// A running exe: renaming it works, replacing it does not.
    #[test]
    fn an_in_use_target_is_set_aside_and_the_new_copy_takes_its_place() {
        let (_dir, source, target) = setup();
        let in_use = |from: &Path, to: &Path| {
            if to == target && to.exists() {
                Err(denied())
            } else {
                fs::rename(from, to)
            }
        };
        let done = replace_with(&source, &target, STAMP, &in_use).unwrap();
        let aside = target.with_file_name(aside_name());
        assert_eq!(done, Replaced::SetAside(aside.clone()));
        assert_eq!(fs::read(&target).unwrap(), b"new build");
        assert_eq!(fs::read(&aside).unwrap(), b"running");
        assert_eq!(
            names(target.parent().unwrap()),
            ["argus.exe".to_owned(), aside_name()]
        );
    }

    #[test]
    fn a_failed_second_rename_puts_the_old_copy_back() {
        let (_dir, source, target) = setup();
        let never_in = |from: &Path, to: &Path| {
            if to == target && from.extension().is_some_and(|e| e == "tmp") {
                Err(denied())
            } else {
                fs::rename(from, to)
            }
        };
        let why = replace_with(&source, &target, STAMP, &never_in).unwrap_err();
        assert!(why.contains("left as it was"), "{why}");
        assert_eq!(fs::read(&target).unwrap(), b"running");
        assert_eq!(names(target.parent().unwrap()), ["argus.exe"]);
    }

    #[test]
    fn a_failed_rename_aside_leaves_the_target_untouched() {
        let (_dir, source, target) = setup();
        let frozen = |from: &Path, to: &Path| {
            if from == target || to == target {
                Err(denied())
            } else {
                fs::rename(from, to)
            }
        };
        let why = replace_with(&source, &target, STAMP, &frozen).unwrap_err();
        assert!(why.contains("argus stop"), "{why}");
        assert_eq!(fs::read(&target).unwrap(), b"running");
        assert_eq!(names(target.parent().unwrap()), ["argus.exe"]);
    }

    #[test]
    fn old_copies_go_and_everything_else_stays() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "argus.exe",
            "argus.exe.old-1",
            "argus.exe.old-2",
            "notes.txt",
        ] {
            fs::write(dir.path().join(name), b"x").unwrap();
        }
        remove_old_copies(dir.path(), "argus.exe");
        assert_eq!(names(dir.path()), ["argus.exe", "notes.txt"]);
        remove_old_copies(&dir.path().join("missing"), "argus.exe");
    }
}
