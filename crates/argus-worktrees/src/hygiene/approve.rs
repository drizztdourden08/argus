//! `coord worktree approve <id>`: the owning lead (or a thread the worktree's rule allows) or the Overlord approves
//! removing a worktree that holds work. Argus files one `approved` request: the worker writes a full backup, then
//! removes the worktree with what is left on it (its branch is kept). The result comes to the approver's inbox, on the
//! Overlord's own board when he approved from outside the project.

use super::memory::HygieneState;
use super::release::why_not;
use crate::board::{BoardFacts, Caller};
use crate::error::Result;
use crate::fact::Fact;
use crate::guards::check_caller;
use crate::remove::forbidden;
use crate::request::RemoveInput;
use crate::requests::{Mode, Queued, RequestInput};
use crate::service::Service;
use argus_core::ids::OVERLORD_PROJECT;

impl Service {
    /// May `by` approve this removal? The Overlord always may (his result lands on his board when he is not a thread
    /// of the project); the owning lead may; any other thread as its rule allows a removal.
    fn approver(&self, project: &str, board: &BoardFacts, fact: &Fact, by: &str) -> Result<Option<String>> {
        if self.overlord().as_deref() == Some(by) {
            return Ok(board
                .live_thread(by)
                .is_none()
                .then(|| OVERLORD_PROJECT.to_string()));
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
        let Some(by) = input.caller.thread().map(ToString::to_string) else {
            return Err(forbidden(
                "approve is a lead's or the Overlord's word; the owner's page removes directly (Delete now)"
                    .to_string(),
            ));
        };
        let (board, listed) = self.listed_fact(input)?;
        let inbox = self.approver(&input.project, &board, &listed, &by)?;
        Self::worktree_only(&listed, "approve")?;
        let rule = board.rules.iter().find(|r| r.id == listed.rule);
        let fact = self.read_now(&listed, rule)?;
        let verdict = self.look_now(&input.project, &board, &fact, |look| {
            self.judge(&input.project, look, &fact).0
        });
        if let Some(why) = why_not(&verdict, true) {
            self.note(
                &input.project,
                "warn",
                &format!("refused the approval of {} by {by}: {why}", fact.path),
                Some(&by),
            );
            return Err(forbidden(format!(
                "{} was not approved for removal: {why}. Nothing was removed",
                fact.path
            )));
        }
        let mut ask = RequestInput::new(&input.project, Caller::Thread(by.clone()));
        ask.targets = vec![fact.id.clone()];
        ask.discard = true;
        ask.inbox = inbox;
        ask.reason = Some(
            input
                .reason
                .clone()
                .unwrap_or_else(|| format!("approved by {by}")),
        );
        let mut queued = self.request_hygiene(&ask, Mode::Approved)?;
        self.with_hygiene(&input.project, |m| {
            if let Some(mark) = m.marks.get_mut(&fact.id) {
                mark.request = Some(queued.id.clone());
                mark.state = HygieneState::Removing;
            }
        });
        queued.text = format!(
            "{}: Argus writes a full backup of {}, then removes it with what is left (its branch stays); the result comes to your inbox",
            queued.id, fact.path
        );
        Ok(queued)
    }
}
