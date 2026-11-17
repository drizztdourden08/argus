//! The threads that hold a queued wake and have no listener: what a relay has to wake.

use std::collections::BTreeMap;

use serde::Serialize;

use super::watch::listener_of;
use super::{Env, Registry};
use crate::engine::ListenerInfo;
use crate::reasons::relays;
use crate::time::iso;

/// A queued wake is kept this long (and handed to the first listener).
const KEPT_MS: i64 = 24 * 3_600_000;

/// A live thread with queued wakes and no listener.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DueThread {
    /// Project key.
    pub project: String,
    /// The thread.
    pub thread: String,
    /// How many wakes wait.
    pub count: usize,
    /// When the oldest was made (ISO).
    pub oldest: String,
    /// The CLI session id the thread's last hook saw.
    pub session_id: Option<String>,
}

impl Registry {
    /// The threads with a queued wake and no listener, project by project.
    #[must_use]
    pub fn due(&self, env: &Env<'_>) -> Vec<DueThread> {
        let since = iso(self.now() - KEPT_MS);
        self.scopes()
            .iter()
            .flat_map(|scope| self.due_in(env, scope, &since))
            .collect()
    }

    fn due_in(&self, env: &Env<'_>, scope: &str, since: &str) -> Vec<DueThread> {
        let Some(board) = self.boards.board(scope) else {
            return Vec::new();
        };
        let listeners: Vec<ListenerInfo> = env
            .engine
            .listeners_of(scope)
            .into_iter()
            .filter(|l| l.v >= 2)
            .collect();
        let mut queued: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for row in env
            .engine
            .queued_since(scope, since)
            .into_iter()
            .filter(|row| relays(&row.reason))
        {
            queued.entry(row.thread).or_default().push(row.at);
        }
        queued
            .into_iter()
            .filter_map(|(id, times)| {
                let thread = board.threads.get(&id)?;
                if !thread.is_live() || listener_of(&id, Some(thread), &listeners).is_some() {
                    return None;
                }
                Some(DueThread {
                    project: scope.to_owned(),
                    thread: id,
                    count: times.len(),
                    oldest: times.iter().min()?.clone(),
                    session_id: thread.session_id.some().map(str::to_owned),
                })
            })
            .collect()
    }
}
