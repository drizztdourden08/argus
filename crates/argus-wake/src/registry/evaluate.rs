//! Reactions and timeouts of every pending wake (or of one project).

use super::faults::{Failure, secs};
use super::{Env, Key, Pending, Registry};

impl Registry {
    /// Settle every pending wake: reacted, gone, timed out. Then check the boards. Returns true when anything changed.
    pub(super) fn evaluate_scope(&mut self, env: &mut Env<'_>, only: Option<&str>) -> bool {
        let snapshot: Vec<(Key, Pending)> = self
            .pending
            .iter()
            .map(|(k, p)| (k.clone(), p.clone()))
            .collect();
        let mut changed = false;
        for ((scope, thread), pending) in snapshot {
            if only.is_some_and(|o| o != scope) {
                continue;
            }
            changed |= self.settle(env, &scope, &thread, &pending);
        }
        changed | self.check_boards(env, only)
    }

    fn settle(&mut self, env: &mut Env<'_>, scope: &str, thread: &str, pending: &Pending) -> bool {
        let key: Key = (scope.to_owned(), thread.to_owned());
        if !self.thread_of(scope, thread).is_some_and(|t| t.is_live()) {
            return self.boards.board(scope).is_some() && self.pending.shift_remove(&key).is_some();
        }
        if self.reacted(scope, thread, pending) {
            self.pending.shift_remove(&key);
            return true;
        }
        let now = self.now();
        match pending.delivered_at_ms {
            Some(delivered) if now - delivered > self.config.reaction_ms => {
                let detail = format!("delivered, no sign of life for {}", secs(now - delivered));
                let failure = Failure {
                    reason: "no-reaction",
                    wake_id: Some(&pending.id),
                    detail,
                    text: None,
                };
                self.raise(env, scope, thread, &failure);
                true
            }
            None if now - pending.since_ms > self.config.no_listener_ms => {
                let detail = format!(
                    "queued for {}, nobody listening",
                    secs(now - pending.since_ms)
                );
                let failure = Failure {
                    reason: "no-listener",
                    wake_id: Some(&pending.id),
                    detail,
                    text: None,
                };
                self.raise(env, scope, thread, &failure);
                true
            }
            _ => false,
        }
    }
}
