//! Moving an item into a stage of its type; the stage's lane becomes the item's status.

use super::runs::{PLAN, RUN};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc_mut, set};
use crate::registry::registry;

/// Move a plan to a stage.
pub(super) fn plan_to(board: &mut Board, id: &str, stage: &str) -> Result<(), BoardError> {
    move_to(board, (PLAN, id), stage)
}

/// Move a run to a stage.
pub(super) fn run_to(board: &mut Board, id: &str, stage: &str) -> Result<(), BoardError> {
    move_to(board, (RUN, id), stage)
}

fn move_to(board: &mut Board, (ty, id): (&str, &str), stage: &str) -> Result<(), BoardError> {
    let found = registry().item_type(ty).and_then(|t| t.stage(stage));
    let lane = found.map(|s| s.lane.clone());
    let (Some(lane), Some(item)) = (lane, doc_mut(board, "items", id)) else {
        return Err(refuse(format!("{id} cannot move to {stage}.")));
    };
    set(item, "stage", stage);
    set(item, "status", lane);
    Ok(())
}
