//! Deleting one leftover folder: a folder with no `.git` that no repository lists. Git has nothing to say about it, so
//! there is no backup; the guards stand in for git's. The folder is renamed first (atomic, frees the name, proves
//! nothing holds it), then deleted in the background by a job.

use crate::board::BoardFacts;
use crate::clock::{Clock, SystemClock, iso_from_ms};
use crate::error::{ErrorCode, Result, WtError, io_code};
use crate::fact::Fact;
use crate::fs::Fs;
use crate::jobs::JobKind;
use crate::measure::{Leftover, about_text, measure_folder};
use crate::parse::parse_worktree_list;
use crate::paths::{is_below, join_path, path_key};
use crate::request::{RemoveInput, RemoveOutcome};
use crate::rule::Rule;
use crate::scan::inside_worktree;
use crate::service::{Service, age_ms};
use crate::submodules::Except;

/// A thread may go below the server's age limit down to this (6 hours), and only with a reason. The project's lead
/// and the Overlord may go lower, with a reason too (P-258): the process, link and listing guards still run.
pub const THREAD_MIN_AGE_DAYS: f64 = 0.25;

/// How many folders the walk for a nested `.git` visits before it gives up.
const WALK_CAP: i32 = 3000;

/// What the walk for a nested `.git` found. A walk that hit its cap is `TooBig`, and a folder it could not read is
/// `Unreadable` (the folder and the reason): both unknown, never "nothing found".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Within {
    Nothing,
    Git(String),
    TooBig,
    Unreadable(String),
}

/// The refusal sentence for a walk that could not read a folder. `undone` says what did not happen ("deleted").
pub(crate) fn unreadable_refusal(path: &str, unreadable: &str, undone: &str) -> String {
    format!(
        "{path} has a folder that could not be read ({unreadable}), so Argus cannot tell whether it holds a nested git repository: nothing was {undone}"
    )
}

fn walk_for_git(fs: &dyn Fs, dir: &str, level: u32, budget: &mut i32, except: &Except) -> Within {
    if level > 3 {
        return Within::Nothing;
    }
    if *budget <= 0 {
        return Within::TooBig;
    }
    *budget -= 1;
    let entries = match fs.read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Within::Nothing,
        Err(err) => return Within::Unreadable(format!("{dir}: {}", io_code(&err))),
    };
    if level > 0 && entries.iter().any(|e| e.name == ".git") && !except.holds(dir) {
        return Within::Git(join_path(dir, ".git"));
    }
    for entry in entries
        .iter()
        .filter(|e| e.is_dir && !e.is_link && e.name != "node_modules" && e.name != ".git")
    {
        match walk_for_git(fs, &join_path(dir, &entry.name), level + 1, budget, except) {
            Within::Nothing => {}
            found => return found,
        }
    }
    Within::Nothing
}

/// A `.git` (file or folder) within three levels below the folder, links and `node_modules` not entered. A folder with
/// more than the cap of subfolders is `TooBig`, and one that cannot be read is `Unreadable`; both count as unsafe like a
/// nested repository. A folder that vanished during the walk holds nothing.
pub fn git_within(fs: &dyn Fs, path: &str) -> Within {
    git_within_capped(fs, path, WALK_CAP)
}

pub(crate) fn git_within_capped(fs: &dyn Fs, path: &str, cap: i32) -> Within {
    let mut budget = cap;
    walk_for_git(fs, path, 0, &mut budget, &Except::none())
}

/// The same walk for a worktree about to be removed: the `.git` of an initialized submodule (`except`) is that
/// submodule's own, not a nested repository, so the walk goes on below it.
pub(crate) fn git_within_except(fs: &dyn Fs, path: &str, except: &Except) -> Within {
    let mut budget = WALK_CAP;
    walk_for_git(fs, path, 0, &mut budget, except)
}

fn conflict(message: String) -> WtError {
    WtError::new(ErrorCode::Conflict, message)
}

fn forbidden(message: String) -> WtError {
    WtError::new(ErrorCode::Forbidden, message)
}

/// What a folder holds, in a sentence: size, files, newest change.
fn describe(svc: &Service, m: &Leftover) -> String {
    let size = m.size_mb.map_or_else(
        || "unknown size".to_string(),
        |mb| {
            format!(
                "{}{}",
                about_text(Some(mb)),
                if m.node_modules || m.partial {
                    " or more"
                } else {
                    ""
                }
            )
        },
    );
    let files = m.files.map_or_else(|| "?".to_string(), |f| f.to_string());
    format!(
        "{size}, {files} files, newest change {}",
        svc.ago_text(m.newest_ms)
    )
}

