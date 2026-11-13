//! `coord worktree approve <id>...`: the owning lead (or a thread the worktree's rule allows) or the Overlord approves
//! removing worktrees that hold work. Argus files ONE `approved` request for every id that passes, so the queue's cap
//! counts it once: the worker writes a full backup of each, then removes it with what is left on it (its branch goes
//! when merged and stays when not: the approval covers the work in the folder, never an unmerged branch). The route does no git and no backup: it checks the ids against the last scan and the board, once. The result
//! comes to the approver's inbox, on the Overlord's own board when he approved from outside the project.

use super::memory::HygieneState;
use super::release::why_not;
use crate::board::{BoardFacts, Caller};
use crate::error::{ErrorCode, Result, WtError};
use crate::fact::Fact;
use crate::guards::check_caller;
use crate::remove::forbidden;
use crate::request::RemoveInput;
use crate::requests::{MAX_TARGETS, Queued, RequestInput, Target};
use crate::service::Service;
use serde::Serialize;

/// Several worktrees approved in one word.
#[derive(Clone, Debug)]
pub struct ApproveInput {
    pub project: String,
    /// Worktree ids or paths.
    pub ids: Vec<String>,
    pub caller: Caller,
    pub reason: Option<String>,
}

/// An id the approval did not take, and why.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NotApproved {
    pub id: String,
    pub code: String,
    pub text: String,
}

/// The answer to an approval of several: the one request filed for those that passed (none when none did) and the ids
/// that did not.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Approved {
    pub queued: Option<Queued>,
    pub refused: Vec<NotApproved>,
}

#[derive(Default)]
struct Vetted {
    facts: Vec<Fact>,
    refused: Vec<(String, WtError)>,
    inbox: Option<String>,
}

fn approver_name(caller: &Caller) -> Result<String> {
    caller.thread().map(ToString::to_string).ok_or_else(|| {
        forbidden(
            "approve is a lead's or the Overlord's word; the owner's page removes directly (Delete now)"
                .to_string(),
        )
    })
}

impl Service {
    /// May `by` approve this removal? The Overlord always may (his result lands on his board when he is not a thread
    /// of the project); the owning lead may; any other thread as its rule allows a removal.
    fn approver(
        &self,
        project: &str,
        board: &BoardFacts,
        fact: &Fact,
        by: &str,
    ) -> Result<Option<String>> {
        if self.overlord().as_deref() == Some(by) {
            return Ok(board
                .live_thread(by)
                .is_none()
                .then(|| self.deps.store.overlord_board()));
        }
        let lead = self.with_hygiene(project, |m| {
            m.marks.get(&fact.id).and_then(|mark| mark.thread.clone())
        });
        if lead.as_deref() == Some(by) && board.live_thread(by).is_some() {
            return Ok(None);
        }
        let rule = board.rules.iter().find(|r| r.id == fact.rule);
        let caller = Caller::Thread(by.to_string());
        check_caller(board, &caller, &Self::caller_rules(board, rule)).map(|()| None)
    }

    /// Approve the removal of one worktree with what is left on it. Refused while anything is attached to it, while
    /// it is an exception, and for a folder the hygiene does not remove (a main checkout, one outside its rule).
    pub fn approve(&self, input: &RemoveInput) -> Result<Queued> {
        let by = approver_name(&input.caller)?;
        let board = self.board(&input.project)?;
        let ids = [input.id.clone()];
        let mut vetted = self.vet_all(&input.project, &board, &by, &ids, &input.caller);
        if let Some((_, err)) = vetted.refused.pop() {
            return Err(err);
        }
        self.file_approved(&input.project, &by, vetted, input.reason.as_deref())
    }

