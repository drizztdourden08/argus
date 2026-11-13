//! A stale Claude Code lock: Claude Code locks an agent worktree with `claude agent <id> (pid <n>)`. Once that pid is
//! no longer running nothing holds the lock, so the step lifts it and the worktree is judged like any other. A live
//! pid, a lock that is not Claude's, or a reason with no pid stays as it is.

use crate::board::BoardFacts;
use crate::fact::Fact;
use crate::guards::claude_lock_pid;
use crate::service::Service;

impl Service {
    /// Unlock the worktrees of the project whose Claude lock names a pid that is gone. True when one was unlocked.
    pub(crate) fn unlock_stale_claude(&self, project: &str, board: &BoardFacts) -> bool {
        let stale: Vec<(Fact, u32)> = self
            .facts_of(project)
            .into_iter()
            .filter(|f| !f.main && self.rule_of(board, f).is_ok())
            .filter(|f| !self.reports_only(board, &f.rule) && !self.is_kept(f))
            .filter_map(|f| {
                let pid = claude_lock_pid(f.locked.as_deref()?)?;
                Some((f, pid))
            })
            .filter(|(_, pid)| !self.deps.procs.alive(*pid))
            .collect();
        let mut lifted = false;
        for (fact, pid) in &stale {
            lifted |= self.unlock_one(project, fact, *pid);
        }
        lifted
    }

    fn unlock_one(&self, project: &str, fact: &Fact, pid: u32) -> bool {
        let Ok(repo) = Self::git_repo(fact) else {
            return false;
        };
        let run = self
            .deps
            .git
            .write(repo, &["worktree", "unlock", &fact.path]);
        if !run.ok() {
            return false;
        }
        if let Some(scan) = self.st().scans.get_mut(project) {
            scan.facts
                .iter_mut()
                .filter(|f| f.id == fact.id)
                .for_each(|f| f.locked = None);
        }
        self.request_rescan(project);
        let text = format!(
            "unlocked {}: the Claude Code lock names pid {pid}, which is not running",
            fact.path
        );
        self.note(project, "info", &text, None);
        true
    }
}
