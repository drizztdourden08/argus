//! When git cannot delete the folder. On Windows `git worktree remove` often ends `failed to delete '<path>': Directory
//! not empty` (or `Permission denied`) on a folder with pnpm `node_modules` junctions or a handle held by a process,
//! after it has deleted the tracked files and its `.git`. Once every guard has passed, Argus finishes the removal itself:
//! the folder is renamed `<name>.argus-deleting-<ms>` and the junction-safe job deletes it in the background.

use crate::backup::Backup;
use crate::branch::{BranchOutcome, BranchPlan};
use crate::deleting::deleting_path;
use crate::error::{ErrorCode, Result, WtError, io_code};
use crate::exec::GitOutput;
use crate::fact::Fact;
use crate::jobs::JobKind;
use crate::modules::NodeModules;
use crate::parse::{ListedWorktree, parse_worktree_list};
use crate::paths::{clean_path, path_key};
use crate::remove::apply_branch;
use crate::request::{Discarded, RemoveInput, RemoveOutcome};
use crate::service::Service;

const FOLDER_ERRORS: [&str; 11] = [
    "directory not empty",
    "failed to delete",
    "permission denied",
    "access is denied",
    "used by another process",
    "device or resource busy",
    "text file busy",
    "operation not permitted",
    "invalid argument",
    "cannot remove",
    "resource temporarily unavailable",
];

/// Is this what git says when it could not delete the folder itself?
pub fn is_folder_error(text: &str) -> bool {
    let lower = text.to_lowercase();
    FOLDER_ERRORS.iter().any(|s| lower.contains(s))
}

/// What a removal knows when git could not delete the folder.
#[derive(Clone, Copy)]
pub(crate) struct Finish<'a> {
    pub input: &'a RemoveInput,
    pub fact: &'a Fact,
    pub now: &'a Fact,
    pub run: &'a GitOutput,
    pub plan: &'a BranchPlan,
    pub modules: NodeModules,
}

/// Has git let go of the worktree after its removal failed on the folder?
pub(crate) enum LetGo {
    Yes,
    No,
    /// git let go, but pruning its records would also forget some Argus did not vet: the sentence says which.
    Unvetted(String),
}

/// What git says about one worktree now.
enum Listing {
    /// git could not answer.
    Unknown,
    /// git no longer lists it.
    Absent,
    Present(ListedWorktree),
}

impl Service {
    fn look_listed(&self, now: &Fact) -> Listing {
        let Some(repo) = now.repo.as_deref() else {
            return Listing::Unknown;
        };
        let listing = self
            .deps
            .git
            .read(repo, &["worktree", "list", "--porcelain"]);
        if !listing.ok() {
            return Listing::Unknown;
        }
        let key = path_key(&now.path, self.ci());
        parse_worktree_list(&listing.stdout)
            .into_iter()
            .find(|i| path_key(&i.path, self.ci()) == key)
            .map_or(Listing::Absent, Listing::Present)
    }

    /// After `git worktree remove` failed on the folder or ran past its timeout and was killed: has git already let go
    /// of the worktree (it no longer lists it, or lists it only because its `.git` file is gone, which `git worktree
    /// prune` clears)? False when the error was neither, git cannot say, the worktree is still listed with its `.git`
    /// or is locked.
    pub(crate) fn git_let_go(&self, project: &str, now: &Fact, run: &GitOutput) -> LetGo {
        let said = format!(
            "{}
{}",
            run.stderr,
            run.error.as_deref().unwrap_or_default()
        );
        if !(run.was_cut() || is_folder_error(&said))
            || self.deps.fs.is_dir(&format!("{}/.git", now.path))
        {
            return LetGo::No;
        }
        let entry = match self.look_listed(now) {
            Listing::Unknown => return LetGo::No,
            Listing::Absent => return LetGo::Yes,
            Listing::Present(entry) => entry,
        };
        if entry.locked.is_some() || self.deps.fs.exists(&format!("{}/.git", now.path)) {
            return LetGo::No;
        }
        let (Some(repo), Ok(board)) = (now.repo.as_deref(), self.board(project)) else {
            return LetGo::No;
        };
        let mut vetted: Vec<String> = self
            .vetted_gone(project, &board)
            .into_iter()
            .map(|f| f.path)
            .collect();
        vetted.push(now.path.clone());
        let allowed: Vec<&str> = vetted.iter().map(String::as_str).collect();
        if let Err(err) = self.vet_prune(repo, &allowed) {
            return LetGo::Unvetted(err.message);
        }
        if self.run_prune(repo).is_ok() && matches!(self.look_listed(now), Listing::Absent) {
            LetGo::Yes
        } else {
            LetGo::No
        }
    }

