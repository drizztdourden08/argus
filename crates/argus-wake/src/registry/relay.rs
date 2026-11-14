//! What the wake relay reads from the registry: the due threads as targets, and the threads that can carry a note.

use super::watch::listener_of;
use super::{DueThread, Env, Registry};
use crate::engine::ListenerInfo;
use crate::relay::{RelayLive, RelayTarget, ThreadKey, Ward};
use crate::time::parse_ms;

/// A due thread as the relay sees it.
#[must_use]
pub fn target_of(due: &DueThread) -> RelayTarget {
    RelayTarget {
        ward: Ward {
            key: ThreadKey::new(&due.project, &due.thread),
            session_id: due.session_id.clone(),
        },
        since_ms: parse_ms(&due.oldest).unwrap_or(0),
    }
}

impl Registry {
    /// The live threads with a v2 listener: the ones a relay note can reach.
    #[must_use]
    pub fn relay_live(&self, env: &Env<'_>) -> Vec<RelayLive> {
        let mut out = Vec::new();
        for scope in self.scopes() {
            let Some(board) = self.boards.board(&scope) else {
                continue;
            };
            let listeners: Vec<ListenerInfo> = env
                .engine
                .listeners_of(&scope)
                .into_iter()
                .filter(|l| l.v >= 2)
                .collect();
            for (id, thread) in &board.threads {
                if thread.is_live() && listener_of(id, Some(thread), &listeners).is_some() {
                    out.push(RelayLive {
                        key: ThreadKey::new(&scope, id),
                        in_turn: self.in_turn(thread),
                        main: self.is_main(&scope, id),
                    });
                }
            }
        }
        out
    }
}
