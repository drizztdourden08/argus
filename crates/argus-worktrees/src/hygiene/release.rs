//! Removing what the hygiene hands on: the look again right before each removal, what the request worker leaves in
//! the hygiene, a fresh read of one worktree, and a thread's own `release`.

use super::judge::{Look, Verdict};
use super::memory::{Refused, RemovedDoc};
use crate::board::BoardFacts;
use crate::clock::iso_from_ms;
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::inspect::{ScanCtx, inspect_dir};
use crate::paths::{clean_path, is_worktree_id, worktree_id};
use crate::remove::forbidden;
use crate::request::RemoveInput;
use crate::requests::{Mode, Outcome, Queued, RequestInput, RequestRec, Target};
use crate::rule::Rule;
use crate::service::Service;

/// The code of a target the look again found attached, excepted or changed.
const NOT_FREE: &str = "not_free";

/// Why a removal may not go on for this verdict, or none: a clean worktree always may, one that holds work only once
/// its lead or the Overlord approved.
pub(super) fn why_not(verdict: &Verdict, approved: bool) -> Option<String> {
    match verdict {
        Verdict::Clean => None,
        Verdict::Left(_) if approved => None,
        Verdict::Left(why) | Verdict::Attached(why) => Some(why.clone()),
        Verdict::Exception(reason) => {
            Some(format!("it is on the Overlord's exception list ({reason})"))
        }
        Verdict::Quiet => Some("it is not a worktree Argus removes by itself".to_string()),
    }
}