    /// The name a folder is renamed to before it is deleted (free in its parent).
    pub(crate) fn deleting_name(&self, path: &str) -> String {
        let stamp = self.now();
        let mut target = String::new();
        for i in 0..20 {
            target = deleting_path(path, stamp + i);
            if !self.deps.fs.exists(&target) {
                break;
            }
        }
        target
    }

    /// The rest of a removal git started: rename the folder and let the job delete it; the branch goes first, as asked.
    pub(crate) fn finish_by_argus(
        &self,
        step: &Finish<'_>,
        backup: Option<Backup>,
        discarded: Option<Discarded>,
    ) -> Result<RemoveOutcome> {
        let Finish {
            input,
            fact,
            now,
            run,
            plan,
            modules,
        } = *step;
        self.remember_made(&input.project, &now.path);
        let branch = if input.branch {
            self.delete_branch(now, plan)
        } else {
            BranchOutcome::default()
        };
        let target = self.deleting_name(&now.path);
        let git_said = run.failure_text();
        if let Err(err) = self.deps.fs.rename(&now.path, &target) {
            let branch_text = branch_sentence(input.branch, now, &branch);
            let here = backup
                .as_ref()
                .map(|b| format!(" The backup is in {}.", b.dir))
                .unwrap_or_default();
            let message = format!(
                "git removed the worktree record of {} but could not delete its folder ({git_said}), and Argus could not rename it for deletion either ({}): something has it open or locked. It is a leftover folder now: close what holds it and run coord worktree remove on it again (a leftover this removal made is taken without the age limit).{branch_text}{here}",
                now.path,
                io_code(&err)
            );
            return Err(WtError::new(ErrorCode::InUse, message).with_backup(backup));
        }
        let by = input.caller.by_text();
        let job_id = self.new_job(&input.project, fact, &by, JobKind::Worktree);
        self.update_job(&job_id, |job| {
            job.folder = clean_path(&target);
            job.git_error = Some(git_said.clone());
        });
        self.forget(&input.project, &fact.id);
        self.enqueue_delete(&job_id);
        let mut out = RemoveOutcome {
            removed: true,
            finished_by_argus: true,
            job_id: Some(job_id),
            folder: Some(clean_path(&target)),
            path: Some(now.path.clone()),
            branch: now.branch.clone(),
            backup,
            discarded,
            git_error: Some(git_said),
            node_modules_cleared: modules.cleared(),
            ..RemoveOutcome::default()
        };
        apply_branch(&mut out, &branch);
        out.text = out.sentence();
        Ok(out)
    }
}

fn branch_sentence(asked: bool, now: &Fact, branch: &BranchOutcome) -> String {
    let name = now.branch.as_deref().unwrap_or_default();
    match (asked, branch.deleted, &branch.note) {
        (true, true, _) => format!(" Branch {name} was deleted."),
        (true, false, Some(note)) => format!(" Branch {name} kept ({note})."),
        (false, ..) | (true, false, None) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_folder_deletion_sentences_git_uses_are_recognised() {
        for said in [
            "error: failed to delete 'X:/a/b': Directory not empty",
            "error: failed to delete 'X:/a/b': Permission denied",
            "error: cannot remove X:/a: Access is denied",
            "The process cannot access the file because it is being used by another process.",
        ] {
            assert!(is_folder_error(said), "{said}");
        }
        for said in [
            "fatal: not a git repository",
            "fatal: something else went wrong",
            "fatal: '.' contains modified or untracked files, use --force to delete it",
        ] {
            assert!(!is_folder_error(said), "{said}");
        }
    }
}
