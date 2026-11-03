//! The guard of a check run: `brock ci` once removed tracked files of a main checkout. Before a
//! run starts, the tracked changes of the folder it runs in (and of the project root when that
//! differs) are kept, as `git status --porcelain --untracked-files=no` prints them. When the run
//! ends they are read again: a line that was not there before (a tracked file the run changed or
//! deleted) is a harm, and the starter fails the run with the note this module writes.

use std::time::Duration;

use argus_worktrees::{GitExec, SystemExec};

const GIT_TIMEOUT: Duration = Duration::from_secs(120);
/// How many paths a note names before it counts the rest.
const SHOWN: usize = 5;

/// The tracked changes of a folder, one porcelain line each, or what git said when it could not
/// read it.
fn tracked_changes(dir: &str) -> Result<Vec<String>, String> {
    let args = ["status", "--porcelain", "--untracked-files=no"].map(str::to_owned);
    let out = SystemExec::default().exec(dir, &args, GIT_TIMEOUT);
    if !out.ok() {
        return Err(out.failure_text());
    }
    let lines = out.stdout.lines().filter(|l| !l.trim().is_empty());
    Ok(lines.map(str::to_owned).collect())
}

/// The path of a porcelain line (`XY path`).
fn path_of(line: &str) -> &str {
    line.get(3..).unwrap_or(line)
}

/// The paths of the lines that are new after the run.
fn new_paths<'a>(before: &[String], after: &'a [String]) -> Vec<&'a str> {
    let fresh = after.iter().filter(|line| !before.contains(line));
    fresh.map(|line| path_of(line)).collect()
}

fn note(dir: &str, paths: &[&str]) -> String {
    let shown = paths
        .iter()
        .take(SHOWN)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    let more = paths.len().saturating_sub(SHOWN);
    let rest = if more > 0 {
        format!(" (and {more} more)")
    } else {
        String::new()
    };
    format!("brock ci changed tracked files in {dir}: {shown}{rest}")
}

/// What one watched folder says after the run: its note, when the run harmed it.
fn judged(dir: &str, before: &[String]) -> Option<String> {
    match tracked_changes(dir) {
        Ok(after) => {
            let paths = new_paths(before, &after);
            (!paths.is_empty()).then(|| note(dir, &paths))
        }
        Err(why) => Some(format!(
            "brock ci changed tracked files in {dir}: git status no longer reads it ({why})"
        )),
    }
}

/// The tracked changes of the folders a run can touch, before it ran. A folder git cannot read
/// before the run (not a repository) is not watched.
pub(super) struct Before(Vec<(String, Vec<String>)>);

impl Before {
    /// Reads the folders now. Blocking: git runs.
    pub(super) fn take(dirs: Vec<String>) -> Before {
        let read = dirs.into_iter().filter_map(|dir| {
            let lines = tracked_changes(&dir).ok()?;
            Some((dir, lines))
        });
        Before(read.collect())
    }

    /// The note when the run changed tracked files in a watched folder. Blocking: git runs.
    pub(super) fn verdict(&self) -> Option<String> {
        let notes: Vec<String> = self
            .0
            .iter()
            .filter_map(|(dir, before)| judged(dir, before))
            .collect();
        (!notes.is_empty()).then(|| notes.join("; "))
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: which paths a guard run changed, and the note that names them.
    use super::*;

    fn lines(all: &[&str]) -> Vec<String> {
        all.iter().map(|l| (*l).to_owned()).collect()
    }

    #[test]
    fn only_lines_new_after_the_run_count_and_a_deletion_of_a_changed_file_is_new() {
        let before = lines(&[" M src/a.rs", " M src/b.rs"]);
        let after = lines(&[" M src/a.rs", " D src/b.rs", " D README.md"]);
        assert_eq!(new_paths(&before, &after), ["src/b.rs", "README.md"]);
        assert_eq!(new_paths(&before, &before), Vec::<&str>::new());
    }

    #[test]
    fn a_note_names_the_folder_and_the_first_paths() {
        let paths = ["a", "b", "c", "d", "e", "f", "g"];
        assert_eq!(
            note("X:/p", &paths),
            "brock ci changed tracked files in X:/p: a, b, c, d, e (and 2 more)"
        );
        assert_eq!(
            note("X:/p", &["a"]),
            "brock ci changed tracked files in X:/p: a"
        );
    }
}
