//! The runs of a test plan: `plan-run-start`, `plan-run-record` and `plan-run-end`. A run is an
//! item of its own (type `test-run`) that links its plan in `fields.plan` and keeps its build in
//! `fields.build` and its results in `fields.results`:
//! `[{ test, status, note, at, by, proofs }]`. Recording a result in a run also records it on the
//! test through the same result the test verb uses, so the test's own `result`, `runs` and the
//! `test-fail` wake stay true; a skip stays in the run only. A plan has one open run at a time.
//! Starting moves the run and the plan to `running`, ending moves the run to `reported` (to
//! `passed` when every case passed) and the plan to `reported`.

mod build;
mod done;
mod end;
mod fresh;
mod given;
mod place;
mod record;
mod runs;
mod start;
mod target;

pub(crate) use runs::{cases_of, is_ended, last_status, runs_of};

use serde_json::Value;

use super::authorize::{Check, authorize};
use super::result::OpResult;
use super::who::Who;
use crate::Board;
use crate::error::BoardError;

/// Whether an op kind is one of the plan run ops.
pub(crate) fn is_kind(kind: &str) -> bool {
    matches!(kind, "plan-run-start" | "plan-run-record" | "plan-run-end")
}

/// Apply a plan run op as `who`.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let found = target::of(board, op)?;
    authorize(who, &Check::new("edit", "items", Some(&found.plan)))?;
    match op["kind"].as_str() {
        Some("plan-run-start") => start::start(board, (op, who), &found),
        Some("plan-run-record") => record::record(board, (op, who), &found),
        _ => end::end(board, who, &found),
    }
}
