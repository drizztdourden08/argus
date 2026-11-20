//! A pending keep: a thread that calls `coord worktree keep` cannot write the exception list, but it can ask. Argus
//! records an exception marked `pending`, held for 24 hours exactly like a confirmed one, and tells the Overlord once
//! with the two answers. His keep confirms it, his clear declines it, and a lapse after 24 hours drops it.

use super::exception::{KeepInput, Kept, bad};
use super::memory::Exception;
use super::pending_text::{
    answer_note, ended_note, filed_reply, lapsed_note, overlord_note, overlord_wake,
};
use super::text::{APPROVE_REASON, REPORT_REASON};
use crate::board::Caller;
use crate::error::Result;
use crate::fact::Fact;
use crate::ports::WakeEvent;
use crate::remove::forbidden;
use crate::request::RemoveInput;
use crate::service::Service;

/// How long a pending keep holds a worktree while the Overlord has not answered.
pub(crate) const PENDING_KEEP_MS: i64 = 24 * 60 * 60 * 1000;

impl Service {
    /// What a caller that is not the Overlord does: a thread files or withdraws its pending keep, the page is refused.
    pub(super) fn thread_keep(&self, input: &KeepInput) -> Result<Kept> {
        let Some(thread) = input.caller.thread().map(ToString::to_string) else {
            self.note(
                &input.project,
                "warn",
                &format!("refused a worktree keep of {} from the page", input.id),
                None,
            );
            return Err(forbidden(format!(
                "only the Overlord puts a worktree on the exception list: the page cannot ask for a keep. Nothing was kept ({})",
                input.id.trim()
            )));
        };
        let lookup = RemoveInput::new(&input.project, &input.id, Caller::Page);
        let (_, fact) = self.listed_fact(&lookup)?;
        let prior = self.with_hygiene(&input.project, |m| m.exceptions.get(&fact.id).cloned());
        if input.clear {
            return self.withdraw(&input.project, &fact, &thread, prior.as_ref());
        }
        let reason = Self::reason_of(input)?;
        if prior.as_ref().is_some_and(|e| !e.pending) {
            return Ok(Kept {
                id: fact.id,
                text: format!("{} is already on the exception list", fact.path),
            });
        }
        self.file_pending(&input.project, &fact, &thread, &reason, prior.is_none());
        Ok(Kept {
            id: fact.id,
            text: filed_reply(&fact.path),
        })
    }

    /// Record the pending keep (a second ask refreshes it), and tell the Overlord the first time.
    fn file_pending(&self, project: &str, fact: &Fact, thread: &str, reason: &str, first: bool) {
        let now = self.now();
        let exception = Exception {
            path: fact.path.clone(),
            reason: reason.to_string(),
            asked_by: Some(thread.to_string()),
            by: thread.to_string(),
            at: now,
            until: Some(now + PENDING_KEEP_MS),
            pending: true,
            told: false,
        };
        self.with_hygiene(project, |m| {
            m.exceptions.insert(fact.id.clone(), exception);
        });
        self.note(
            project,
            "info",
            &format!("{thread} filed a pending keep of {}: {reason}", fact.path),
            Some(thread),
        );
        if first {
            let note = overlord_note(project, &fact.id, &fact.path, thread, reason);
            let wake = overlord_wake(project, &fact.id, thread);
            self.tell_overlord_keep(project, &note, wake);
        }
    }

    /// A thread's `--clear`: its own pending keep only, never a confirmed one.
    fn withdraw(
        &self,
        project: &str,
        fact: &Fact,
        thread: &str,
        prior: Option<&Exception>,
    ) -> Result<Kept> {
        let own = prior.is_some_and(|e| e.pending && e.asked_by.as_deref() == Some(thread));
        if !own {
            return Err(bad(format!(
                "{thread} has no pending keep of {}: only the Overlord clears a confirmed one",
                fact.path
            )));
        }
        self.with_hygiene(project, |m| m.exceptions.remove(&fact.id));
        self.note(
            project,
            "info",
            &format!("{thread} withdrew its pending keep of {}", fact.path),
            Some(thread),
        );
        Ok(Kept {
            id: fact.id.clone(),
            text: format!(
                "pending keep withdrawn: the hygiene judges {} again",
                fact.path
            ),
        })
    }

    /// The note on the Overlord's board, then the wake. Nothing when no Overlord is live.
    fn tell_overlord_keep(&self, project: &str, note: &str, wake: String) {
        let Some(overlord) = self.overlord() else {
            return;
        };
        let board = self.deps.store.overlord_board();
        let _ = self.deps.store.post_inbox(&board, &overlord, note);
        let _ = self.wake_one(
            project,
            WakeEvent {
                scope: board,
                thread: overlord.clone(),
                reason: REPORT_REASON,
                subject: overlord,
                text: wake,
            },
        );
    }

    /// Tell the asker of a pending keep that the Overlord confirmed or declined it.
    pub(super) fn tell_answer(&self, project: &str, prior: Option<&Exception>, confirmed: bool) {
        if let Some(e) = prior.filter(|e| e.pending) {
            self.tell_asker(project, e, &answer_note(&e.path, confirmed));
        }
    }

    /// Tell the askers of the keeps that ended: a pending one lapsed unanswered, a confirmed one ran out.
    pub(super) fn tell_lapsed(&self, project: &str, lapsed: &[Exception]) {
        for e in lapsed {
            let note = if e.pending {
                lapsed_note(&e.path)
            } else {
                ended_note(&e.path)
            };
            self.tell_asker(project, e, &note);
        }
    }

    /// One inbox note and a wake for the thread that asked.
    fn tell_asker(&self, project: &str, exception: &Exception, note: &str) {
        let Some(asker) = exception.asked_by.as_deref() else {
            return;
        };
        let _ = self.deps.store.post_inbox(project, asker, note);
        let _ = self.wake_one(
            project,
            WakeEvent {
                scope: project.to_string(),
                thread: asker.to_string(),
                reason: APPROVE_REASON,
                subject: asker.to_string(),
                text: note.chars().take(crate::status::MAX_TEXT).collect(),
            },
        );
    }
}
