//! The wakes that wait for a reaction, and how a reaction is seen.

use super::{Env, Key, Registry};
use crate::board::ThreadView;
use crate::time::{iso, parse_ms};
use crate::wake::WakeRow;

/// A wake the registry waits on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    /// The event (first wake) id.
    pub id: String,
    /// When it was made (ISO).
    pub at: String,
    /// Why it was made.
    pub reason: Option<String>,
    /// Its text.
    pub text: Option<String>,
    /// When the registry started waiting (epoch ms).
    pub since_ms: i64,
    /// When a listener took it (epoch ms); `None` while it is queued.
    pub delivered_at_ms: Option<i64>,
    /// The inbox messages that were waiting when it was sent.
    pub inbox: Vec<String>,
}

impl Pending {
    pub(super) fn rebuilt(row: &WakeRow, started_at: i64, inbox: Vec<String>) -> Self {
        Self {
            id: row.id.clone(),
            at: row.at.clone(),
            reason: Some(row.reason.clone()),
            text: Some(row.text.clone()),
            since_ms: started_at,
            delivered_at_ms: None,
            inbox,
        }
    }
}

/// An event the engine reported.
pub(super) struct TrackedEvent {
    pub id: String,
    pub at: String,
    pub reason: Option<String>,
    pub text: Option<String>,
}

impl Registry {
    pub(super) fn thread_of(&self, scope: &str, thread: &str) -> Option<ThreadView> {
        self.boards.board(scope)?.threads.get(thread).cloned()
    }

    pub(super) fn inbox_ids(&self, scope: &str, thread: &str) -> Vec<String> {
        self.boards
            .board(scope)
            .map(|b| b.inbox_ids(thread))
            .unwrap_or_default()
    }

    pub(super) fn track(
        &mut self,
        env: &mut Env<'_>,
        scope: &str,
        thread: &str,
        event: TrackedEvent,
        delivered: bool,
    ) {
        let key: Key = (scope.to_owned(), thread.to_owned());
        if let Some(open) = self.open_faults().remove(&key) {
            if !delivered {
                self.realert(env, &key, &open);
            }
            return;
        }
        let now = self.now();
        if !self.pending.contains_key(&key) {
            let pending = Pending {
                id: event.id,
                at: if event.at.is_empty() {
                    iso(now)
                } else {
                    event.at
                },
                reason: event.reason,
                text: event.text,
                since_ms: now,
                delivered_at_ms: None,
                inbox: self.inbox_ids(scope, thread),
            };
            self.pending.insert(key.clone(), pending);
        }
        if let Some(pending) = self.pending.get_mut(&key) {
            if delivered && pending.delivered_at_ms.is_none() {
                pending.delivered_at_ms = Some(now);
            }
        }
    }

    pub(super) fn reacted(&self, scope: &str, thread: &str, pending: &Pending) -> bool {
        let seen = self
            .thread_of(scope, thread)
            .and_then(|t| parse_ms(t.last_seen.as_str()));
        if seen.is_some_and(|seen| seen > pending.delivered_at_ms.unwrap_or(pending.since_ms)) {
            return true;
        }
        if pending.inbox.is_empty() {
            return false;
        }
        let now = self.inbox_ids(scope, thread);
        pending.inbox.iter().all(|id| !now.contains(id))
    }
}
