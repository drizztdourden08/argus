//! A state file is written in one place only: `argus_core::fs`. No crate keeps its own
//! write-temp-then-rename (the Node fix `e9fc73f` is a flush before the rename, and a second copy
//! of the routine is where a flush goes missing). The folder and file renames that are not state
//! writes are listed here by name.

use std::fs;
use std::path::{Path, PathBuf};

/// Source files that rename for another reason: the board save hands `fs::rename` to the helper,
/// the log rotation, the database copy (flushed by
/// `flush_file` first) and the worktree folder moves.
const RENAMES_ALLOWED: [&str; 7] = [
    "argus-board/src/store/io.rs",
    "argus-worktrees/src/leftover_remove.rs",
    "argus-core/src/fs.rs",
    "argus-core/src/log.rs",
    "argus-store/src/snapshot.rs",
    "argus-worktrees/src/fs.rs",
    "argus-worktrees/src/finish.rs",
];

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() && !matches!(name.as_str(), "tests" | "specs" | "target") {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") && name != "testkit.rs" {
            out.push(path);
        }
    }
}

/// Every non-test source file of the workspace crates, with its path under `crates/` and its text
/// up to the first `#[cfg(test)]`.
fn workspace_sources() -> Vec<(String, String)> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut files = Vec::new();
    for krate in fs::read_dir(crates).unwrap().flatten() {
        let src = krate.path().join("src");
        if src.is_dir() {
            sources(&src, &mut files);
        }
    }
    files
        .into_iter()
        .map(|file| {
            let rel = file
                .strip_prefix(crates)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&file).unwrap();
            let live = text.split("#[cfg(test)]").next().unwrap_or_default();
            (rel, live.to_owned())
        })
        .collect()
}

#[test]
fn no_crate_names_its_own_temp_file() {
    let mut own: Vec<String> = workspace_sources()
        .into_iter()
        .filter(|(rel, text)| rel != "argus-core/src/fs.rs" && text.contains(".tmp"))
        .map(|(rel, _)| rel)
        .collect();
    own.sort();
    assert!(own.is_empty(), "write through argus_core::fs: {own:?}");
}

#[test]
fn only_the_listed_files_rename() {
    let mut renaming: Vec<String> = workspace_sources()
        .into_iter()
        .filter(|(rel, text)| text.contains("rename(") && !RENAMES_ALLOWED.contains(&rel.as_str()))
        .map(|(rel, _)| rel)
        .collect();
    renaming.sort();
    assert!(
        renaming.is_empty(),
        "a state file is written with argus_core::fs::write_atomic: {renaming:?}"
    );
}
