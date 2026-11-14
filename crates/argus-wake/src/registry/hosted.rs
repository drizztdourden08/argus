//! The hosted Overlord's thread: it has no listener process and no board tab, so the board-closed and
//! not-listening faults do not apply. Its listener is the sink the server attaches while the child
//! runs. A thread that was alive a moment ago and has no sink for the board grace means the child is dead or crashed:
//! `hosted-down`, which the owner hears at once (the Overlord cannot resolve its own outage).

use super::faults::secs;
use super::watch::{Open, Watch};
use super::{Env, Key, Registry};
use crate::reminders::HOSTED_DOWN;

impl Registry {
    /// Open `hosted-down` for a live hosted thread with no sink past the grace, and clear it when the
    /// sink is back. Returns true when anything changed.
    pub(super) fn check_hosted(
        &mut self,
        env: &mut Env<'_>,
        w: &Watch<'_>,
        open: &mut Open,
    ) -> bool {
        let key: Key = (w.scope.to_owned(), w.id.to_owned());
        let down = open.get(&key).is_some_and(|f| f.reason == HOSTED_DOWN);
        if w.listener.is_some() || !w.thread.is_live() {
            self.deaf.remove(&key);
            if down {
                self.clear_by_registry(w, "the hosted child is up again");
                open.remove(&key);
            }
            return down;
        }
        let now = self.now();
        let since = match self.deaf.get(&key) {
            Some(since) => *since,
            None if self.heartbeat(w.thread) => *self.deaf.entry(key.clone()).or_insert(now),
            None => return false,
        };
        if !(w.warm && !open.contains_key(&key) && now - since > self.config.board_grace_ms) {
            return false;
        }
        let detail = format!("no hosted child for {}", secs(now - since));
        let text = format!(
            "{}/{}: the hosted Overlord is down, {detail}. Start it again from the Overlord tab; wakes stay pending until it is up.",
            w.scope, w.id
        );
        if let Some(fault) =
            self.raise_noticed(env, w.scope, w.id, HOSTED_DOWN, (detail, text))
        {
            open.insert(key, fault);
        }
        true
    }
}
