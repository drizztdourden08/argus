//! Recording a result in the open run of a plan.

use serde_json::Value;

use super::given::Given;
use super::runs::{cases_of, progress};
use super::target::{Found, open_run};
use crate::Board;
use crate::error::BoardError;
use crate::json::{array_mut, display, doc, doc_mut, slot};
use crate::ops::result::OpResult;
use crate::ops::who::Who;
use crate::store::stamp;

fn push(board: &mut Board, run: &str, entry: Value) {
    if let Some(item) = doc_mut(board, "items", run) {
        array_mut(slot(item, "fields"), "results").push(entry);
    }
}

pub(super) fn record(
    board: &mut Board,
    (op, who): (&Value, &Who),
    found: &Found,
) -> Result<OpResult, BoardError> {
    let run = open_run(found)?;
    let (run_id, n) = (display(&run["id"]), run["n"].clone());
    let given = Given::of(board, who, op, &found.plan)?;
    let by = who.actor_of();
    given.on_test(board, &by, (&found.plan_id, &n))?;
    push(board, &run_id, given.entry(&by));
    let updated = doc(board, "items", &run_id).cloned().unwrap_or(Value::Null);
    let (done, total) = progress(&updated, &cases_of(&found.plan));
    stamp(board, "items", &run_id, &by)?;
    stamp(board, "items", &found.plan_id, &by)?;
    let said = format!(
        "{}: {} in run {} of {} ({done} of {total} cases)",
        given.test,
        given.status,
        display(&n),
        found.plan_id
    );
    Ok(OpResult::applied(said).with_extra("run", n))
}
