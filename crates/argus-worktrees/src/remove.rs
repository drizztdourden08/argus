//! Removing a worktree through Argus: the guards, in order, and then `git worktree remove`.
//!
//! GUARDS: the caller (the page, or the project's lead or the rule's thread); the id is in the live listing; the folder
//! is under a current rule; not a main checkout; not locked; the HEAD and dirty count the caller saw still hold; no live
//! process uses the folder; a fresh read-only look agrees with the row; and the work: removing the folder loses nothing
//! when it is clean and a branch or ref keeps HEAD. Anything else is refused: the owning lead commits what is worth
//! keeping, discards the rest inside the worktree and asks again. Only `discard` with the owner's say removes it as it
//! stands, and a backup is written first. Merged or not, the commits stay in the branch.

use crate::backup::{Backup, Need};
use crate::board::{BoardFacts, Caller};
use crate::branch::BranchOutcome;
use crate::error::{ErrorCode, Result, WtError};
use crate::exec::GitOutput;
use crate::fact::Fact;
use crate::finish::{Finish, LetGo};
use crate::guards::is_brock_lock;
use crate::inspect::{ScanCtx, inspect_dir};
use crate::leftover_remove::{Within, git_within_except, unreadable_refusal};
use crate::modules::NodeModules;
use crate::paths::{clean_path, is_worktree_id, join_path, parent_of, worktree_id};
use crate::progress::{WAITING, target_key};
use crate::request::{Discarded, RemoveInput, RemoveOutcome};
use crate::rule::Rule;
use crate::service::Service;
use crate::tree::drop_links;

/// What a removal notes for the activity line.
#[derive(Default)]
pub(crate) struct Trace {
    pub leftover: bool,
}

pub(crate) fn plural(n: u32, one: &str) -> String {
    format!("{n} {one}{}", if n == 1 { "" } else { "s" })
}

pub(crate) fn short7(text: Option<&str>) -> String {
    text.unwrap_or_default().chars().take(7).collect()
}

pub(crate) fn forbidden(message: String) -> WtError {
    WtError::new(ErrorCode::Forbidden, message)
}

/// The folder of a nested `.git`, relative to the worktree (`vendor/lib`).
fn relative_folder(root: &str, git: &str) -> String {
    let folder = parent_of(git);
    folder
        .strip_prefix(&format!("{root}/"))
        .map_or(folder.clone(), ToString::to_string)
}

/// What losing the folder would cost.
pub(crate) struct Safety {
    unmerged: Option<u32>,
    pub(crate) kept: bool,
    safe: bool,
}

impl Safety {
    /// Clean AND HEAD kept by a branch or another ref: removing the FOLDER loses nothing, merged or not.
    pub(crate) fn of(now: &Fact) -> Self {
        let unmerged = if now.merged { Some(0) } else { now.unmerged };
        let kept = now.referenced == Some(true)
            || (now.branch.is_some() && !now.detached)
            || now.merged
            || unmerged == Some(0);
        Self {
            unmerged,
            kept,
            safe: now.dirty == 0 && kept && now.nested_git.is_none(),
        }
    }

    fn loss(&self, now: &Fact, force_branch: bool) -> String {
        let lost_commits = if self.kept {
            None
        } else {
            Some(match self.unmerged {
                Some(n) if n > 0 => format!(
                    "{} on a detached HEAD that no branch keeps",
                    plural(n, "commit")
                ),
                _ => "commits on a detached HEAD that no branch keeps".to_string(),
            })
        };
        let branch = force_branch.then(|| {
            format!(
                "the commits of branch {} that are not merged",
                now.branch.as_deref().unwrap_or_default()
            )
        });
        let dirty = (now.dirty > 0).then(|| plural(now.dirty, "uncommitted change"));
        let nested = now.nested_git.as_ref().map(|n| {
            if now.nested_too_big {
                "a folder too big to check for nested git repositories".to_string()
            } else {
                format!(
                    "a nested git repository ({})",
                    relative_folder(&now.path, n)
                )
            }
        });
        [dirty, nested, lost_commits, branch]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" and ")
    }
}

