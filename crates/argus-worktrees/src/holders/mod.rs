//! Who holds a folder that could not be renamed or deleted, so the refusal names it instead of "something".
//!
//! Two sources, both read only when a rename or a delete was refused (access denied, a sharing or lock violation):
//! - Argus's own live runs whose working directory is the folder, above it or inside it: a dev server or a file watcher
//!   there holds folder handles, and Windows names no process for a folder held that way
//! - Windows' Restart Manager over a bounded sample of the files below the folder: the processes that have one open

mod restart;

use crate::error::{ErrorCode, WtError, io_code};
use crate::fs::Fs;
use crate::guards::short_cmd;
use crate::paths::{base_name, is_below, join_path, same_path};
use crate::ports::LiveRun;
use crate::service::Service;
use std::collections::VecDeque;
use std::io;

/// Files the Restart Manager is asked about.
const SAMPLE_FILES: usize = 200;
/// Folders the sample walks at most.
const SAMPLE_DIRS: usize = 2000;
/// Holders a sentence names; the rest are counted.
const NAMED_MAX: usize = 4;

/// A rename or a delete refused because something has the folder or a file in it open.
pub(crate) fn is_held(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::PermissionDenied | io::ErrorKind::ResourceBusy
    ) || (cfg!(windows) && matches!(err.raw_os_error(), Some(5 | 32 | 33)))
}

/// Up to `SAMPLE_FILES` files below the folder, breadth first, links never followed.
fn sample_files(fs: &dyn Fs, path: &str) -> Vec<String> {
    let mut files = Vec::new();
    let mut dirs = VecDeque::from([path.to_string()]);
    let mut walked = 0;
    while let Some(dir) = dirs.pop_front() {
        walked += 1;
        if walked > SAMPLE_DIRS || files.len() >= SAMPLE_FILES {
            break;
        }
        let Ok(entries) = fs.read_dir(&dir) else {
            continue;
        };
        for entry in entries.into_iter().filter(|e| !e.is_link) {
            let child = join_path(&dir, &entry.name);
            if entry.is_dir {
                dirs.push_back(child);
            } else if files.len() < SAMPLE_FILES {
                files.push(child);
            }
        }
    }
    files
}

fn run_text(run: &LiveRun) -> String {
    format!(
        "{} ({}, {} in {})",
        run.id,
        run.project,
        short_cmd(&run.cmd),
        run.cwd
    )
}

/// `a`, `a and b`, `a, b and c`, the ones past the limit counted.
fn listed_text(all: &[String]) -> Option<String> {
    let mut shown: Vec<String> = all.iter().take(NAMED_MAX).cloned().collect();
    let more = all.len().saturating_sub(NAMED_MAX);
    if more > 0 {
        shown.push(format!("{more} more"));
    }
    let last = shown.pop()?;
    if shown.is_empty() {
        return Some(last);
    }
    Some(format!("{} and {last}", shown.join(", ")))
}

impl Service {
    /// Argus's live runs whose working directory is the folder, one of its ancestors, or inside it.
    fn runs_holding(&self, path: &str) -> Vec<String> {
        let ci = self.ci();
        self.deps
            .procs
            .runs()
            .iter()
            .filter(|run| !run.cwd.trim().is_empty())
            .filter(|run| {
                same_path(&run.cwd, path, ci)
                    || is_below(path, &run.cwd, ci)
                    || is_below(&run.cwd, path, ci)
            })
            .map(run_text)
            .collect()
    }

    /// The processes Windows says have a file below the folder open, by their executable's name.
    fn processes_holding(&self, path: &str) -> Vec<String> {
        let held = restart::file_holders(&sample_files(self.deps.fs.as_ref(), path));
        if held.is_empty() {
            return Vec::new();
        }
        let known = self.deps.procs.list().unwrap_or_default();
        let mut out: Vec<String> = Vec::new();
        for holder in held {
            let name = known
                .iter()
                .find(|p| p.pid == holder.pid && !p.name.is_empty())
                .map_or(holder.app, |p| base_name(&p.name));
            let text = format!("pid {} {name}", holder.pid);
            if !out.contains(&text) {
                out.push(text);
            }
        }
        out
    }

    /// Who holds the folder, as a phrase (`R-17 (tessera, pnpm dev in X:\tessera) and pid 1234 node.exe`); None when
    /// neither source names anyone.
    pub(crate) fn holders_of(&self, path: &str) -> Option<String> {
        let all: Vec<String> = self
            .runs_holding(path)
            .into_iter()
            .chain(self.processes_holding(path))
            .collect();
        listed_text(&all)
    }

    /// What a delete that something holds adds to its sentence: who, and what to do. None when it was not held, or
    /// nobody is named.
    pub(crate) fn held_note(&self, path: &str, err: &io::Error) -> Option<String> {
        if !is_held(err) {
            return None;
        }
        self.holders_of(path)
            .map(|who| format!("{path} is held by {who}: stop it and ask again"))
    }

    /// The refusal of a rename before a deletion: who holds the folder when Argus can tell, else what to look for.
    pub(crate) fn rename_refusal(&self, path: &str, err: &io::Error) -> WtError {
        let code = io_code(err);
        let message = match self.held_note(path, err) {
            Some(note) => format!(
                "{path} could not be renamed for deletion ({code}): {note}; nothing was deleted"
            ),
            None => format!(
                "{path} could not be renamed for deletion ({code}): something has it open or locked. Close it and try again; nothing was deleted"
            ),
        };
        WtError::new(ErrorCode::InUse, message)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::RealFs;
    use crate::testkit::{put, root_of};

    #[test]
    fn holders_read_as_a_short_list() {
        let names = |n: usize| (1..=n).map(|i| format!("h{i}")).collect::<Vec<_>>();
        assert_eq!(listed_text(&names(0)), None);
        assert_eq!(listed_text(&names(1)).unwrap(), "h1");
        assert_eq!(listed_text(&names(2)).unwrap(), "h1 and h2");
        assert_eq!(listed_text(&names(3)).unwrap(), "h1, h2 and h3");
        assert_eq!(listed_text(&names(6)).unwrap(), "h1, h2, h3, h4 and 2 more");
    }

    #[test]
    fn the_sample_is_bounded_and_breadth_first() {
        let dir = tempfile::tempdir().unwrap();
        let root = root_of(&dir);
        put(&root, "top.txt", "x");
        put(&root, "a/b/deep.txt", "x");
        let files = sample_files(&RealFs, &root);
        assert_eq!(files.len(), 2);
        assert!(files[0].ends_with("/top.txt"), "{files:?}");
    }

    #[test]
    fn access_denied_and_busy_are_held_a_missing_file_is_not() {
        assert!(is_held(&io::Error::from(io::ErrorKind::PermissionDenied)));
        assert!(is_held(&io::Error::from(io::ErrorKind::ResourceBusy)));
        assert!(!is_held(&io::Error::from(io::ErrorKind::NotFound)));
    }
}
