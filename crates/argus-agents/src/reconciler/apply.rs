//! Writing the decisions of an evaluation onto the board under its lock.

use crate::board::Board;
use crate::holds::release;
use crate::record::{Patches, is_live, is_set, stamp};
use serde_json::Value;

const ACTOR: &str = "dashboard";

/// Applies every patch to its record and stamps it `dashboard`. A record that is flagged as an
/// orphan keeps the revision it was left at, so a later update by anyone else is seen.
pub(super) fn apply_patches(board: &mut Board, patches: &Patches, now_ms: i64) {
    for (id, patch) in patches.iter() {
        let Some(agent) = board.agent_mut(id) else {
            continue;
        };
        patch.apply_to(agent);
        stamp(agent, ACTOR, now_ms);
        if is_set(agent, "orphanSince") {
            let rev = agent.get("rev").cloned().unwrap_or(Value::Null);
            agent.insert("orphanRev".into(), rev);
        }
        if !is_live(agent) {
            release(board, id, now_ms);
        }
    }
}
