//! A registered worktree whose folder lost its `.git` file and whose Claude Code lock is stale (I-163). Git still lists
//! it, so it is not a leftover to git, but nothing holds it: the lock names a pid that is not running (or names none and
//! is older than a day) and the folder holds no link. The lock is lifted, the folder goes the way a leftover goes, and
//! git forgets the record. A live pid, a link or a `.git` folder keeps the refusal.

use crate::clock::DAY_MS;
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::fs::Fs;
use crate::guards::{claude_lock_pid, is_claude_lock};
use crate::parse::parse_worktree_list;
use crate::paths::{join_path, path_key};
use crate::service::Service;

/// How many folders the walk for links visits before it gives up.
const LINK_WALK_CAP: usize = 3000;

/// What the walk for links found: the first link, or why it could not tell.
enum Links {
    None,
    Link(String),
    Unknown(String),
}

fn walk_links(fs: &dyn Fs, root: &str) -> Links {
    let mut todo = vec![root.to_string()];
    let mut seen = 0;
    while let Some(dir) = todo.pop() {
        seen += 1;
        if seen > LINK_WALK_CAP {
            return Links::Unknown(format!("{root} has more than {LINK_WALK_CAP} folders"));
        }
        let entries = match fs.read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => return Links::Unknown(format!("{dir}: {err}")),
        };
        for entry in entries {
            let path = join_path(&dir, &entry.name);
            if entry.is_link {
                return Links::Link(path);
            }
            if entry.is_dir {
                todo.push(path);
            }
        }
    }
    Links::None
}

impl Service {
    /// What git lists right now for a folder with no `.git` file: registered, its lock and its repository. The scan
    /// reads a repository's listing only through a worktree of it, so it may have missed the lock; the removal asks
    /// again. A repository that does not answer adds nothing here (the leftover guards refuse on it later).
    pub(crate) fn with_live_lock(&self, project: &str, mut fact: Fact) -> Fact {
        let (ci, mine) = (self.ci(), path_key(&fact.path, self.ci()));
        for repo in self.candidate_repos(project) {
            if !self.deps.fs.exists(&repo) {
                continue;
            }
            let run = self
                .deps
                .git
                .read(&repo, &["worktree", "list", "--porcelain"]);
            if !run.ok() {
                continue;
            }
            let items = parse_worktree_list(&run.stdout);
            if let Some(item) = items.iter().skip(1).find(|i| path_key(&i.path, ci) == mine) {
                fact.registered = true;
                fact.locked.clone_from(&item.locked);
                if fact.repo.is_none() {
                    fact.repo = items.first().map(|i| i.path.clone());
                }
                return fact;
            }
        }
        fact
    }

    /// For a registered folder with no `.git` file: lift a stale Claude Code lock so the folder can be deleted like a
    /// leftover. `Ok(Some(reason))` is the lock that was lifted (to put back if a later guard refuses); `Ok(None)` means
    /// this is not the case (the usual lock guard judges it). A live pid, or a link in the folder, is a refusal.
    pub(crate) fn lift_stale_lock(&self, fact: &Fact, not_git: bool) -> Result<Option<String>> {
        let Some(reason) = fact.locked.clone().filter(|r| is_claude_lock(r)) else {
            return Ok(None);
        };
        let held = fact.leftover.is_some() && not_git && fact.registered;
        if !held || self.deps.fs.exists(&join_path(&fact.path, ".git")) {
            return Ok(None);
        }
        if !self.lock_is_stale(fact, &reason)? {
            return Ok(None);
        }
        self.assert_no_links(fact)?;
        let run = self
            .deps
            .git
            .write(Self::git_repo(fact)?, &["worktree", "unlock", &fact.path]);
        if !run.ok() {
            return Err(WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "git worktree unlock failed for {}: {}; nothing was deleted",
                    fact.path,
                    run.failure_text()
                ),
            ));
        }
        Ok(Some(reason))
    }

    /// Is the lock dead? A pid that is running refuses with its number; no pid counts after a day of no change.
    fn lock_is_stale(&self, fact: &Fact, reason: &str) -> Result<bool> {
        let Some(pid) = claude_lock_pid(reason) else {
            let newest = fact.leftover.as_ref().and_then(|l| l.newest_ms);
            return Ok(newest.is_some_and(|n| self.now() - n >= DAY_MS));
        };
        if self.deps.procs.alive(pid) {
            return Err(WtError::new(
                ErrorCode::Forbidden,
                format!(
                    "{} is locked ({reason}) by pid {pid}, which is running: the agent still holds it, so Argus does not remove it",
                    fact.path
                ),
            ));
        }
        Ok(true)
    }

    fn assert_no_links(&self, fact: &Fact) -> Result<()> {
        match walk_links(self.deps.fs.as_ref(), &fact.path) {
            Links::None => Ok(()),
            Links::Link(link) => Err(WtError::new(
                ErrorCode::Forbidden,
                format!(
                    "{} holds a link or junction ({link}) and its lock is stale: Argus unlinks nothing here and left the lock. Remove the link by hand",
                    fact.path
                ),
            )),
            Links::Unknown(why) => Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "Argus cannot tell whether {} holds a link ({why}): the stale lock was left and nothing was deleted",
                    fact.path
                ),
            )),
        }
    }

    /// Put a lifted lock back when a later guard refused the deletion.
    pub(crate) fn relock(&self, fact: &Fact, reason: &str) {
        if let Ok(repo) = Self::git_repo(fact) {
            let _ = self
                .deps
                .git
                .write(repo, &["worktree", "lock", "--reason", reason, &fact.path]);
        }
    }
}
