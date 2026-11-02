//! What one stall tick reads from the wake service.

use tracing::warn;

use crate::source::BoardSource;
use crate::store::{RowsQuery, WakeStore};
use crate::wake::WakeRow;

const ROWS_PER_SCOPE: usize = 200;

/// The boards, the wakes table and the Overlord's thread.
pub struct StallEnv<'a> {
    /// The boards of every project.
    pub boards: &'a dyn BoardSource,
    /// The wakes table.
    pub store: &'a dyn WakeStore,
    /// The thread of the Overlord, who hears the stalls.
    pub main: &'a str,
}

impl StallEnv<'_> {
    /// When the newest wake of a reason went to a thread about a subject (epoch ms); 0 when never, or when the
    /// table cannot be read.
    pub(super) fn last_sent(&self, scope: &str, thread: &str, reason: &str, subject: &str) -> i64 {
        self.store
            .last_at(scope, thread, reason, Some(subject))
            .unwrap_or_else(|err| {
                warn!(%err, "stall watch: last wake not read");
                0
            })
    }

    /// The newest wake rows of a project, newest first.
    pub(super) fn rows(&self, scope: &str) -> Vec<WakeRow> {
        let query = RowsQuery {
            limit: ROWS_PER_SCOPE,
            thread: None,
        };
        self.store.rows(scope, &query).unwrap_or_else(|err| {
            warn!(%err, "stall watch: wake rows not read");
            Vec::new()
        })
    }
}