    /// Approve several worktrees at once: every id is checked against one listing, the ones that pass become ONE
    /// request, and each one that does not is reported with its sentence. Nothing is read from git here.
    pub fn approve_many(&self, input: &ApproveInput) -> Result<Approved> {
        let by = approver_name(&input.caller)?;
        if input.ids.is_empty() {
            return Err(WtError::new(
                ErrorCode::BadRequest,
                "name at least one worktree to approve",
            ));
        }
        if input.ids.len() > MAX_TARGETS {
            return Err(WtError::new(
                ErrorCode::BadRequest,
                format!(
                    "an approval names {MAX_TARGETS} worktrees at most ({} given)",
                    input.ids.len()
                ),
            ));
        }
        let board = self.board(&input.project)?;
        let mut vetted = self.vet_all(&input.project, &board, &by, &input.ids, &input.caller);
        let refused = std::mem::take(&mut vetted.refused)
            .into_iter()
            .map(|(id, err)| NotApproved {
                id,
                code: err.code.as_str().to_string(),
                text: err.message,
            })
            .collect();
        if vetted.facts.is_empty() {
            return Ok(Approved {
                queued: None,
                refused,
            });
        }
        let queued = self.file_approved(&input.project, &by, vetted, input.reason.as_deref())?;
        Ok(Approved {
            queued: Some(queued),
            refused,
        })
    }

    /// Check each id once against the last scan and the board; the same worktree named twice counts once.
    fn vet_all(
        &self,
        project: &str,
        board: &BoardFacts,
        by: &str,
        ids: &[String],
        caller: &Caller,
    ) -> Vetted {
        let mut vetted = Vetted::default();
        for raw in ids {
            let lookup = RemoveInput::new(project, raw, caller.clone());
            match self.vet_one(&lookup, board, by) {
                Ok((fact, inbox)) => {
                    vetted.inbox = inbox;
                    if !vetted.facts.iter().any(|f| f.id == fact.id) {
                        vetted.facts.push(fact);
                    }
                }
                Err(err) => vetted.refused.push((raw.trim().to_string(), err)),
            }
        }
        vetted
    }

    fn vet_one(
        &self,
        lookup: &RemoveInput,
        board: &BoardFacts,
        by: &str,
    ) -> Result<(Fact, Option<String>)> {
        let project = &lookup.project;
        let fact = self.listed_in(lookup, board)?;
        let inbox = self.approver(project, board, &fact, by)?;
        Self::worktree_only(&fact, "approve")?;
        let verdict = self.look_now(project, board, &fact, |look| {
            self.judge(project, look, &fact).0
        });
        if let Some(why) = why_not(&verdict, true) {
            self.note(
                project,
                "warn",
                &format!("refused the approval of {} by {by}: {why}", fact.path),
                Some(by),
            );
            return Err(forbidden(format!(
                "{} was not approved for removal: {why}. Nothing was removed",
                fact.path
            )));
        }
        Ok((fact, inbox))
    }

    /// File the one request of every worktree that passed. The backup and the removal are the worker's.
    fn file_approved(
        &self,
        project: &str,
        by: &str,
        vetted: Vetted,
        reason: Option<&str>,
    ) -> Result<Queued> {
        let mut ask = RequestInput::new(project, Caller::Thread(by.to_string()));
        ask.targets = vetted.facts.iter().map(|f| f.id.clone()).collect();
        ask.discard = true;
        ask.branch = true;
        ask.inbox = vetted.inbox;
        ask.reason = Some(reason.map_or_else(|| format!("approved by {by}"), ToString::to_string));
        let targets = vetted
            .facts
            .iter()
            .map(|f| Target {
                id: f.id.clone(),
                label: f.path.clone(),
            })
            .collect();
        let mut queued = self.request_approved(&ask, targets)?;
        self.with_hygiene(project, |m| {
            for fact in &vetted.facts {
                if let Some(mark) = m.marks.get_mut(&fact.id) {
                    mark.request = Some(queued.id.clone());
                    mark.state = HygieneState::Removing;
                }
            }
        });
        queued.text = match vetted.facts.as_slice() {
            [one] => format!(
                "{}: Argus writes a full backup of {}, then removes it with what is left (its branch goes when merged and stays when not); the result comes to your inbox",
                queued.id, one.path
            ),
            many => format!(
                "{}: Argus writes a full backup of each of {} worktrees, then removes each with what is left (each branch goes when merged and stays when not); the result comes to your inbox",
                queued.id,
                many.len()
            ),
        };
        Ok(queued)
    }
}
