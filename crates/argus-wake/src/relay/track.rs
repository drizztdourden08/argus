//! What was tried for one target, and what comes next.

use super::pick::pick;
use super::types::{Action, AttemptKind, LastAttempt, RelayLive, RelayTarget, ThreadKey};
use super::{AGAIN_AFTER_MS, FIRST_AFTER_MS, MAX_NOTES, NOTICE_AFTER_MS, QUIET_MS};

/// The next move for a target.
pub(super) enum Step {
    /// A note to this relay (the caller folds the notes of one relay together).
    Note(ThreadKey),
    /// Anything else.
    Do(Action),
}

/// The attempts made for one target.
#[derive(Debug, Default)]
pub(super) struct Attempt {
    notes: u32,
    used: Vec<ThreadKey>,
    last_note_at: i64,
    escalated_at: Option<i64>,
    notified_at: Option<i64>,
    pub(super) last: Option<LastAttempt>,
}

impl Attempt {
    pub(super) fn step(
        &mut self,
        now: i64,
        target: &RelayTarget,
        live: &[RelayLive],
        headless: bool,
    ) -> Option<Step> {
        if let Some(at) = self.notified_at {
            if now - at < QUIET_MS {
                return None;
            }
            *self = Self::default();
        }
        if let Some(at) = self.escalated_at {
            return self.notice(now, at, target);
        }
        let ready_at = if self.notes == 0 {
            target.since_ms + FIRST_AFTER_MS
        } else {
            self.last_note_at + AGAIN_AFTER_MS
        };
        if now < ready_at {
            return None;
        }
        let relay = (self.notes < MAX_NOTES)
            .then(|| pick(&target.ward.key, live, &self.used))
            .flatten();
        match relay {
            Some(via) => Some(self.note(now, via)),
            None => self.escalate(now, target, headless),
        }
    }

    fn note(&mut self, now: i64, via: ThreadKey) -> Step {
        self.notes += 1;
        self.last_note_at = now;
        if !self.used.contains(&via) {
            self.used.push(via.clone());
        }
        self.last = Some(LastAttempt {
            kind: AttemptKind::Note,
            at_ms: now,
            via: Some(via.thread.clone()),
        });
        Step::Note(via)
    }

    fn escalate(&mut self, now: i64, target: &RelayTarget, headless: bool) -> Option<Step> {
        self.escalated_at = Some(now);
        if !headless {
            return None;
        }
        self.last = Some(LastAttempt {
            kind: AttemptKind::Headless,
            at_ms: now,
            via: None,
        });
        Some(Step::Do(Action::Headless(target.ward.clone())))
    }

    fn notice(&mut self, now: i64, since: i64, target: &RelayTarget) -> Option<Step> {
        if now - since < NOTICE_AFTER_MS {
            return None;
        }
        self.notified_at = Some(now);
        self.last = Some(LastAttempt {
            kind: AttemptKind::Notified,
            at_ms: now,
            via: None,
        });
        Some(Step::Do(Action::Notice(target.ward.clone())))
    }
}
