//! The worktree as `git worktree remove` itself will judge it, read before Argus touches the folder (I-177).
//!
//! git refuses a worktree without `--force` when `git status --porcelain --ignore-submodules=none` prints anything. The
//! scan's own status is close to that but not the same, and Argus changes the folder before git looks: it clears the
//! ignored `node_modules` and drops every link. A link that holds tracked paths (a tracked symlink, or a junction git
//! for Windows reads as the folder it points to) leaves those paths deleted once it goes, so a worktree the scan called
//! clean is one git refuses, after `node_modules` is already gone. Both are counted here, while nothing is touched yet,
//! as uncommitted changes: the removal is then refused like any other dirty worktree, or forced with the owner's say.

use crate::error::{ErrorCode, Result, WtError, io_code};
use crate::fact::Fact;
use crate::fs::Fs;
use crate::paths::join_path;
use crate::service::Service;
use std::io;

/// The status `git worktree remove` runs, with untracked files always shown (`status.showUntrackedFiles=no` would
/// hide them from the count and not from the loss).
const GIT_CHECK: &[&str] = &[
    "status",
    "--porcelain",
    "--ignore-submodules=none",
    "--untracked-files=normal",
];

/// How many link paths one `ls-files` call names.
const LINKS_PER_CALL: usize = 100;

/// The links below `abs` (`rel` is its path in the worktree), outside every `node_modules` and `.git` folder: pnpm's
/// links hold no tracked path, and a root `node_modules` with a tracked file is never cleared. A link is never entered.
fn links_below(fs: &dyn Fs, abs: &str, rel: &str, out: &mut Vec<String>) -> io::Result<()> {
    for entry in fs.read_dir(abs)? {
        let skipped = entry.name == "node_modules" || entry.name == ".git";
        if skipped || !(entry.is_dir || entry.is_link) {
            continue;
        }
        let path = join_path(abs, &entry.name);
        let child = if rel.is_empty() {
            entry.name.clone()
        } else {
            format!("{rel}/{}", entry.name)
        };
        let Ok(stat) = fs.lstat(&path) else {
            continue;
        };
        if stat.is_link {
            out.push(child);
        } else if stat.is_dir {
            links_below(fs, &path, &child, out)?;
        }
    }
    Ok(())
}

fn count_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Argus cannot tell what git will see: nothing is touched.
fn cannot_tell(now: &Fact, why: &str) -> WtError {
    WtError::new(
        ErrorCode::GitFailed,
        format!(
            "git could not read the state of {} ({why}), so Argus cannot tell whether git would refuse it or work would be lost: nothing was removed",
            now.path
        ),
    )
}

impl Service {
    /// `now` with its dirty count raised to what git's own check will count once the links are gone.
    pub(crate) fn as_git_sees(&self, mut now: Fact) -> Result<Fact> {
        let changes = self.git_check_count(&now)? + self.tracked_behind_links(&now)?;
        now.dirty = now.dirty.max(changes);
        Ok(now)
    }

    /// The lines git's own check prints now: tracked changes and untracked, not ignored, files.
    fn git_check_count(&self, now: &Fact) -> Result<u32> {
        let run = self.deps.git.read(&now.path, GIT_CHECK);
        if !run.ok() {
            return Err(cannot_tell(now, &run.failure_text()));
        }
        let lines = run.stdout.lines().filter(|l| !l.trim().is_empty()).count();
        Ok(count_u32(lines))
    }

    /// The tracked paths that a link holds, which read as deleted once Argus drops the links. None when links stay.
    fn tracked_behind_links(&self, now: &Fact) -> Result<u32> {
        if !self.opts.drop_links_first {
            return Ok(0);
        }
        let mut links = Vec::new();
        links_below(self.deps.fs.as_ref(), &now.path, "", &mut links)
            .map_err(|err| cannot_tell(now, &io_code(&err)))?;
        let mut tracked = 0;
        for chunk in links.chunks(LINKS_PER_CALL) {
            let specs: Vec<String> = chunk.iter().map(|l| format!(":(literal){l}")).collect();
            let mut args = vec!["ls-files", "--cached", "-z", "--"];
            args.extend(specs.iter().map(String::as_str));
            let run = self.deps.git.write(&now.path, &args);
            if !run.ok() {
                return Err(cannot_tell(now, &run.failure_text()));
            }
            tracked += run.stdout.split('\0').filter(|p| !p.is_empty()).count();
        }
        Ok(count_u32(tracked))
    }
}