impl Service {
    pub(crate) fn caller_rules<'a>(board: &'a BoardFacts, rule: Option<&'a Rule>) -> Vec<&'a Rule> {
        rule.filter(|r| r.problem().is_none() && board.rules.iter().any(|b| b.id == r.id))
            .into_iter()
            .collect()
    }

    pub(crate) fn who_and_why(input: &RemoveInput) -> (String, String) {
        let why = input
            .reason
            .as_deref()
            .map(|r| r.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|r| !r.is_empty());
        (
            input.caller.for_text(),
            why.map(|r| format!(": {}", r.chars().take(200).collect::<String>()))
                .unwrap_or_default(),
        )
    }

    /// One removal at a time, never during a scan; every outcome is an activity line.
    pub(crate) fn guarded(
        &self,
        input: &RemoveInput,
        rescan: bool,
        what: impl FnOnce(&mut Trace) -> Result<RemoveOutcome>,
    ) -> Result<RemoveOutcome> {
        self.step(&input.id, WAITING);
        let (who, why) = Self::who_and_why(input);
        let key = target_key(&input.id, self.ci());
        let named = if input.id.trim().is_empty() {
            "every gone worktree"
        } else {
            input.id.trim()
        };
        let holder = format!("the removal of {named} {who}");
        let _one = self.one_at_a_time(&input.project, &holder, Some(&key));
        self.step(&input.id, "checking");
        let mut trace = Trace::default();
        let thread = input.caller.thread();
        let result = what(&mut trace);
        self.step_end(&input.id);
        match &result {
            Ok(out) => self.note(
                &input.project,
                "info",
                &format!("{} {who}{why}", out.text),
                thread,
            ),
            Err(err) => {
                let level = if matches!(err.code, ErrorCode::GitFailed | ErrorCode::BackupFailed) {
                    "error"
                } else {
                    "warn"
                };
                let what_word = if trace.leftover {
                    "delete leftover folder"
                } else if input.prune {
                    "prune worktree"
                } else {
                    "remove worktree"
                };
                let id = if input.id.trim().is_empty() {
                    "all gone"
                } else {
                    input.id.as_str()
                };
                self.note(
                    &input.project,
                    level,
                    &format!("refused to {what_word} {id} {who}: {}{why}", err.message),
                    thread,
                );
            }
        }
        // A folder in use changed nothing: no rescan, which in a big project holds up the next removal for a whole scan.
        let in_use = matches!(&result, Err(err) if err.code == ErrorCode::InUse);
        if rescan && !in_use {
            self.request_rescan(&input.project);
        }
        result
    }

    /// Remove a worktree (or start deleting a leftover folder). Refusals are errors; nothing outside the rules is touched.
    pub fn remove(&self, input: &RemoveInput) -> Result<RemoveOutcome> {
        self.guarded(input, true, |trace| self.run(input, trace))
    }

    /// The same removal for the request worker, with no rescan per target: the worker asks for one when the request
    /// ends (I-134: in a big project each rescan held the next target's turn for a whole full scan).
    pub(crate) fn remove_in_request(&self, input: &RemoveInput) -> Result<RemoveOutcome> {
        self.guarded(input, false, |trace| self.run(input, trace))
    }

    /// Prune one gone worktree (by id or path), or with no id every gone one of the project.
    pub fn prune(&self, input: &RemoveInput) -> Result<RemoveOutcome> {
        if !input.id.trim().is_empty() {
            let mut one = input.clone();
            one.prune = true;
            return self.guarded(&one, true, |trace| self.run(&one, trace));
        }
        self.guarded(
            &RemoveInput {
                prune: true,
                ..input.clone()
            },
            true,
            |_| self.prune_all(input),
        )
    }

