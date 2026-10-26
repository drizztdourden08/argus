//! The log lines a service writes that also reach the activity log: one line per message within
//! a minute, so a tick that fails every five seconds is one line a minute, not twelve.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

const DEDUP_MS: i64 = 60_000;
const PRUNE_ABOVE: usize = 500;

/// When each message was last written.
#[derive(Default)]
pub(crate) struct Notes {
    seen: Mutex<HashMap<String, i64>>,
}

impl Notes {
    /// Whether the message may be written at `now`; it then counts as written.
    pub(crate) fn admit(&self, kind: &str, message: &str, now: i64) -> bool {
        let mut seen = self.seen.lock().unwrap_or_else(PoisonError::into_inner);
        let key = format!("{kind}\u{0}{message}");
        if seen.get(&key).is_some_and(|at| now - at < DEDUP_MS) {
            return false;
        }
        seen.insert(key, now);
        if seen.len() > PRUNE_ABOVE {
            seen.retain(|_, at| now - *at < DEDUP_MS);
        }
        true
    }
}