impl Service {
    /// The age a leftover must have for this request, in days. The page: what it asks (else the server's limit). A
    /// thread: the server's limit, or less down to 6 hours with a reason; a plain sentence (400) otherwise.
    /// The project's live lead, or the Overlord, may ask for an age below the 6 hour floor.
    fn below_floor_allowed(&self, input: &RemoveInput) -> bool {
        let Some(by) = input.caller.thread() else {
            return false;
        };
        if self.overlord().as_deref() == Some(by) {
            return true;
        }
        self.board(&input.project)
            .ok()
            .and_then(|b| b.threads.get(by).map(|t| t.coordinator && t.live))
            .unwrap_or(false)
    }

    pub(crate) fn age_limit(&self, input: &RemoveInput) -> Result<f64> {
        let limit = self.opts.leftover_age_days;
        let Some(asked) = input
            .older_than_days
            .filter(|d| d.is_finite())
            .map(|d| d.max(0.0))
        else {
            return Ok(limit);
        };
        if input.caller.is_page() || asked >= limit {
            return Ok(asked);
        }
        if asked < THREAD_MIN_AGE_DAYS && !self.below_floor_allowed(input) {
            return Err(WtError::new(
                ErrorCode::BadRequest,
                format!(
                    "a thread deletes a leftover untouched for {} at the least (--older-than 6h), not {}; the owner's page, the project's lead and the Overlord delete younger ones",
                    Self::duration_text(THREAD_MIN_AGE_DAYS),
                    Self::duration_text(asked)
                ),
            ));
        }
        if input.reason.as_deref().is_none_or(|r| r.trim().is_empty()) {
            return Err(WtError::new(
                ErrorCode::BadRequest,
                format!(
                    "--older-than {} is below the server's limit of {}: say why it is safe with --reason \"<why>\" (for example: the finished agent worktrees of this project, nothing runs in them)",
                    Self::duration_text(asked),
                    Self::duration_text(limit)
                ),
            ));
        }
        Ok(asked)
    }

    /// A folder that is no work tree goes the way a leftover goes: measured now, so the change token has something to
    /// compare.
    pub(crate) fn as_leftover(&self, mut fact: Fact, not_git: bool) -> Fact {
        if not_git {
            fact.leftover = Some(measure_folder(
                self.deps.fs.as_ref(),
                &fact.path,
                self.opts.remove_bounds,
                &|| SystemClock.now_ms(),
            ));
        }
        fact
    }

    /// A `.git` the folder has of its own: a folder is one, and a file is one when git reads a work tree from it. A
    /// not-git folder passes with a `.git` file git cannot read.
    fn assert_no_repo_of_its_own(&self, project: &str, fact: &Fact, not_git: bool) -> Result<()> {
        let dot = join_path(&fact.path, ".git");
        let own = self.deps.fs.exists(&dot)
            && (!not_git
                || self.deps.fs.is_dir(&dot)
                || self
                    .deps
                    .git
                    .read(&fact.path, &["rev-parse", "--git-dir"])
                    .ok());
        if !own {
            return Ok(());
        }
        self.request_rescan(project);
        Err(conflict(format!(
            "{} has a .git of its own now: it is not a leftover, nothing was deleted; the listing is being refreshed",
            fact.path
        )))
    }

    /// The repositories that still list a not-git folder: its stale registration goes with the folder.
    fn prune_stale(&self, project: &str, fact: &Fact, repos: &[String]) {
        for repo in repos {
            if let Err(err) = self.prune_repo(repo, &[fact.path.as_str()]) {
                self.note(
                    project,
                    "warn",
                    &format!(
                        "removed {} but could not prune its registration in {repo}: {}",
                        fact.path, err.message
                    ),
                    None,
                );
            }
        }
    }

    fn assert_leftover_path(&self, board: &BoardFacts, rule: &Rule, path: &str) -> Result<()> {
        let rules = board.valid_rules();
        match crate::guards::leftover_path_problem(&rules, rule, path, self.ci()) {
            Some(problem) => Err(forbidden(problem)),
            None => Ok(()),
        }
    }

    /// The repositories a folder of this project may belong to: those of its scanned facts and the known ones.
    pub(crate) fn candidate_repos(&self, project: &str) -> Vec<String> {
        let mut repos: Vec<String> = self
            .facts_of(project)
            .into_iter()
            .filter_map(|f| f.repo)
            .chain(self.known_repos())
            .collect();
        repos.sort();
        repos.dedup();
        repos
    }

