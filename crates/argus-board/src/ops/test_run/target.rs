//! What a run op points at: a plan and its open run, or a run and its plan.

use serde_json::Value;

use super::runs::{PLAN, RUN, is_open, runs_of};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc};
use crate::ops::target::target_of;

/// The plan of an op and the run it is about: the open run of a plan target, the run itself of a
/// run target, `None` when a plan has no open run.
pub(super) struct Found {
    pub plan_id: String,
    pub plan: Value,
    pub run: Option<Value>,
}

fn item(board: &Board, id: &str) -> Value {
    doc(board, "items", id).cloned().unwrap_or(Value::Null)
}

/// The plan and run an op points at.
pub(super) fn of(board: &Board, op: &Value) -> Result<Found, BoardError> {
    let (coll, id) = target_of(board, op, false)?;
    let target = item(board, &id);
    match (coll.as_str(), target["type"].as_str()) {
        ("items", Some(PLAN)) => {
            let run = runs_of(board, &id).into_iter().find(is_open);
            Ok(Found {
                plan_id: id,
                plan: target,
                run,
            })
        }
        ("items", Some(RUN)) => {
            let plan_id = display(&target["fields"]["plan"]);
            let plan = item(board, &plan_id);
            Ok(Found {
                plan_id,
                plan,
                run: Some(target),
            })
        }
        _ => Err(refuse(format!("{id} is not a test plan."))),
    }
}

/// The open run of the target, or the refusal that says how to start one.
pub(super) fn open_run(found: &Found) -> Result<&Value, BoardError> {
    found.run.as_ref().filter(|r| is_open(r)).ok_or_else(|| {
        refuse(format!(
            "{id} has no open run; start one with coord test plan start {id}.",
            id = found.plan_id
        ))
    })
}
