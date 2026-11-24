//! The `kanban-config` op: the columns and rules of a board's Kanban tab. A change at the
//! coordinator's level (the owner and the board's lead): the board holds `kanban`, one audit line
//! records a change, and `null` clears it so the default runs again.

use serde_json::Value;

use super::{
    authorize::{Check, authorize},
    prelude::*,
};
use crate::{
    json::{set, unset},
    kanban_config::parse,
};

fn clear(board: &mut Board, who: &Who) -> OpResult {
    if board.get("kanban").is_none() {
        return OpResult::applied("the kanban config is unchanged");
    }
    unset(board, "kanban");
    audit(
        board,
        &who.actor_of(),
        &format!("{} cleared the kanban config", who.label()),
    );
    OpResult::applied("kanban config cleared")
}

/// Set or clear the board's Kanban config.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("hand", "threads", None))?;
    let given = &op["kanban"];
    if given.is_null() {
        return Ok(clear(board, who));
    }
    let config = parse(given).map_err(|e| refuse(e.to_string()))?;
    let next = serde_json::to_value(&config)?;
    if board.get("kanban") == Some(&next) {
        return Ok(OpResult::applied("the kanban config is unchanged"));
    }
    set(board, "kanban", next);
    audit(
        board,
        &who.actor_of(),
        &format!("{} set the kanban config", who.label()),
    );
    Ok(OpResult::applied("kanban config set"))
}
