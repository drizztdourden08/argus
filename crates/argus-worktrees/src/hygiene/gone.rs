//! The gone records of a repository: a worktree whose folder is deleted stays listed until a prune forgets it. The
//! step prunes the safe ones by itself, once per repository, so they stop showing as prunable.

use crate::board::BoardFacts;
use crate::error::Result;
use crate::fact::Fact;
use crate::paths::same_path;
use crate::prune::{Record, folder_names};
use crate::service::Service;

/// The reason of the lock that keeps a stray record out of the prune for a moment.
const GUARD: &str = "argus: held while the gone records of its repository are pruned";

fn is_allowed(record: &Record, allowed: &[&str], ci: bool) -> bool {
    record
        .path
        .as_deref()
        .is_some_and(|p| allowed.iter().any(|a| same_path(a, p, ci)))
}

impl Service {
    /// Prune the safe gone worktrees of the project: under a valid rule, not locked, no commit lost. Unlike the manual
    /// prune it never refuses a whole repository: a record outside every rule is held back, the others go. True when
    /// a record went.
    pub(crate) fn prune_gone_records(&self, project: &str, board: &BoardFacts) -> bool {
        let gone: Vec<Fact> = self
            .vetted_gone(project, board)
            .into_iter()
            .filter(|f| !self.reports_only(board, &f.rule) && !self.is_kept(f))
            .collect();
        let mut repos: Vec<&str> = gone.iter().filter_map(|f| f.repo.as_deref()).collect();
        repos.sort_unstable();
        repos.dedup();
        let mut pruned = false;
        for repo in repos {
            let facts: Vec<&Fact> = gone
                .iter()
                .filter(|f| f.repo.as_deref() == Some(repo))
                .collect();
            let allowed: Vec<&str> = facts.iter().map(|f| f.path.as_str()).collect();
            let ran = self.prune_allowed(repo, &allowed);
            if ran.is_ok() {
                self.st().prune_errors.remove(repo);
            }
            match ran {
                Ok(0) => {}
                Ok(count) => {
                    pruned = true;
                    self.pruned_note(project, repo, &facts, count);
                    for f in &facts {
                        self.forget(project, &f.id);
                    }
                }
                Err(err) => self.prune_failed(project, repo, &err.message),
            }
        }
        pruned
    }

    /// A failing prune is said once per repository and distinct error, not at every step.
    fn prune_failed(&self, project: &str, repo: &str, message: &str) {
        let before = self
            .st()
            .prune_errors
            .insert(repo.to_string(), message.to_string());
        if before.as_deref() != Some(message) {
            let text = format!("could not prune the gone worktrees of {repo}: {message}");
            self.note(project, "warn", &text, None);
        }
    }

    fn pruned_note(&self, project: &str, repo: &str, facts: &[&Fact], count: usize) {
        let names = folder_names(facts.iter().copied());
        let text = format!("pruned {count} gone worktree record(s) of {repo} ({names})");
        self.note(project, "info", &text, None);
    }

    /// One `git worktree prune` that forgets only the `allowed` records: every other record it would forget is locked
    /// for the length of the prune. Returns how many allowed records were forgotten; none when a stray record cannot
    /// be held back (its folder is unknown).
    fn prune_allowed(&self, repo: &str, allowed: &[&str]) -> Result<usize> {
        let ci = self.ci();
        let records = self.prune_records(repo)?;
        let (safe, strays): (Vec<&Record>, Vec<&Record>) =
            records.iter().partition(|r| is_allowed(r, allowed, ci));
        let held: Option<Vec<&str>> = strays.iter().map(|r| r.path.as_deref()).collect();
        let (false, Some(held)) = (safe.is_empty(), held) else {
            return Ok(0);
        };
        let locked: Vec<&str> = held
            .iter()
            .copied()
            .filter(|p| {
                self.deps
                    .git
                    .write(repo, &["worktree", "lock", "--reason", GUARD, p])
                    .ok()
            })
            .collect();
        let all_held = locked.len() == held.len();
        let ran = if all_held {
            self.run_prune(repo)
        } else {
            Ok(())
        };
        for path in &locked {
            self.deps.git.write(repo, &["worktree", "unlock", path]);
        }
        ran.map(|()| if all_held { safe.len() } else { 0 })
    }
}
