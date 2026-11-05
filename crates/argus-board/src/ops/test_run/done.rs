//! The end of every run op: stamp the plan and the run, tell the plan's thread, say how it went.

use serde_json::{Value, json};

use crate::Board;
use crate::error::BoardError;
use crate::json::doc;
use crate::ops::announce::announce;
use crate::ops::result::OpResult;
use crate::ops::who::Who;
use crate::store::stamp;

/// Stamp the plan and the run, tell the plan's thread when someone else changed it, and say how
/// it went.
pub(super) fn finish(
    board: &mut Board,
    who: &Who,
    (plan, run): (&str, &str),
    said: String,
) -> Result<OpResult, BoardError> {
    stamp(board, "items", run, &who.actor_of())?;
    stamp(board, "items", plan, &who.actor_of())?;
    let n = doc(board, "items", run).map_or(Value::Null, |r| r["n"].clone());
    let owner = doc(board, "items", plan).and_then(|p| p["thread"].as_str().map(str::to_owned));
    let notify = announce(board, who, &[owner], &said, Some(plan));
    Ok(OpResult::applied(said)
        .with_extra("run", n)
        .with_extra("runItem", json!(run))
        .with_notify(notify))
}
