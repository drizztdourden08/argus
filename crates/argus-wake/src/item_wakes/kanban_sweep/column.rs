//! Which items a column holds, and since when an item has been in its lane.

use argus_board::kanban_config::Column;

use crate::board::{BoardView, ItemView};
use crate::time::ms_or_zero;

fn stage_of<'a>(board: &'a BoardView, item: &ItemView) -> &'a str {
    board.json()["items"][item.rec.id.as_str()]["stage"]
        .as_str()
        .unwrap_or("")
}

/// Does the column hold the item: its lane, and its stage when the column names stages?
pub(super) fn holds(column: &Column, board: &BoardView, item: &ItemView) -> bool {
    let in_lane = column.matches.lanes.iter().any(|lane| item.status.is(lane));
    let stages = &column.matches.stages;
    in_lane && (stages.is_empty() || stages.iter().any(|s| s == stage_of(board, item)))
}

/// When the item entered its lane: `laneAt`, then `startedAt`, then `updatedAt`; 0 when none is set.
pub(super) fn entered_at(board: &BoardView, item: &ItemView) -> i64 {
    let raw = &board.json()["items"][item.rec.id.as_str()];
    ["laneAt", "startedAt", "updatedAt"]
        .into_iter()
        .map(|key| ms_or_zero(raw[key].as_str()))
        .find(|ms| *ms > 0)
        .unwrap_or(0)
}
