//! Choosing the thread that carries a relay note.

use super::types::{RelayLive, ThreadKey};

fn rank(live: &RelayLive) -> u8 {
    match (live.in_turn, live.main) {
        (false, true) => 0,
        (false, false) => 1,
        (true, _) => 2,
    }
}

/// The best relay for `target`: the Overlord when idle, then any idle thread, then a thread in a turn. Never the
/// target itself, and a relay not yet `used` for it comes before one that was.
#[must_use]
pub fn pick(target: &ThreadKey, live: &[RelayLive], used: &[ThreadKey]) -> Option<ThreadKey> {
    let mut candidates: Vec<&RelayLive> = live.iter().filter(|l| &l.key != target).collect();
    candidates.sort_by_key(|l| (rank(l), l.key.clone()));
    candidates
        .iter()
        .find(|l| !used.contains(&l.key))
        .or(candidates.first())
        .map(|l| l.key.clone())
}
