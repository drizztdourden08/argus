//! The submodules of a worktree that is being removed.
//!
//! `git worktree remove` refuses a worktree with initialized submodules ("working trees containing submodules cannot be
//! moved or removed") and only `--force` gets past it. So Argus looks at the submodules itself first: one moved off its
//! recorded commit, in conflict or with uncommitted changes keeps the worktree, because the removal would delete work that
//! lives only there. An uninitialized submodule is an empty folder and holds nothing.

use crate::backup::Backup;
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::paths::{join_path, path_key};
use crate::remove::plural;
use crate::service::Service;

/// What `git submodule status` says about one submodule, from the first character of its line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    /// `-`: not initialized, an empty folder.
    Uninitialized,
    /// A space: checked out at the recorded commit.
    Clean,
    /// `+`: checked out at another commit than the recorded one.
    Moved,
    /// `U`: merge conflicts.
    Conflict,
}

struct Sub {
    /// The path below the worktree (a nested submodule is listed with its parents in front).
    path: String,
    mark: Mark,
}

/// A line of `git submodule status --recursive`: the mark, the commit, the path and, when checked out, a describe in
/// parentheses.
fn parse_line(line: &str) -> Option<Sub> {
    let mark = match line.chars().next()? {
        '-' => Mark::Uninitialized,
        '+' => Mark::Moved,
        'U' => Mark::Conflict,
        ' ' => Mark::Clean,
        _ => return None,
    };
    let (_, named) = line[1..].split_once(' ')?;
    let path = match named.rfind(" (") {
        Some(at) if named.ends_with(')') => &named[..at],
        _ => named,
    };
    Some(Sub {
        path: path.trim().to_string(),
        mark,
    })
}

/// Folders whose own `.git` is a submodule's, not a nested repository: the walk for a nested `.git` passes them by.
pub(crate) struct Except {
    keys: Vec<String>,
    ci: bool,
}

impl Except {
    pub(crate) fn none() -> Self {
        Self {
            keys: Vec::new(),
            ci: false,
        }
    }

    fn new(folders: &[String], ci: bool) -> Self {
        Self {
            keys: folders.iter().map(|f| path_key(f, ci)).collect(),
            ci,
        }
    }

    pub(crate) fn holds(&self, folder: &str) -> bool {
        let key = path_key(folder, self.ci);
        self.keys.contains(&key)
    }
}

fn backup_here(backup: Option<&Backup>) -> String {
    backup
        .map(|b| format!("; the backup is in {}", b.dir))
        .unwrap_or_default()
}

impl Service {
    /// The submodules git lists for the worktree (read-only). No `.gitmodules` means none, and no git call.
    fn listed_submodules(&self, worktree: &str) -> std::result::Result<Vec<Sub>, String> {
        if !self.deps.fs.exists(&join_path(worktree, ".gitmodules")) {
            return Ok(Vec::new());
        }
        let run = self
            .deps
            .git
            .read(worktree, &["submodule", "status", "--recursive"]);
        if !run.ok() {
            return Err(run.failure_text());
        }
        run.stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| parse_line(line).ok_or_else(|| format!("unexpected line: {line}")))
            .collect()
    }

    /// The folders of the initialized submodules, for the walk for a nested repository. A listing git cannot give
    /// leaves them out: their `.git` then counts as a nested repository, which is the safe side.
    pub(crate) fn submodule_folders(&self, worktree: &str) -> Except {
        let folders: Vec<String> = self
            .listed_submodules(worktree)
            .unwrap_or_default()
            .iter()
            .filter(|s| s.mark != Mark::Uninitialized)
            .map(|s| join_path(worktree, &s.path))
            .collect();
        Except::new(&folders, self.ci())
    }

    /// What `git status` inside the submodule says is not committed there, or why it could not be read.
    fn submodule_changes(&self, folder: &str) -> Option<String> {
        let run = self.deps.git.read(folder, &["status", "--porcelain"]);
        if run.ok() {
            let changes = u32::try_from(run.stdout.lines().count()).unwrap_or(u32::MAX);
            return (changes > 0).then(|| format!("has {}", plural(changes, "uncommitted change")));
        }
        Some(format!("could not be read ({})", run.failure_text()))
    }

    /// Why this initialized submodule keeps the worktree, or None when it holds nothing that would be lost.
    fn submodule_problem(&self, worktree: &str, sub: &Sub) -> Option<String> {
        let why = match sub.mark {
            Mark::Moved => Some("moved off its recorded commit".to_string()),
            Mark::Conflict => Some("has a merge conflict".to_string()),
            Mark::Clean | Mark::Uninitialized => {
                self.submodule_changes(&join_path(worktree, &sub.path))
            }
        };
        why.map(|why| format!("{} {why}", sub.path))
    }

    /// Before the removal: every initialized submodule must be clean and on its recorded commit. Ok(true) when the
    /// worktree has initialized submodules that passed (git then needs `--force`), Ok(false) when it has none.
    pub(crate) fn check_submodules(&self, now: &Fact, backup: Option<&Backup>) -> Result<bool> {
        let subs = self.listed_submodules(&now.path).map_err(|why| {
            WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "git could not list the submodules of {} ({why}), so Argus cannot tell whether work would be lost: nothing was removed{}",
                    now.path,
                    backup_here(backup)
                ),
            )
            .with_backup(backup.cloned())
        })?;
        let held: Vec<&Sub> = subs
            .iter()
            .filter(|s| s.mark != Mark::Uninitialized)
            .collect();
        let problems: Vec<String> = held
            .iter()
            .filter_map(|s| self.submodule_problem(&now.path, s))
            .collect();
        if problems.is_empty() {
            return Ok(!held.is_empty());
        }
        Err(WtError::new(
            ErrorCode::Forbidden,
            format!(
                "{} holds submodules whose work would be lost: {}. Commit and push the work inside each submodule, or put it back on its recorded commit, then request the removal again: nothing was removed{}",
                now.path,
                problems.join("; "),
                backup_here(backup)
            ),
        )
        .with_backup(backup.cloned()))
    }
}
