//! The exception list: `coord worktree keep <id> --note "why"`, the Overlord's word only. A worktree on it is never
//! removed and never reported. A lead asks the Overlord for one (`coord lead portfolio "..."`); a thread that calls
//! keep itself is refused and told so. Keeps written on the board before 2026-11-06 do not hold a worktree here.

use super::memory::Exception;
use crate::board::Caller;
use crate::error::{ErrorCode, Result, WtError};
use crate::remove::forbidden;
use crate::request::RemoveInput;
use crate::service::Service;
use argus_core::ids::OVERLORD_PROJECT;
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

fn bad(message: impl Into<String>) -> WtError {
    WtError::new(ErrorCode::BadRequest, message)
}

impl Service {
    /// Only the Overlord writes the list; anyone else is told whom to ask.
    fn overlord_only(&self, input: &KeepInput) -> Result<String> {
        let by = input.caller.thread().unwrap_or_default().to_string();
        let from_portfolio = input.by_project.as_deref() == Some(OVERLORD_PROJECT);
        if from_portfolio && !by.is_empty() && self.overlord().as_deref() == Some(by.as_str()) {
            return Ok(by);
        }
        let who = if by.is_empty() {
            "the page"
        } else {
            by.as_str()
        };
        self.note(
            &input.project,
            "warn",
            &format!(
                "refused a worktree keep of {} from {who}: only the Overlord keeps",
                input.id
            ),
            input.caller.thread(),
        );
        Err(forbidden(format!(
            "only the Overlord puts a worktree on the exception list: ask him with coord lead portfolio \"keep worktree {} of {}: <why>\". Nothing was kept",
            input.id.trim(),
            input.project
        )))
    }

    /// Write or clear one exception.
    pub fn keep(&self, input: &KeepInput) -> Result<Kept> {
        let by = self.overlord_only(input)?;
        let lookup = RemoveInput::new(&input.project, &input.id, Caller::Page);
        let (_, fact) = self.listed_fact(&lookup)?;
        if input.clear {
            self.with_hygiene(&input.project, |m| m.exceptions.remove(&fact.id));
            self.note(
                &input.project,
                "info",
                &format!("{by} took {} off the exception list", fact.path),
                Some(&by),
            );
            return Ok(Kept {
                id: fact.id,
                text: format!(
                    "{} is off the exception list: the hygiene judges it again",
                    fact.path
                ),
            });
        }
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
        let now = self.now();
        if input.until_ms.is_some_and(|u| u <= now) {
            return Err(bad("--until must be in the future"));
        }
        let exception = Exception {
            path: fact.path.clone(),
            reason: reason.clone(),
            asked_by: input.asked_by.clone().filter(|a| !a.trim().is_empty()),
            by: by.clone(),
            at: now,
            until: input.until_ms,
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
        Ok(Kept {
            id: fact.id,
            text: format!(
                "{} is on the exception list: Argus never removes or reports it",
                fact.path
            ),
        })
    }
}