    pub(crate) fn run(&self, input: &RemoveInput, trace: &mut Trace) -> Result<RemoveOutcome> {
        let board = self.board(&input.project)?;
        let id = input.id.trim();
        if id.is_empty() {
            return Err(WtError::new(
                ErrorCode::BadRequest,
                "name the worktree: {id} (its id from the listing, or its path)",
            ));
        }
        let wid = if is_worktree_id(id) {
            id.to_string()
        } else {
            worktree_id(&clean_path(id), self.ci())
        };
        let Some(fact) = self.fact_of(&input.project, &wid, &board) else {
            return Err(self.not_listed(input, &board, &wid));
        };
        let not_git = fact.not_git() && !input.prune;
        let fact = self.as_leftover(fact, not_git);
        let fact = self.with_live_lock(&input.project, fact, not_git);
        trace.leftover = fact.leftover.is_some();
        let rule = board
            .rules
            .iter()
            .find(|r| r.id == fact.rule && r.problem().is_none());
        crate::guards::check_caller(&board, &input.caller, &Self::caller_rules(&board, rule))?;
        let rule = self.rule_of(&board, &fact)?;
        self.assert_not_main(&fact)?;
        let lifted = if input.prune {
            None
        } else {
            self.lift_stale_lock(&fact, not_git)?
        };
        if lifted.is_none() {
            Self::assert_not_locked(
                &fact,
                fact.leftover.is_none() && fact.exists && !input.prune,
            )?;
        }
        if input.job.is_some() && fact.leftover.is_none() {
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} is not a leftover folder any more (a scan took it for a worktree): nothing was deleted",
                    fact.path
                ),
            ));
        }
        if fact.leftover.is_some() && !input.prune {
            let started = self.start_leftover(input, (&fact, not_git), &rule, &board);
            if let (Err(_), Some(reason)) = (&started, &lifted) {
                self.relock(&fact, reason);
            }
            return started;
        }
        if !fact.exists || input.prune {
            return self.prune_gone(input, &fact);
        }
        self.remove_worktree(input, &board, &fact, &rule)
    }

    pub(crate) fn not_listed(&self, input: &RemoveInput, board: &BoardFacts, wid: &str) -> WtError {
        let rules: Vec<&Rule> = board.valid_rules();
        if let Err(err) = crate::guards::check_caller(board, &input.caller, &rules) {
            return err;
        }
        if let Some(job) = self.active_job_for(&input.project, wid) {
            return WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} is already being deleted (job {}, {})",
                    job.path,
                    job.id,
                    job.state.as_str()
                ),
            );
        }
        let no_rule = if rules.is_empty() {
            format!(
                "; {} has no valid worktree rule, so Argus lists nothing there: add one first (coord worktree rule add <folder>, with --kind repo for a repository)",
                input.project
            )
        } else {
            String::new()
        };
        WtError::new(
            ErrorCode::NotFound,
            format!(
                "{} is not in the live listing of {} (coord worktree list): Argus only removes what it lists{no_rule}",
                input.id.trim(),
                input.project
            ),
        )
    }

    pub(crate) fn git_repo(fact: &Fact) -> Result<&str> {
        fact.repo.as_deref().ok_or_else(|| {
            WtError::new(
                ErrorCode::GitFailed,
                format!("no repository is known for {}", fact.path),
            )
        })
    }

    fn check_seen(input: &RemoveInput, fact: &Fact) -> Result<()> {
        if let Some(head) = &input.head {
            if short7(Some(head)) != short7(fact.head.as_deref()) {
                return Err(WtError::new(
                    ErrorCode::Conflict,
                    format!(
                        "{} changed since you looked: its HEAD is {}, not {}. Look again (coord worktree list) before removing it",
                        fact.path,
                        short7(fact.head.as_deref()),
                        short7(Some(head))
                    ),
                ));
            }
        }
        if let Some(dirty) = input.dirty.filter(|d| *d != fact.dirty) {
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} changed since you looked: {}, not {dirty}. Look again before removing it",
                    fact.path,
                    plural(fact.dirty, "uncommitted change")
                ),
            ));
        }
        Ok(())
    }

    /// The folder as it is now (read-only git): not a link, readable by git, a nested repository named.
    fn inspect_now(&self, input: &RemoveInput, fact: &Fact, rule: &Rule) -> Result<Fact> {
        let mut ctx = ScanCtx {
            git: &self.deps.git,
            fs: self.deps.fs.as_ref(),
            lists: Vec::new(),
            case_insensitive: self.ci(),
        };
        if self.deps.fs.lstat(&fact.path).is_ok_and(|s| s.is_link) {
            return Err(forbidden(format!(
                "{} is a link or junction, not a worktree folder: Argus never removes one",
                fact.path
            )));
        }
        let mut now = inspect_dir(&fact.path, rule, &mut ctx);
        if let Some(why) = &now.error {
            return Err(WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "git could not read the state of {} ({why}), so Argus cannot tell whether work would be lost: nothing was removed",
                    fact.path
                ),
            ));
        }
        let except = self.submodule_folders(&fact.path);
        match git_within_except(self.deps.fs.as_ref(), &fact.path, &except) {
            Within::Nothing => {}
            Within::Git(git) => now.nested_git = Some(git),
            Within::TooBig => {
                now.nested_git = Some(join_path(&fact.path, ".git"));
                now.nested_too_big = true;
            }
            Within::Unreadable(why) => {
                return Err(WtError::new(
                    ErrorCode::Conflict,
                    unreadable_refusal(&fact.path, &why, "removed"),
                ));
            }
        }
        if !now.is_git || !self.deps.fs.exists(&fact.path) {
            self.request_rescan(&input.project);
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} is no longer a git worktree since the listing: nothing was removed, the listing is being refreshed",
                    fact.path
                ),
            ));
        }
        Ok(now)
    }

    /// The folder as it is now, against the row.
    fn look_again(&self, input: &RemoveInput, fact: &Fact, rule: &Rule) -> Result<Fact> {
        let mut now = self.inspect_now(input, fact, rule)?;
        if now.repo.is_none() {
            now.repo.clone_from(&fact.repo);
        }
        self.assert_not_main(&now)?;
        Self::assert_not_locked(&now, true)?;
        if !now.registered {
            return Err(forbidden(format!(
                "{} is not a linked worktree its repository lists: Argus never removes it",
                fact.path
            )));
        }
        let same = short7(now.head.as_deref()) == short7(fact.head.as_deref())
            && now.dirty == fact.dirty
            && now.branch == fact.branch
            && now.merged == fact.merged
            && now.referenced == fact.referenced;
        if !same {
            self.request_rescan(&input.project);
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} changed since the listing (HEAD {} -> {}, {} -> {} uncommitted): nothing was removed, the listing is being refreshed",
                    fact.path,
                    short7(fact.head.as_deref()),
                    short7(now.head.as_deref()),
                    fact.dirty,
                    now.dirty
                ),
            ));
        }
        Ok(now)
    }

    /// Why the owner's say for discarding is missing, or none: the page is the owner; a thread needs his entry; the
    /// hygiene has his rule of 2026-11-06 once the lead or the Overlord approved, and no say at all before that.
    fn owner_say_missing(
        &self,
        input: &RemoveInput,
        board: &BoardFacts,
        now: &Fact,
    ) -> Option<String> {
        match &input.caller {
            Caller::Page
            | Caller::Hygiene {
                approved_by: Some(_),
            } => None,
            Caller::Hygiene { approved_by: None } => {
                Some("nothing may be lost before the lead approves".to_string())
            }
            Caller::Thread(_) => {
                let entry = input.owner_entry.as_deref().unwrap_or_default();
                crate::approval::approval_of(board, entry, now, self.now())
            }
        }
    }

    /// Is discarding `loss` allowed: asked for, the owner's say given, and (when the hygiene removes it) a backup that
    /// holds all of it.
    fn may_discard(
        &self,
        input: &RemoveInput,
        board: &BoardFacts,
        now: &Fact,
        (loss, partial): (&str, Option<String>),
    ) -> Result<()> {
        if !input.discard {
            return Err(forbidden(format!(
                "{} has {loss}: removing it discards them. The owning lead decides: commit what is worth keeping, discard the rest inside the worktree, then request the removal again (the owner is not asked). Only the owner's own say discards it as it stands: Delete now on the page, or send discard with his entry (--discard --owner-entry <id>)",
                now.path
            )));
        }
        if let Some(why) = self.owner_say_missing(input, board, now) {
            return Err(forbidden(format!(
                "discarding {loss} in {} needs the owner: {why}",
                now.path
            )));
        }
        match partial {
            Some(why) => Err(forbidden(format!(
                "the backup of {} would be partial ({why}): nothing was removed. Commit or move that work, or discard it with the owner's say",
                now.path
            ))),
            None => Ok(()),
        }
    }

    /// The owner's say and the backup, for anything that is not safe. None when nothing is lost.
    fn authorize(
        &self,
        input: &RemoveInput,
        board: &BoardFacts,
        now: &Fact,
        safety: &Safety,
        force_branch: bool,
    ) -> Result<Option<Backup>> {
        if safety.safe && !force_branch {
            return Ok(None);
        }
        let warning = if now.dirty > 0 || now.nested_git.is_some() {
            self.untracked_warning(now)
        } else {
            None
        };
        let partial = matches!(input.caller, Caller::Hygiene { .. })
            .then(|| Self::partial_backup_why(now, warning.as_deref()))
            .flatten();
        let loss = [Some(safety.loss(now, force_branch)), warning]
            .into_iter()
            .flatten()
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join(" and ");
        self.may_discard(input, board, now, (&loss, partial))?;
        let need = Need {
            bundle: !safety.kept || force_branch,
            patch: now.dirty > 0 || now.nested_git.is_some(),
        };
        self.write_backup(&input.project, now, need).map(Some)
    }

    fn remove_worktree(
        &self,
        input: &RemoveInput,
        board: &BoardFacts,
        fact: &Fact,
        rule: &Rule,
    ) -> Result<RemoveOutcome> {
        Self::check_seen(input, fact)?;
        if let Some(user) = self.busy_by(&fact.path, false, "removed")? {
            return Err(Self::in_use(&fact.path, &user, "removed"));
        }
        self.probe_release(&fact.path)?;
        let now = self.look_again(input, fact, rule)?;
        let safety = Safety::of(&now);
        let owner_says = input.discard && !matches!(input.caller, Caller::Hygiene { .. });
        let plan = self.plan_branch(input.branch, owner_says, &now);
        let backup = self.authorize(input, board, &now, &safety, plan.force)?;
        let submodules = self.check_submodules(&now, backup.as_ref())?;
        self.step(&fact.id, "node_modules clear");
        let modules = self.clear_ignored_node_modules(&now);
        self.step(&fact.id, "links");
        self.drop_links_before_git(&now, backup.as_ref(), modules)?;
        self.lift_brock_lock(&now, backup.as_ref())?;
        self.step(&fact.id, "git worktree remove");
        let run = self.git_remove(&now, submodules)?;
        let discarded = (!safety.safe || plan.force).then_some(Discarded {
            dirty: now.dirty,
            unmerged: safety.unmerged,
        });
        let finish = Finish {
            input,
            fact,
            now: &now,
            run: &run,
            plan: &plan,
            modules,
        };
        if !run.ok() {
            return self.git_could_not(&finish, backup, discarded);
        }
        Ok(self.removed(&finish, backup, discarded))
    }

    /// Brock git-locks its clones. When every link is gone and the lock reason (read just now from `git worktree list`)
    /// is Brock's, the lock is lifted so `git worktree remove` can run. Any other lock was refused earlier.
    fn lift_brock_lock(&self, now: &Fact, backup: Option<&Backup>) -> Result<()> {
        if !now.locked.as_deref().is_some_and(is_brock_lock) {
            return Ok(());
        }
        let run = self
            .deps
            .git
            .write(Self::git_repo(now)?, &["worktree", "unlock", &now.path]);
        if run.ok() {
            return Ok(());
        }
        Err(WtError::new(
            ErrorCode::GitFailed,
            format!(
                "git worktree unlock failed for {}: {}; nothing was removed",
                now.path,
                run.failure_text()
            ),
        )
        .with_backup(backup.cloned()))
    }

    /// `git worktree remove`. `--force` for uncommitted changes the caller discards, and for initialized submodules
    /// (git refuses a worktree that holds one without it; they passed `check_submodules` already).
    fn git_remove(&self, now: &Fact, submodules: bool) -> Result<GitOutput> {
        let repo = Self::git_repo(now)?;
        let git = &self.deps.git;
        Ok(if now.dirty > 0 || submodules {
            git.write(repo, &["worktree", "remove", "--force", &now.path])
        } else {
            git.write(repo, &["worktree", "remove", &now.path])
        })
    }

    /// git failed: if it has let go of the worktree Argus finishes the folder itself, else the failure stands.
    fn git_could_not(
        &self,
        step: &Finish<'_>,
        backup: Option<Backup>,
        discarded: Option<Discarded>,
    ) -> Result<RemoveOutcome> {
        let why = match self.git_let_go(&step.input.project, step.now, step.run) {
            LetGo::Yes => return self.finish_by_argus(step, backup, discarded),
            LetGo::Unvetted(text) => format!("; Argus did not prune git's records: {text}"),
            LetGo::No => String::new(),
        };
        let cut = cut_text(step.run, step.modules);
        let here = backup
            .as_ref()
            .map(|b| format!("; the backup is in {}", b.dir))
            .unwrap_or_default();
        Err(WtError::new(
            ErrorCode::GitFailed,
            format!(
                "git worktree remove failed: {}{cut}{why}{here}",
                step.run.failure_text()
            ),
        )
        .with_backup(backup))
    }

    /// git removed the worktree: remember a folder it left behind, take the branch as decided, forget the row.
    fn removed(
        &self,
        step: &Finish<'_>,
        backup: Option<Backup>,
        discarded: Option<Discarded>,
    ) -> RemoveOutcome {
        let Finish {
            input,
            fact,
            now,
            plan,
            modules,
            ..
        } = *step;
        let left = self.deps.fs.exists(&now.path);
        if left {
            self.remember_made(&input.project, &now.path);
        }
        let branch = if input.branch {
            self.delete_branch(now, plan)
        } else {
            BranchOutcome::default()
        };
        self.forget(&input.project, &fact.id);
        let mut out = RemoveOutcome {
            removed: true,
            path: Some(now.path.clone()),
            branch: now.branch.clone(),
            backup,
            discarded,
            folder_left: left,
            node_modules_cleared: modules.cleared(),
            ..RemoveOutcome::default()
        };
        apply_branch(&mut out, &branch);
        out.text = out.sentence();
        out
    }

    /// Windows: git for Windows follows a junction when it deletes a folder, and deletes what the junction points to.
    /// So every link in the worktree is removed AS a link first; the removal is authorized by now. A link that cannot be
    /// removed stops the removal: git must not meet it.
    fn drop_links_before_git(
        &self,
        now: &Fact,
        backup: Option<&Backup>,
        modules: NodeModules,
    ) -> Result<()> {
        if !self.opts.drop_links_first {
            return Ok(());
        }
        drop_links(self.deps.fs.as_ref(), &now.path).map_err(|err| {
            let began = if modules.begun() {
                " (except the ignored node_modules Argus began to clear first)"
            } else {
                ""
            };
            let here = backup.map(|b| format!("; the backup is in {}", b.dir)).unwrap_or_default();
            WtError::new(ErrorCode::GitFailed, format!("could not remove the junctions or symlinks inside {} first ({}): git for Windows would follow them and delete what they point to, so nothing was removed{began}{here}", now.path, crate::error::io_code(&err))).with_backup(backup.cloned())
        })
    }
}

/// What a refusal adds when git was cut: what Argus itself deleted before git ran.
fn cut_text(run: &GitOutput, modules: NodeModules) -> String {
    if !run.was_cut() {
        return String::new();
    }
    match modules {
        NodeModules::Untouched => "; nothing was deleted by Argus".to_string(),
        NodeModules::Cleared => "; Argus deleted nothing but the ignored node_modules".to_string(),
        NodeModules::Partial => {
            "; Argus deleted nothing but part of the ignored node_modules".to_string()
        }
    }
}

pub(crate) fn apply_branch(out: &mut RemoveOutcome, branch: &BranchOutcome) {
    out.deleted_branch = branch.deleted;
    out.branch_note.clone_from(&branch.note);
    out.branch_why.clone_from(&branch.why);
}
