//! Removing released worktrees: the look again right before each removal, what the request worker leaves in the
//! hygiene, and a thread's own `release`.

use super::judge::{Look, Verdict};
use super::memory::{Refused, RemovedDoc};
use crate::clock::iso_from_ms;
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::paths::{clean_path, is_worktree_id, worktree_id};
use crate::remove::forbidden;
use crate::request::{RemoveInput, RemoveOutcome};
use crate::requests::{Mode, Outcome, RequestRec, Target};
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
    /// processes listed now. A request of another mode passes.
    pub(crate) fn still_released(
        &self,
        rec: &RequestRec,
        target: &Target,
    ) -> std::result::Result<(), Outcome> {
        if rec.mode != Mode::Released {
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
                match self.judge(&rec.key, &look, &fact) {
                    (Verdict::Released, owner)
                        if owner.thread.as_deref() == Some(rec.by.as_str()) =>
                    {
                        return Ok(());
                    }
                    (verdict, _) => Self::why_not(&verdict),
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

    /// A request of released worktrees ended: what went is listed for the page, what the removal refused is not
    /// released again while it stays as it is, and the rest is judged afresh by the next step.
    pub(crate) fn after_released(&self, project: &str, id: &str) {
        let Some(rec) = self
            .request_of(project, id)
            .filter(|r| r.mode == Mode::Released)
        else {
            return;
        };
        let facts = self.facts_of(project);
        let at = iso_from_ms(self.now());
        self.with_hygiene(project, |memory| {
            for result in &rec.results {
                if result.ok {
                    let branch = memory.marks.get(&result.id).and_then(|m| m.branch.clone());
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

    /// `coord worktree release <id|path>`: the thread no longer needs the worktree. After a fresh scan Argus checks
    /// that nothing uses it and that removing it loses nothing, then removes it at once through the same removal (the
    /// branch too when merged). Anything that is not safe is refused: the thread removes it itself if it means to.
    pub fn release(&self, input: &RemoveInput) -> Result<RemoveOutcome> {
        let Some(thread) = input.caller.thread().map(ToString::to_string) else {
            return Err(forbidden(
                "release is a thread's word; the owner's page removes directly (Delete now)"
                    .to_string(),
            ));
        };
        let _ = self.scan_now(&input.project);
        let (board, fact) = self.listed_fact(input)?;
        let rule = board.rules.iter().find(|r| r.id == fact.rule);
        crate::guards::check_caller(&board, &input.caller, &Self::caller_rules(&board, rule))?;
        if !Self::judged(&fact) {
            return Err(WtError::new(
                ErrorCode::Conflict,
                format!(
                    "{} is not a linked git worktree: release takes a worktree (coord worktree remove takes a leftover folder)",
                    fact.path
                ),
            ));
        }
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
        let mut asked = input.clone();
        asked.id.clone_from(&fact.id);
        asked.branch = true;
        asked
            .reason
            .get_or_insert_with(|| format!("released by {thread}"));
        let out = self.remove(&asked)?;
        let at = iso_from_ms(self.now());
        self.with_hygiene(&input.project, |memory| {
            memory.removed_one(RemovedDoc {
                id: fact.id.clone(),
                path: fact.path.clone(),
                branch: fact.branch.clone(),
                thread: Some(thread.clone()),
                at,
                text: out.text.clone(),
            });
        });
        Ok(out)
    }
}