impl Service {
    /// The look of one worktree right now: the board, the runs and the processes listed afresh.
    pub(super) fn look_now<T>(
        &self,
        project: &str,
        board: &BoardFacts,
        fact: &Fact,
        judge: impl FnOnce(&Look<'_>) -> T,
    ) -> T {
        let runs = self.deps.store.live_runs();
        let (refused, exception) = self.with_hygiene(project, |m| {
            (
                m.refused.get(&fact.id).cloned(),
                m.exceptions.get(&fact.id).cloned(),
            )
        });
        let look = Look {
            board,
            runs: &runs,
            scanned_ms: self.now(),
            fresh: true,
            refused: refused.as_ref(),
            exception: exception.as_ref(),
        };
        judge(&look)
    }

    /// Right before a hygiene removal, and a thread's own release: the verdict again, on the fresh scan, the board now
    /// and the processes listed now. A detached worktree and a release must still be clean and unattached; an
    /// approved one must still be unattached and not an exception. A request of another mode passes.
    pub(crate) fn still_free(
        &self,
        rec: &RequestRec,
        target: &Target,
    ) -> std::result::Result<(), Outcome> {
        if !matches!(rec.mode, Mode::Detached | Mode::Approved) && !rec.release {
            return Ok(());
        }
        let found = self.deps.store.board(&rec.key).and_then(|board| {
            self.fact_of(&rec.key, &target.id, &board)
                .map(|f| (board, f))
        });
        let why = match found {
            None => "it is not in the listing any more".to_string(),
            Some((board, fact)) => {
                let verdict = self.look_now(&rec.key, &board, &fact, |look| {
                    self.judge(&rec.key, look, &fact).0
                });
                match why_not(&verdict, rec.mode == Mode::Approved) {
                    None => return Ok(()),
                    Some(why) => why,
                }
            }
        };
        Err(Outcome {
            id: target.id.clone(),
            label: target.label.clone(),
            ok: false,
            text: format!("{} was not removed: {why}", target.label),
            code: Some(NOT_FREE.to_string()),
        })
    }

    /// A hygiene request or a release ended: what went is listed for the page; what the removal refused is clean no
    /// more while it stays as it is (its lead is asked about it); the mark lets go of the request, keeping when its
    /// lead was asked and when the Overlord was told.
    pub(crate) fn after_hygiene(&self, project: &str, id: &str) {
        let Some(rec) = self
            .request_of(project, id)
            .filter(|r| matches!(r.mode, Mode::Detached | Mode::Approved) || r.release)
        else {
            return;
        };
        let facts = self.facts_of(project);
        let at = iso_from_ms(self.now());
        self.with_hygiene(project, |memory| {
            for result in &rec.results {
                let fact = facts.iter().find(|f| f.id == result.id);
                if result.ok {
                    let branch = memory
                        .marks
                        .get(&result.id)
                        .and_then(|m| m.branch.clone())
                        .or_else(|| fact.and_then(|f| f.branch.clone()));
                    memory.removed_one(RemovedDoc {
                        id: result.id.clone(),
                        path: result.label.clone(),
                        branch,
                        thread: Some(rec.by.clone()),
                        at: at.clone(),
                        text: result.text.clone(),
                    });
                    continue;
                }
                if let Some(mark) = memory
                    .marks
                    .get_mut(&result.id)
                    .filter(|m| m.request.as_deref() == Some(id))
                {
                    mark.request = None;
                }
                if let Some(fact) = fact.filter(|_| result.code.as_deref() != Some(NOT_FREE)) {
                    memory.refused.insert(
                        result.id.clone(),
                        Refused {
                            head: fact.head.clone(),
                            dirty: fact.dirty,
                            why: result.text.clone(),
                        },
                    );
                }
            }
        });
    }

    /// The listed worktree an id or a path names, with the board.
    pub(super) fn listed_fact(&self, input: &RemoveInput) -> Result<(BoardFacts, Fact)> {
        let board = self.board(&input.project)?;
        let id = input.id.trim();
        let wid = if is_worktree_id(id) {
            id.to_string()
        } else {
            worktree_id(&clean_path(id), self.ci())
        };
        match self.fact_of(&input.project, &wid, &board) {
            Some(fact) => Ok((board, fact)),
            None => Err(self.not_listed(input, &board, &wid)),
        }
    }

    /// The worktree as it is now, read with git on this thread. It does not take the turn of scans and removals: a
    /// release or an approval must answer while a removal request runs (a scan of the project would wait for it,
    /// minutes). The request worker scans and looks again right before the removal.
    pub(super) fn read_now(&self, listed: &Fact, rule: Option<&Rule>) -> Result<Fact> {
        let Some(rule) = rule else {
            return Ok(listed.clone());
        };
        let mut ctx = ScanCtx {
            git: &self.deps.git,
            fs: self.deps.fs.as_ref(),
            lists: Vec::new(),
            case_insensitive: self.ci(),
        };
        let now = inspect_dir(&listed.path, rule, &mut ctx);
        match &now.error {
            Some(why) => Err(WtError::new(
                ErrorCode::GitFailed,
                format!(
                    "git could not read the state of {} ({why}), so Argus cannot tell whether work would be lost: nothing was done",
                    listed.path
                ),
            )),
            None => Ok(now),
        }
    }

    /// A linked git worktree, or the sentence that says the verb takes one.
    pub(super) fn worktree_only(listed: &Fact, verb: &str) -> Result<()> {
        if Self::judged(listed) {
            return Ok(());
        }
        Err(WtError::new(
            ErrorCode::Conflict,
            format!(
                "{} is not a linked git worktree: {verb} takes a worktree (coord worktree remove takes a leftover folder)",
                listed.path
            ),
        ))
    }

    /// `coord worktree release <id|path>`: the thread no longer needs the worktree. After a fresh look at that one
    /// worktree (never a scan of the project: that would wait for a running removal) Argus checks that nothing is
    /// attached and that removing it loses nothing, then files the removal as a request of the thread (the branch too
    /// when merged) and answers at once: the worker carries it out and the result comes to the thread's inbox.
    /// Anything that holds work is refused: the lead approves that one (`coord worktree approve`).
    pub fn release(&self, input: &RemoveInput) -> Result<Queued> {
        let Some(thread) = input.caller.thread().map(ToString::to_string) else {
            return Err(forbidden(
                "release is a thread's word; the owner's page removes directly (Delete now)"
                    .to_string(),
            ));
        };
        let (board, listed) = self.listed_fact(input)?;
        let rule = board.rules.iter().find(|r| r.id == listed.rule);
        crate::guards::check_caller(&board, &input.caller, &Self::caller_rules(&board, rule))?;
        Self::worktree_only(&listed, "release")?;
        let fact = self.read_now(&listed, rule)?;
        let verdict = self.look_now(&input.project, &board, &fact, |look| {
            self.judge(&input.project, look, &fact).0
        });
        if let Some(why) = why_not(&verdict, false) {
            let text = format!(
                "{} is not released: {why}. Argus removes a worktree on a release only when nothing is attached and nothing is lost; its lead approves the rest (coord worktree approve <id>). Nothing was removed",
                fact.path
            );
            self.note(
                &input.project,
                "warn",
                &format!("refused to release {} for {thread}: {why}", fact.path),
                Some(&thread),
            );
            return Err(forbidden(text));
        }
        let mut ask = RequestInput::new(&input.project, input.caller.clone());
        ask.targets = vec![fact.id.clone()];
        ask.branch = true;
        ask.reason = Some(
            input
                .reason
                .clone()
                .unwrap_or_else(|| format!("released by {thread}")),
        );
        let mut queued = self.request_release(&ask)?;
        queued.text = format!(
            "{}: Argus removes it now; the result comes to your inbox (coord worktree requests shows it)",
            queued.id
        );
        Ok(queued)
    }
}
