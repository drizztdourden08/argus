//! `coord test plan`: test plans and their runs. A plan is an item of type `test-plan` whose
//! `cases` are tests (`T-n`); a run records one result per case, and each result is also written
//! to the test itself.
//!
//! ```text
//! coord test plan add "<title>" [--parent I-n] [--bundle tag] [--case T-n]...
//! coord test plan cases I-n --add T-n [--before T-m] | --remove T-n
//! coord test plan start I-n [--bundle tag]     coord test plan end I-n     coord test plan show I-n
//! coord test result T-n <status> "<note>" --run I-n
//! ```

mod add;
mod cases;
mod run;
mod show;

use argus_board::Board;
use argus_board::json::display;
use argus_board::ops::{OpResult, Status, apply_op};
use serde_json::Value;

use crate::cli::Outcome;
use crate::verbs::work::crud::{find_live, who_of};
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

pub(super) use cases::append as append_case;

const PLAN: &str = "test-plan";

/// `coord test plan <sub-verb> ...`.
pub(super) fn plan(r: &mut Run<'_>) -> Outcome {
    let sub = r.shift().unwrap_or_default();
    match sub.as_str() {
        "add" => add::add(r),
        "cases" => cases::cases(r),
        "start" => run::start(r),
        "end" => run::end(r),
        "show" => show::show(r),
        _ => Err(bad("plan verb must be one of add, cases, start, end, show.").into()),
    }
}

/// The live test plan an argument names.
pub(super) fn plan_of(board: &Board, id: Option<&str>) -> Res<Value> {
    let found = find_live(board, "items", id)?;
    if found["type"] != PLAN {
        return Err(bad(format!(
            "{} is not a test plan.",
            display(&found["id"])
        )));
    }
    Ok(found)
}

/// Applies an op as the caller and gives its result.
pub(super) fn send(board: &mut Board, who: &Caller, operation: &Value) -> Res<OpResult> {
    let res = apply_op(board, operation, &who_of(who), None);
    match res.status {
        Status::Applied => Ok(res),
        Status::Refused | Status::Conflict => Err(bad(res.outcome)),
    }
}

/// The ids in the `cases` of a plan, in order.
fn cases_of(plan: &Value) -> Vec<String> {
    let cases = plan["fields"]["cases"].as_array();
    cases.into_iter().flatten().map(display).collect()
}
