//! Removing released worktrees: the look again right before each removal, what the request worker leaves in the
//! hygiene, and a thread's own `release`.

use super::judge::{Look, Verdict};
use super::memory::{Refused, RemovedDoc};
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

/// The code of a target the look again found no longer released.
const NOT_RELEASED: &str = "not_released";

impl Service {
    fn why_not(verdict: &Verdict) -> String {
        match verdict {
            Verdict::Released => "its thread changed".to_string(),
            Verdict::Quiet => "it is not a worktree Argus removes by itself".to_string(),
            Verdict::Kept(_) => "the thread kept it".to_string(),
            Verdict::InUse(why) | Verdict::Review(why) => why.clone(),
        }
    }

    /// Right before a released worktree is removed: every condition again, on the fresh scan, the board now and the
    /// processes listed now. A request of another mode passes, except a thread's own release: nothing may use the
    /// worktree and nothing may be lost, as when it asked.
    pub(crate) fn still_released(
        &self,
        rec: &RequestRec,
        target: &Target,
    ) -> std::result::Result<(), Outcome> {
        if rec.mode != Mode::Released && !rec.release {
            return Ok(());
        }
        let fact = self.deps.store.board(&rec.key).and_then(|board| {
            self.fact_of(&rec.key, &target.id, &board)
                .map(|f| (board, f))
        });
        let why = match fact {
            None => "it is not in the listing any more".to_string(),
            Some((board, fact)) => {
                let runs = self.deps.store.live_runs();
                let refused = self.with_hygiene(&rec.key, |m| m.refused.get(&fact.id).cloned());
                let look = Look {
                    board: &board,
                    runs: &runs,
                    scanned_ms: self.now(),
                    fresh: true,
                    refused: refused.as_ref(),
                };
                match self.looked_again(rec, &look, &fact) {
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
            code: Some(NOT_RELEASED.to_string()),
        })
    }

    /// Why the worktree of a request is no longer to be removed, or none when it still is.
    fn looked_again(&self, rec: &RequestRec, look: &Look<'_>, fact: &Fact) -> Option<String> {
        if rec.release {
            return self.blocked_why(look, fact);
        }
        match self.judge(&rec.key, look, fact) {
            (Verdict::Released, owner) if owner.thread.as_deref() == Some(rec.by.as_str()) => None,
            (verdict, _) => Some(Self::why_not(&verdict)),
        }
    }

    /// A request of released worktrees ended: what went is listed for the page, what the removal refused is not
    /// released again while it stays as it is, and the rest is judged afresh by the next step.
    pub(crate) fn after_released(&self, project: &str, id: &str) {
        let Some(rec) = self
            .request_of(project, id)
            .filter(|r| r.mode == Mode::Released || r.release)
        else {
            return;
        };
        let facts = self.facts_of(project);
        let at = iso_from_ms(self.now());
        self.with_hygiene(project, |memory| {
            for result in &rec.results {
                if result.ok {
                    let branch = memory
                        .marks
                        .get(&result.id)
                        .and_then(|m| m.branch.clone())
                        .or_else(|| {
                            facts
                                .iter()
                                .find(|f| f.id == result.id)
                                .and_then(|f| f.branch.clone())
                        });
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
                let carried = memory
                    .marks
                    .get(&result.id)
                    .is_some_and(|m| m.request.as_deref() == Some(id));
                if carried {
                    memory.marks.remove(&result.id);
                }
                if result.code.as_deref() != Some(NOT_RELEASED) {
                    if let Some(fact) = facts.iter().find(|f| f.id == result.id) {
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
            }
        });
    }

    fn listed_fact(&self, input: &RemoveInput) -> Result<(crate::board::BoardFacts, Fact)> {
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
    /// release must answer while a removal request runs (a scan of the project would wait for it, minutes). The
    /// request worker scans and looks again right before the removal.
    fn read_now(&self, listed: &Fact, rule: Option<&Rule>) -> Result<Fact> {
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

    /// `coord worktree release <id|path>`: the thread no longer needs the worktree. After a fresh look at that one
    /// worktree (never a scan of the project: that would wait for a running removal) Argus checks that nothing uses it
    /// and that removing it loses nothing, then files the removal as a request of the thread (the branch too when
    /// merged) and answers at once: the removal takes minutes, so the worker carries it out and the result comes to the
    /// thread's inbox. Anything that is not safe is refused: the thread removes it itself if it means to.
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
        if !Self::judged(&listed) {
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} is not a linked git worktree: release takes a worktree (coord worktree remove takes a leftover folder)",
                    listed.path
                ),
            ));
        }
        let fact = self.read_now(&listed, rule)?;
        let runs = self.deps.store.live_runs();
        let look = Look {
            board: &board,
            runs: &runs,
            scanned_ms: self.now(),
            fresh: true,
            refused: None,
        };
        if let Some(why) = self.blocked_why(&look, &fact) {
            let text = format!(
                "{} is not released: {why}. Argus removes a worktree by itself only when nothing uses it and nothing is lost; nothing was removed",
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
