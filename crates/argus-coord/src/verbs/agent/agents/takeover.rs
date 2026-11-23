//! `coord agent set <A-n> --claude-id <id>` where the hook already made a record of its own for the
//! same sub-agent (the lead had started it by hand first): the hook's marks and transcript move
//! onto the lead's record and the hook's record is closed as the duplicate.

use argus_board::Board;
use argus_board::agents::is_live;
use argus_board::holds::transfer;
use argus_board::json::{doc, set};
use argus_board::store::stamp;
use argus_core::clock::now_iso;
use serde_json::Value;

use super::fields::Fields;
use super::find::agent_mut;
use crate::cli::exit::CoordError;

/// What the target takes from the twin when it has none of its own.
const MOVED: [&str; 5] = [
    "hookId",
    "toolUseId",
    "agentType",
    "transcriptFile",
    "modelUsed",
];
/// What the twin gives up for good.
const CLEARED: [&str; 3] = ["hookId", "claudeId", "transcriptFile"];

/// Adds to `fields` what the target lacks and the twin holds.
fn take_into(fields: &mut Fields, twin: &Value, target: &Value) {
    for key in MOVED {
        if !twin[key].is_null() && target[key].is_null() {
            fields.insert(key.to_owned(), twin[key].clone());
        }
    }
}

/// Moves what the target lacks from the twin into `fields`, then empties the twin's ids, marks it
/// `duplicateOf` the target and stops it when it was live. The items the twin held go to the target.
///
/// # Errors
/// No such twin or target record.
pub fn absorb(
    board: &mut Board,
    twin_id: &str,
    target_id: &str,
    fields: &mut Fields,
    actor: &str,
) -> Result<(), CoordError> {
    let target = doc(board, "agents", target_id)
        .cloned()
        .unwrap_or(Value::Null);
    let twin = agent_mut(board, twin_id)?;
    take_into(fields, twin, &target);
    let live = is_live(twin);
    for key in CLEARED {
        set(twin, key, Value::Null);
    }
    set(twin, "duplicateOf", target_id);
    if live {
        set(twin, "status", "stopped");
        set(twin, "finishedAt", now_iso());
    }
    stamp(board, "agents", twin_id, actor)?;
    transfer(board, twin_id, target_id, actor)?;
    Ok(())
}
