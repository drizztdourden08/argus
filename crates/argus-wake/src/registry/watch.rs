//! Watching the live threads: an active session nobody can wake, and a thread inside a turn with no board open.

use std::collections::HashMap;

use super::faults::{Failure, secs};
use super::{Env, Key, Registry};
use crate::board::ThreadView;
use crate::engine::ListenerInfo;
use crate::faults::{ClearFault, Fault};
use crate::time::{iso, parse_ms};

/// A turn this recent counts as running.
const BUSY_MS: i64 = 3 * 60 * 1000;
/// The faults that end by themselves once the thread listens again. A `no-reaction` fault does not:
/// its wake was delivered to a listener and the thread stayed silent.
const LISTEN_CLEARS: [&str; 2] = ["not-listening", "no-listener"];

type Open = HashMap<Key, Fault>;

/// The listener that covers a thread: its own, or the `*` one for the project's coordinator.
pub(super) fn listener_of<'a>(
    id: &str,
    thread: Option<&ThreadView>,
    listeners: &'a [ListenerInfo],
) -> Option<&'a ListenerInfo> {
    let own = listeners.iter().find(|l| l.thread == id);
    let lead = thread
        .is_some_and(|t| t.role.is("coordinator"))
        .then(|| listeners.iter().find(|l| l.thread == "*"));
    own.or(lead.flatten())
}

/// What one pass over a thread needs.
struct Watch<'a> {
    scope: &'a str,
    id: &'a str,
    thread: &'a ThreadView,
    listener: Option<&'a ListenerInfo>,
    warm: bool,
}

impl Registry {
    pub(super) fn in_turn(&self, thread: &ThreadView) -> bool {
        let seen = parse_ms(thread.last_seen.as_str()).unwrap_or(0);
        thread.activity.is("working") && self.now() - seen <= BUSY_MS
    }

    pub(super) fn heartbeat(&self, thread: &ThreadView) -> bool {
        parse_ms(thread.last_seen.as_str())
            .is_some_and(|seen| self.now() - seen <= self.config.alive_ms)
    }

    /// A board must be open only while the thread is inside a turn: the Browser pane of a session
    /// closes by itself between turns, so an idle thread is reached through its listener, not its page.
    pub(super) fn board_required(&self, env: &Env<'_>, thread: Option<&ThreadView>) -> bool {
        thread.is_some_and(|t| env.presence.is_some() && t.is_live() && self.in_turn(t))
    }

    /// Open the not-listening and board-closed faults and clear them when they are over.
    /// A retired project that is not watched is skipped. Returns true when anything changed.
    pub(super) fn check_boards(&mut self, env: &mut Env<'_>, only: Option<&str>) -> bool {
        if only.is_some_and(|o| !self.scopes().iter().any(|s| s == o)) {
            return false;
        }
        let warm = self.now() - self.started_at > self.config.restart_grace_ms;
        let mut open = self.open_faults();
        let scopes = only.map_or_else(|| self.scopes(), |o| vec![o.to_owned()]);
        let mut changed = false;
        for scope in &scopes {
            changed |= self.check_scope(env, scope, &mut open, warm);
        }
        changed
    }

    fn check_scope(&mut self, env: &mut Env<'_>, scope: &str, open: &mut Open, warm: bool) -> bool {
        let Some(board) = self.boards.board(scope) else {
            return false;
        };
        let listeners: Vec<ListenerInfo> = env
            .engine
            .listeners_of(scope)
            .into_iter()
            .filter(|l| l.v >= 2)
            .collect();
        let mut changed = false;
        for (id, thread) in &board.threads {
            let listener = listener_of(id, Some(thread), &listeners);
            let watch = Watch {
                scope,
                id,
                thread,
                listener,
                warm,
            };
            changed |= self.check_listening(env, &watch, open);
            changed |= self.check_board_open(env, &watch, open);
        }
        changed
    }

    fn check_listening(&mut self, env: &mut Env<'_>, w: &Watch<'_>, open: &mut Open) -> bool {
        let key: Key = (w.scope.to_owned(), w.id.to_owned());
        if w.listener.is_some() {
            self.deaf.remove(&key);
            let listening_again = open
                .get(&key)
                .is_some_and(|f| LISTEN_CLEARS.contains(&f.reason.as_str()));
            if listening_again {
                self.clear_by_registry(w, "it listens again");
                open.remove(&key);
            }
            return listening_again;
        }
        if !(w.thread.is_live() && self.heartbeat(w.thread) && !self.in_turn(w.thread)) {
            self.deaf.remove(&key);
            return false;
        }
        let now = self.now();
        let since = *self.deaf.entry(key.clone()).or_insert(now);
        if !(w.warm && !open.contains_key(&key) && now - since > self.config.deaf_grace_ms) {
            return false;
        }
        let detail = format!(
            "no wake-stream for {} while the session is active",
            secs(now - since)
        );
        let text = format!(
            "{}/{}: not listening, {detail}. Resolve it with them now (coord lead): its wake listener (the background hook, started at the end of each turn) is not running, so one message that makes it run a turn starts it; it clears itself when it listens.",
            w.scope, w.id
        );
        let failure = Failure {
            reason: "not-listening",
            wake_id: None,
            detail,
            text: Some(text),
        };
        if let Some(fault) = self.raise(env, w.scope, w.id, &failure) {
            open.insert(key, fault);
        }
        true
    }

    fn check_board_open(&mut self, env: &mut Env<'_>, w: &Watch<'_>, open: &Open) -> bool {
        let Some(presence) = env.presence else {
            return false;
        };
        let key: Key = (w.scope.to_owned(), w.id.to_owned());
        let current = open.get(&key);
        if presence.open_of(w.scope, w.id).is_some() {
            self.missing.remove(&key);
            let reopened = current.is_some_and(|f| f.reason == "board-closed");
            if reopened {
                self.clear_by_registry(w, "the board is open again");
            }
            return reopened;
        }
        if !self.board_required(env, Some(w.thread)) {
            self.missing.remove(&key);
            let stale = current.is_some_and(|f| f.reason == "board-closed");
            if stale {
                self.clear_by_registry(w, "the thread is idle, its board is not required");
            }
            return stale;
        }
        let now = self.now();
        let since = *self.missing.entry(key).or_insert(now);
        if !(w.warm && current.is_none() && now - since > self.config.board_grace_ms) {
            return false;
        }
        let detail = format!("no open board tab for {}", secs(now - since));
        let text = format!(
            "{}/{}: board closed, {detail}. Resolve it with them now (coord lead); it clears itself when the board opens.",
            w.scope, w.id
        );
        let failure = Failure {
            reason: "board-closed",
            wake_id: None,
            detail,
            text: Some(text),
        };
        self.raise(env, w.scope, w.id, &failure);
        true
    }

    fn clear_by_registry(&mut self, w: &Watch<'_>, note: &str) {
        let request = ClearFault {
            scope: w.scope.to_owned(),
            thread: w.id.to_owned(),
            at: iso(self.now()),
            by: "registry".to_owned(),
            note: note.to_owned(),
        };
        if let Err(err) = self.fault_store.clear(&request) {
            tracing::warn!(scope = w.scope, thread = w.id, %err, "wake fault not cleared");
        }
    }
}
