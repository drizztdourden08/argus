//! The build a run is made for, as an op names it.

use serde_json::Value;

use crate::Board;
use crate::content::{Context, check as content_check};
use crate::error::BoardError;
use crate::json::ids;

/// The build an op names, checked against the launchers of the board; `None` when it names none.
pub(super) fn named(board: &Board, op: &Value) -> Result<Option<Value>, BoardError> {
    let Some(build) = op.get("build").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let known = ids(board, "launchers");
    let cx = Context {
        ids: Some(known.as_slice()),
        ..Context::default()
    };
    content_check("build", build, &cx).map(Some)
}
