//! The `title-max` op: the longest item title a board accepts. A change at the coordinator's
//! level: the board holds `titleMax`, a whole number from [`MIN`] to [`MAX`], and one audit line
//! records it. `null` clears it so the default limit runs again.

use serde_json::Value;

use super::{
    announce::audit,
    authorize::{Check, authorize},
    result::OpResult,
    who::Who,
};
use crate::{
    Board,
    error::{BoardError, refuse},
    json::{set, unset},
    title_limit::{MAX, MIN, of},
};

/// The checked limit: `Ok(None)` clears it.
fn check(value: &Value) -> Result<Option<u64>, BoardError> {
    if value.is_null() {
        return Ok(None);
    }
    match value.as_u64() {
        Some(n) if (MIN..=MAX).contains(&n) => Ok(Some(n)),
        _ => Err(refuse(format!(
            "titleMax must be a whole number from {MIN} to {MAX}."
        ))),
    }
}

/// Set or clear the board's item title limit.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("hand", "threads", None))?;
    let next = check(&op["titleMax"])?;
    let was = of(board).and_then(|n| u64::try_from(n).ok());
    if next == was {
        return Ok(OpResult::applied("the title limit is unchanged"));
    }
    let what = match next {
        Some(n) => {
            set(board, "titleMax", n);
            format!("set the item title limit to {n}")
        }
        None => {
            unset(board, "titleMax");
            "cleared the item title limit".to_owned()
        }
    };
    audit(board, &who.actor_of(), &format!("{} {what}", who.label()));
    Ok(OpResult::applied(what))
}