    /// Does any repository the scans know list this folder now (read-only git; a failed question is a refusal, not a
    /// yes)? Beyond "listed itself": a folder that HOLDS a worktree git lists, or sits INSIDE a live linked worktree, or
    /// inside a folder with a `.git` of its own, is live work, never a leftover.
    fn assert_not_listed(&self, project: &str, fact: &Fact, not_git: bool) -> Result<Vec<String>> {
        let mut stale = Vec::new();
        let repos = self.candidate_repos(project);
        let (ci, mine) = (self.ci(), path_key(&fact.path, self.ci()));
        for repo in repos.iter().filter(|r| self.deps.fs.exists(r)) {
            let run = self
                .deps
                .git
                .read(repo, &["worktree", "list", "--porcelain"]);
            if !run.ok() {
                return Err(conflict(format!(
                    "could not ask git in {repo} whether it still lists {} ({}): nothing was deleted",
                    fact.path,
                    run.failure_text()
                )));
            }
            let items = parse_worktree_list(&run.stdout);
            if not_git && items.iter().any(|i| path_key(&i.path, ci) == mine) {
                stale.push(repo.clone());
            } else if items.iter().any(|i| path_key(&i.path, ci) == mine) {
                self.request_rescan(project);
                return Err(conflict(format!(
                    "git lists {} again (a worktree of {repo}): it is not a leftover any more, nothing was deleted; the listing is being refreshed",
                    fact.path
                )));
            }
            if let Some(held) = items.iter().find(|i| is_below(&i.path, &fact.path, ci)) {
                return Err(conflict(format!(
                    "{} holds the working tree {} (a worktree of {repo}): it is live work, not a leftover; nothing was deleted",
                    fact.path, held.path
                )));
            }
            if let Some(around) = items
                .iter()
                .skip(1)
                .find(|i| is_below(&fact.path, &i.path, ci))
            {
                return Err(conflict(format!(
                    "{} is inside the working tree {} (a worktree of {repo}): it is part of live work, not a leftover; nothing was deleted",
                    fact.path, around.path
                )));
            }
        }
        match inside_worktree(&fact.path, self.deps.fs.as_ref()) {
            Some(holder) => Err(conflict(format!(
                "{} is inside {holder}, a folder with a .git of its own: it is part of a live work tree, not a leftover; nothing was deleted",
                fact.path
            ))),
            None => Ok(stale),
        }
    }

    /// The fresh, wider look right before a deletion: not a link, no nested repository.
    fn look_before_deleting(&self, path: &str) -> Result<Leftover> {
        let m = measure_folder(
            self.deps.fs.as_ref(),
            path,
            self.opts.remove_bounds,
            &|| SystemClock.now_ms(),
        );
        if m.is_link {
            return Err(forbidden(format!(
                "{path} is a link or junction, not a folder: Argus never follows or deletes one. Remove the link by hand"
            )));
        }
        if m.has_nested_git {
            return Err(forbidden(format!(
                "{path} contains a nested git repository ({}), which may hold real work: Argus does not delete it. Look at it by hand",
                m.nested_git.as_deref().unwrap_or(".git")
            )));
        }
        match git_within(self.deps.fs.as_ref(), path) {
            Within::Nothing => Ok(m),
            Within::Git(deep) => Err(conflict(format!(
                "{path} contains a .git ({deep}) within three levels: it holds a repository or worktree, not a leftover; nothing was deleted"
            ))),
            Within::TooBig => Err(conflict(format!(
                "{path} is too big to check for a nested git repository: Argus cannot tell whether it holds one, so nothing was deleted"
            ))),
            Within::Unreadable(why) => Err(conflict(unreadable_refusal(path, &why, "deleted"))),
        }
    }

    /// A thread is refused a folder it may not judge or one that is too young; the page needs `confirm_young` for one.
    fn check_age(
        &self,
        input: &RemoveInput,
        fact: &Fact,
        m: &Leftover,
        limit_days: f64,
    ) -> Result<()> {
        let approved = matches!(
            input.caller,
            crate::board::Caller::Hygiene {
                approved_by: Some(_)
            }
        );
        if input.empty_goes && !m.partial && (m.files == Some(0) || approved) {
            return Ok(());
        }
        let made = self.is_made(&input.project, &fact.path);
        let young = !made
            && m.newest_ms
                .is_none_or(|n| self.now() - n < age_ms(limit_days));
        let ago = self.ago_text(m.newest_ms);
        let limit = Self::duration_text(limit_days);
        if input.caller.is_page() {
            if young && !input.confirm_young {
                return Err(forbidden(format!(
                    "{} changed {ago}, less than {limit} ago: confirm that you want a young folder deleted (confirmYoung)",
                    fact.path
                )));
            }
            return Ok(());
        }
        if m.partial {
            let bounds = self.opts.remove_bounds;
            return Err(forbidden(format!(
                "{} is too big to look through (more than {} entries or {} s): only the owner's page deletes it",
                fact.path,
                bounds.max_entries,
                bounds.max_ms / 1000
            )));
        }
        if young {
            return Err(forbidden(format!(
                "{} changed {ago}, less than {limit} ago: a thread only deletes a leftover nothing touched for {limit} (or, with --older-than and --reason, for at least 6 hours). The owner can delete it from the page",
                fact.path
            )));
        }
        Ok(())
    }

