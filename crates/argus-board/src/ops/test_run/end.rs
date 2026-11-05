//! Ending the open run of a plan: count the results, report the run and the plan.

use argus_core::clock::now_iso;
use serde_json::Value;

use super::done::finish;
use super::place::{plan_to, run_to};
use super::runs::{cases_of, summary_of};
use super::target::{Found, open_run};
use crate::Board;
use crate::error::BoardError;
use crate::json::{display, doc_mut, map_mut, set};
use crate::ops::result::OpResult;
use crate::ops::who::Who;

fn counts_line(summary: &Value) -> String {
    format!(
        "{} pass, {} fail, {} partial, {} blocked, {} skip, {} not run",
        display(&summary["pass"]),
        display(&summary["fail"]),
        display(&summary["partial"]),
        display(&summary["blocked"]),
        display(&summary["skip"]),
        display(&summary["notRun"]),
    )
}

/// Whether every case of the plan passed.
fn all_passed(summary: &Value) -> bool {
    summary["total"].as_u64() > Some(0) && summary["pass"] == summary["total"]
}

pub(super) fn end(board: &mut Board, who: &Who, found: &Found) -> Result<OpResult, BoardError> {
    let run = open_run(found)?;
    let run_id = display(&run["id"]);
    let n = display(&run["n"]);
    let summary = summary_of(run, &cases_of(&found.plan));
    let line = counts_line(&summary);
    if let Some(item) = doc_mut(board, "items", &run_id) {
        set(item, "endedAt", now_iso());
        map_mut(item, "fields").insert("summary".to_owned(), Value::String(line.clone()));
    }
    run_to(
        board,
        &run_id,
        if all_passed(&summary) {
            "passed"
        } else {
            "reported"
        },
    )?;
    plan_to(board, &found.plan_id, "reported")?;
    let said = format!("{} run {n} ended: {line}", found.plan_id);
    finish(board, who, (&found.plan_id, &run_id), said).map(|r| r.with_extra("summary", summary))
}
