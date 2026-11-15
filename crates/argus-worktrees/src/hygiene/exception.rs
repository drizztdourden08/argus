//! The exception list: `coord worktree keep <id> --note "why"`. Only the Overlord writes a confirmed exception: a
//! worktree on it is never removed and never reported. A thread that calls keep files a PENDING keep instead (see
//! `pending`): the hygiene holds the worktree for at most 24 hours while the Overlord confirms or declines it. Keeps
//! written on the board before 2026-11-06 do not hold a worktree here.

use super::memory::Exception;
use crate::board::Caller;
use crate::error::{ErrorCode, Result, WtError};
use crate::request::RemoveInput;
use crate::service::Service;
use serde::Serialize;

const NOTE_MAX: usize = 300;

/// What the Overlord writes on the exception list, or clears from it.
#[derive(Clone, Debug)]
pub struct KeepInput {
    pub project: String,
    pub caller: Caller,
    /// The board the caller speaks from. Every lead's thread is named `coordinator`, so the name alone proves
    /// nothing: only a caller from the `portfolio` board who is that board's coordinator is the Overlord.
    pub by_project: Option<String>,
    /// The worktree's id or path.
    pub id: String,
    pub note: Option<String>,
    pub until_ms: Option<i64>,
    /// The lead that asked for it.
    pub asked_by: Option<String>,
    /// Take it off the list instead.
    pub clear: bool,
}

/// The answer to a keep.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Kept {
    pub id: String,
    pub text: String,
}

pub(super) fn bad(message: impl Into<String>) -> WtError {
    WtError::new(ErrorCode::BadRequest, message)
}

impl Service {
    /// The Overlord's thread when the caller is he: a thread of the `portfolio` board that is its coordinator.
    fn as_overlord(&self, input: &KeepInput) -> Option<String> {
        let by = input.caller.thread().unwrap_or_default().to_string();
        let from_portfolio =
            input.by_project.as_deref() == Some(self.deps.store.overlord_board().as_str());
        (from_portfolio && !by.is_empty() && self.overlord().as_deref() == Some(by.as_str()))
            .then_some(by)
    }

    /// The reason of a keep: needed, trimmed and cut.
    pub(super) fn reason_of(input: &KeepInput) -> Result<String> {
        let reason: String = input
            .note
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .chars()
            .take(NOTE_MAX)
            .collect();
        if reason.is_empty() {
            return Err(bad("an exception needs its reason: --note \"why\""));
        }
        Ok(reason)
    }

    /// Write or clear one exception. A thread files or withdraws a pending keep; the page is refused.
    pub fn keep(&self, input: &KeepInput) -> Result<Kept> {
        let Some(by) = self.as_overlord(input) else {
            return self.thread_keep(input);
        };
        let lookup = RemoveInput::new(&input.project, &input.id, Caller::Page);
        let (_, fact) = self.listed_fact(&lookup)?;
        if input.clear {
            let prior = self.with_hygiene(&input.project, |m| m.exceptions.remove(&fact.id));
            self.note(
                &input.project,
                "info",
                &format!("{by} took {} off the exception list", fact.path),
                Some(&by),
            );
            self.tell_answer(&input.project, prior.as_ref(), false);
            return Ok(Kept {
                id: fact.id,
                text: format!(
                    "{} is off the exception list: the hygiene judges it again",
                    fact.path
                ),
            });
        }
        let reason = Self::reason_of(input)?;
        let now = self.now();
        if input.until_ms.is_some_and(|u| u <= now) {
            return Err(bad("--until must be in the future"));
        }
        let prior = self.with_hygiene(&input.project, |m| m.exceptions.get(&fact.id).cloned());
        let exception = Exception {
            path: fact.path.clone(),
            reason: reason.clone(),
            asked_by: input
                .asked_by
                .clone()
                .filter(|a| !a.trim().is_empty())
                .or_else(|| prior.as_ref().and_then(|p| p.asked_by.clone())),
            by: by.clone(),
            at: now,
            until: input.until_ms,
            pending: false,
        };
        self.with_hygiene(&input.project, |m| {
            m.exceptions.insert(fact.id.clone(), exception);
        });
        self.note(
            &input.project,
            "info",
            &format!("{by} put {} on the exception list: {reason}", fact.path),
            Some(&by),
        );
        self.tell_answer(&input.project, prior.as_ref(), true);
        Ok(Kept {
            id: fact.id,
            text: format!(
                "{} is on the exception list: Argus never removes or reports it",
                fact.path
            ),
        })
    }
}
