//! The wake policy inside the engine, at the one place every wake is queued: a passive reason is
//! recorded as a notice and never opens a window, and a thread woken less than the debounce ago
//! has its next wakes held in one digest (see [`crate::wake_policy`]).

use tracing::warn;

use super::{Group, Key, WakeEngine};
use crate::wake::{FinishedEvent, WakeRow, WakeStatus};

/// Where [`WakeEngine::place`] put a row.
pub(super) enum Placed {
    /// It opened a window that closes on the usual timer.
    Opened,
    /// It joined a window that was open already.
    Joined,
    /// It is held by the debounce; `due` when the digest may go now.
    Held { due: bool },
}

impl WakeEngine {
    /// Record a passive row as a notice: closed at once, delivered to nobody, never replayed.
    pub(super) fn shelve(&self, row: &WakeRow) {
        let done = FinishedEvent {
            lines: vec![row.text.clone()],
            count: 1,
            status: WakeStatus::Passive,
            listeners: 0,
            delivered_at: None,
        };
        if let Err(err) = self.store.finish(row, &[], &done) {
            warn!(scope = %row.scope, thread = %row.thread, %err, "wake notice not recorded");
        }
    }

    /// Put a row in the window of its thread. A thread woken inside the debounce gets a held window:
    /// its deadline is the digest, or the end of the debounce for an immediate (owner) reason.
    pub(super) fn place(&mut self, key: &Key, row: WakeRow, immediate: bool) -> Placed {
        let now = self.clock.now_ms();
        let last = self.woken.get(key).copied();
        let held = self.config.policy.held_until(last, now, immediate);
        if let Some(group) = self.groups.iter_mut().find(|g| g.key == *key) {
            group.rows.push(row);
            if !group.held {
                return Placed::Joined;
            }
            if immediate {
                let free_at = held.unwrap_or(now);
                group.deadline = group.deadline.map(|d| d.min(free_at));
            }
            let due = group.deadline.is_some_and(|d| d <= now);
            return Placed::Held { due };
        }
        if let Some(deadline) = held {
            self.groups.push(Group {
                key: key.clone(),
                rows: vec![row],
                deadline: Some(deadline),
                held: true,
            });
            return Placed::Held { due: false };
        }
        let deadline = (self.config.window_ms > 0).then(|| now + self.config.window_ms);
        self.groups.push(Group {
            key: key.clone(),
            rows: vec![row],
            deadline,
            held: false,
        });
        Placed::Opened
    }

    /// The thread was just woken (or its event queued for a listener): the debounce starts now.
    pub(super) fn mark_woken(&mut self, scope: &str, thread: &str) {
        let now = self.clock.now_ms();
        self.woken
            .insert((scope.to_owned(), thread.to_owned()), now);
    }
}