    fn check_token(&self, input: &RemoveInput, fact: &Fact, m: &Leftover) -> Result<()> {
        let seen = fact.leftover.clone().unwrap_or_default();
        if let (Some(asked), false, Some(listed)) =
            (input.newest_at_ms, seen.pending, seen.newest_ms)
        {
            if asked != listed {
                return Err(conflict(format!(
                    "{} changed since you looked (its newest change is {}, not {}): look again before deleting it",
                    fact.path,
                    iso_from_ms(listed),
                    iso_from_ms(asked)
                )));
            }
        }
        if let (false, false, Some(listed), Some(now)) =
            (seen.pending, seen.partial, seen.newest_ms, m.newest_ms)
        {
            if now > listed + 1000 {
                self.request_rescan(&input.project);
                return Err(conflict(format!(
                    "{} changed since the listing (a file was written at {}): nothing was deleted, the listing is being refreshed",
                    fact.path,
                    iso_from_ms(now)
                )));
            }
        }
        Ok(())
    }

    /// The guards of one leftover and its rename; the job is queued for deletion unless a batch runs it.
    pub(crate) fn start_leftover(
        &self,
        input: &RemoveInput,
        (fact, not_git): (&Fact, bool),
        rule: &Rule,
        board: &BoardFacts,
    ) -> Result<RemoveOutcome> {
        let branch_asked = input.branch && !input.empty_goes;
        if !not_git && (branch_asked || input.discard) {
            return Err(WtError::new(
                ErrorCode::BadRequest,
                format!(
                    "{} is a leftover folder: it has no branch and nothing git knows, so --branch and --discard do not apply (run it without them; no backup is possible)",
                    fact.path
                ),
            ));
        }
        let limit_days = if input.caller.is_page() {
            self.opts.leftover_age_days
        } else {
            self.age_limit(input)?
        };
        if let Some(busy) = self
            .active_job_for(&input.project, &fact.id)
            .filter(|j| Some(&j.id) != input.job.as_ref())
        {
            return Err(conflict(format!(
                "{} is already being deleted (job {}, {})",
                fact.path,
                busy.id,
                busy.state.as_str()
            )));
        }
        self.assert_leftover_path(board, rule, &fact.path)?;
        self.assert_no_repo_of_its_own(&input.project, fact, not_git)?;
        let stale = self.assert_not_listed(&input.project, fact, not_git)?;
        let m = self.look_before_deleting(&fact.path)?;
        if let Some(user) = self.busy_by(&fact.path, input.job.is_some(), "deleted")? {
            return Err(Self::in_use(&fact.path, &user, "deleted"));
        }
        self.check_age(input, fact, &m, limit_days)?;
        self.check_token(input, fact, &m)?;
        let outcome = self.rename_and_queue(input, fact, &m)?;
        self.prune_stale(&input.project, fact, &stale);
        Ok(outcome)
    }

    fn rename_and_queue(
        &self,
        input: &RemoveInput,
        fact: &Fact,
        m: &Leftover,
    ) -> Result<RemoveOutcome> {
        let target = self.deleting_name(&fact.path);
        if let Err(err) = self.deps.fs.rename(&fact.path, &target) {
            return Err(self.rename_refusal(&fact.path, &err));
        }
        let by = input.caller.by_text();
        let job_id = input
            .job
            .clone()
            .unwrap_or_else(|| self.new_job(&input.project, fact, &by, JobKind::Leftover));
        let folder = crate::paths::clean_path(&target);
        self.update_job(&job_id, |j| {
            j.folder.clone_from(&folder);
            (j.size_mb, j.files, j.by) = (m.size_mb, m.files, by);
            j.state = crate::jobs::JobState::Queued;
        });
        self.forget(&input.project, &fact.id);
        if input.job.is_none() {
            self.enqueue_delete(&job_id);
        }
        Ok(RemoveOutcome {
            started: true,
            leftover: true,
            job_id: Some(job_id),
            path: Some(fact.path.clone()),
            folder: Some(folder),
            size_mb: m.size_mb,
            files: m.files,
            newest_at: m.newest_ms.map(iso_from_ms),
            text: format!(
                "deleting leftover folder {} ({}; git does not know it, so there is no backup)",
                fact.path,
                describe(self, m)
            ),
            ..RemoveOutcome::default()
        })
    }
}
