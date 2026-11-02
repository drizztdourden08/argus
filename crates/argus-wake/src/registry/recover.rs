//! A fault whose cause is gone clears by itself: the thread shows a sign of life after the fault opened.

use super::watch::{Open, Watch};
use super::{Key, Registry};
use crate::time::parse_ms;

/// The faults a later sign of life ends. A `not-listening` or `no-listener` fault does not end that way
/// (a deaf session still beats), only a thread that listens again ends it.
const LIFE_CLEARS: [&str; 2] = ["no-reaction", "delivery-error"];

impl Registry {
    /// Clear the thread's `no-reaction` or `delivery-error` fault once its heartbeat is newer than the
    /// fault. Returns true when it cleared.
    pub(super) fn check_life(&mut self, w: &Watch<'_>, open: &mut Open) -> bool {
        let key: Key = (w.scope.to_owned(), w.id.to_owned());
        let alive_since = open.get(&key).is_some_and(|fault| {
            LIFE_CLEARS.contains(&fault.reason.as_str())
                && parse_ms(w.thread.last_seen.as_str())
                    .zip(parse_ms(&fault.since))
                    .is_some_and(|(seen, since)| seen > since)
        });
        if alive_since {
            self.clear_by_registry(w, "the thread shows signs of life after the wake");
            open.remove(&key);
        }
        alive_since
    }
}
