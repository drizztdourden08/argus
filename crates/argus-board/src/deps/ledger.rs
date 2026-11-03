//! The ledger: collection `deps` on the portfolio board. Reading it, changing it under the
//! lock, and the one function that moves a dependency from a state to another.

use std::path::Path;

use argus_core::clock::now_iso;
use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use super::error::DepError;
use super::model::{Dep, DepState, Transition};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{array_mut, doc, doc_mut, section, set};
use crate::store::{read_board, stamp, with_board};

/// The ledger collection.
pub(crate) const COLL: &str = "deps";
/// History rows kept per dependency.
const HISTORY_KEEP: usize = 20;

/// The portfolio board, or an empty ledger when there is none yet.
pub(crate) fn read(home: &Path) -> Result<Board, DepError> {
    let board = read_board(home, PORTFOLIO)?;
    Ok(board.unwrap_or_else(|| json!({ "project": PORTFOLIO, "deps": {} })))
}

/// Every dependency of a ledger board, oldest first.
pub(crate) fn deps_in(board: &Board) -> Vec<Dep> {
    section(board, COLL)
        .values()
        .filter_map(Dep::from_value)
        .collect()
}

/// One dependency of a ledger board.
pub(crate) fn dep_in(board: &Board, id: &str) -> Option<Dep> {
    doc(board, COLL, id).and_then(Dep::from_value)
}

/// The dependency `id`, or a 404.
pub(crate) fn get(home: &Path, id: &str) -> Result<Dep, DepError> {
    dep_in(&read(home)?, id).ok_or_else(|| unknown(id))
}

pub(crate) fn unknown(id: &str) -> DepError {
    DepError::Unknown(format!("no dependency {id}"))
}

/// Run `f` under the lock of one board and save it. A refusal of `f` that is not a board error
/// comes back as it is, and nothing is saved.
pub(crate) fn locked<T>(
    home: &Path,
    key: &str,
    f: impl FnOnce(&mut Board) -> Result<T, DepError>,
) -> Result<T, DepError> {
    let mut refused: Option<DepError> = None;
    let out = with_board(home, key, false, |board| {
        f(board).map_err(|e| match e {
            DepError::Board(inner) => inner,
            other => {
                refused = Some(other);
                refuse("refused")
            }
        })
    });
    match refused {
        Some(e) => Err(e),
        None => out.map_err(DepError::from),
    }
}

/// Run `f` under the lock of the portfolio board.
pub(crate) fn locked_ledger<T>(
    home: &Path,
    f: impl FnOnce(&mut Board) -> Result<T, DepError>,
) -> Result<T, DepError> {
    locked(home, PORTFOLIO, f)
}

/// The next history number of a dependency doc: one past the last row, 1 for none.
fn next_seq(d: &Value) -> i64 {
    d["history"]
        .as_array()
        .and_then(|rows| rows.last())
        .and_then(|row| row["seq"].as_i64())
        .map_or(1, |last| last + 1)
}

/// The change of state asked of [`move_state`].
pub(crate) struct Step<'a> {
    pub to: DepState,
    pub by: &'a str,
    pub reason: Option<&'a str>,
}

fn keep_dates(d: &mut Value, to: DepState, at: &str) {
    if to == DepState::Open && d["openedAt"].is_null() {
        set(d, "openedAt", at);
    }
    let ended = matches!(
        to,
        DepState::Done | DepState::Dropped | DepState::Gone | DepState::Cancelled
    );
    set(d, "resolvedAt", if ended { json!(at) } else { Value::Null });
}

fn push_row(d: &mut Value, row: Value) {
    let rows = array_mut(d, "history");
    rows.push(row);
    if rows.len() > HISTORY_KEEP {
        let extra = rows.len() - HISTORY_KEEP;
        rows.drain(..extra);
    }
}

/// Move a dependency to a state: one history row with the next number (the last 20 kept), the
/// reason, the dates, and one more revision.
pub(crate) fn move_state(board: &mut Board, id: &str, step: &Step) -> Result<Transition, DepError> {
    let at = now_iso();
    let Some(d) = doc_mut(board, COLL, id) else {
        return Err(unknown(id));
    };
    let from = serde_json::from_value(d["state"].clone()).unwrap_or(DepState::Opening);
    let seq = next_seq(d);
    push_row(
        d,
        json!({
            "seq": seq, "at": at, "by": step.by, "from": from, "to": step.to, "reason": step.reason,
        }),
    );
    set(d, "state", step.to.as_str());
    set(d, "reason", step.reason);
    keep_dates(d, step.to, &at);
    stamp(board, COLL, id, step.by)?;
    Ok(Transition {
        dep: id.to_owned(),
        seq,
        from,
        to: step.to,
    })
}

/// Which end of a dependency a field belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    Requester,
    Assignee,
}

impl Side {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Side::Requester => "from",
            Side::Assignee => "to",
        }
    }
}

/// Set the item or the thread of one end, when it differs.
pub(crate) fn set_end(
    board: &mut Board,
    id: &str,
    (side, field, value): (Side, &str, &str),
    by: &str,
) -> Result<bool, BoardError> {
    let Some(end) = doc_mut(board, COLL, id).map(|d| &mut d[side.key()]) else {
        return Err(refuse(format!("no deps doc \"{id}\".")));
    };
    if end[field].as_str() == Some(value) {
        return Ok(false);
    }
    set(end, field, value);
    stamp(board, COLL, id, by)?;
    Ok(true)
}
