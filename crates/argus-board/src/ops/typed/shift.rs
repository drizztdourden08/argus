//! An item changing stage: what a `status` or a `stage` argument means for a typed item.

use serde_json::{Map, Value, json};

use super::enter::{Entry, enter};
use super::force::reason;
use super::kind::{is_typed, type_of};
use super::outcome::Outcome;
use super::stage::{asked, current};
use crate::Board;
use crate::error::BoardError;
use crate::ops::Who;
use crate::registry::{ItemType, Stage};

/// The stage an item moves into, and what the move let through.
pub(crate) struct Shift {
    pub stage: &'static Stage,
    pub outcome: Outcome,
}

/// A change of an item: the item as it will be, as it is, and who changes it with which op.
pub(crate) struct Change<'a> {
    pub board: &'a Board,
    pub ty: &'static ItemType,
    pub item: &'a Value,
    pub before: &'a Value,
    pub who: &'a Who,
    pub op: &'a Value,
}

/// Check a move into the stage that `status` or `stage` names, `None` when they name none. The
/// `force` reason of the op overrides a rule that refuses.
pub(crate) fn shift(
    c: &Change,
    status: Option<&Value>,
    stage: Option<&Value>,
) -> Result<Option<Shift>, BoardError> {
    let Some(to) = asked(c.ty, c.item, status, stage)? else {
        return Ok(None);
    };
    let force = reason(c.op)?;
    let entry = Entry {
        board: c.board,
        item: c.item,
        ty: c.ty,
        from: current(c.ty, c.before),
        to,
        who: c.who,
        force: force.as_deref(),
    };
    let entered = enter(&entry)?;
    Ok(Some(Shift {
        stage: to,
        outcome: Outcome::of(entered, &to.id, force),
    }))
}

/// The status and stage a `move` op asks for on an item, put into `changes`. An item that names
/// neither a type nor a stage, and a move that names no stage, are left as they are.
pub(crate) fn on_move(
    board: &Board,
    item: &Value,
    (who, op): (&Who, &Value),
    changes: &mut Map<String, Value>,
    to: &Map<String, Value>,
) -> Result<Outcome, BoardError> {
    let stage = to.get("stage").filter(|s| !s.is_null());
    if !is_typed(item) && stage.is_none() {
        return Ok(Outcome::default());
    }
    let change = Change {
        board,
        ty: type_of(item)?,
        item,
        before: item,
        who,
        op,
    };
    let Some(found) = shift(&change, to.get("status"), stage)? else {
        return Ok(Outcome::default());
    };
    changes.insert("status".to_owned(), json!(found.stage.lane));
    changes.insert("stage".to_owned(), json!(found.stage.id));
    Ok(found.outcome)
}
