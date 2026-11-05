//! `coord test plan run I-n [--build <launcher>]` (also `start`) and `coord test plan end I-n`:
//! open a run of a plan on a build, and end it with its summary. Results go in with
//! `coord test result T-n ... --run I-n`.

use argus_board::Board;
use argus_board::json::display;
use serde_json::{Map, Value, json};

use super::{plan_of, send};
use crate::cli::Outcome;
use crate::verbs::work::crud::op;
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

/// Sends a run op for the plan the first word names, and prints how it went.
fn run_op(r: &mut Run<'_>, kind: &str, rest: Map<String, Value>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let plan = plan_of(board, r.arg(0))?;
        let id = display(&plan["id"]);
        let res = send(board, &who, &op((kind, "items", &id), rest, r.options)?)?;
        r.say(&[&format!("{}.", res.outcome)]);
        Ok(())
    })
}

/// The `plan-run-start` rest: the launcher the run is for.
fn start_rest(r: &Run<'_>) -> Res<Map<String, Value>> {
    if r.text_opt("bundle").is_some() {
        return Err(bad(
            "--bundle is gone: a run carries its build, pass --build <launcher>.",
        ));
    }
    let mut rest = Map::new();
    if let Some(build) = r.text_opt("build") {
        rest.insert("build".into(), json!(build));
    }
    Ok(rest)
}

/// Starts a run of the plan an id names, as the caller, and gives how it went.
pub(super) fn start_op(board: &mut Board, who: &Caller, plan: &str, r: &Run<'_>) -> Res<String> {
    let operation = op(("plan-run-start", "items", plan), start_rest(r)?, r.options)?;
    Ok(send(board, who, &operation)?.outcome)
}

/// `coord test plan run` and `coord test plan start`.
pub(super) fn start(r: &mut Run<'_>) -> Outcome {
    let rest = start_rest(r)?;
    run_op(r, "plan-run-start", rest)
}

/// `coord test plan end`.
pub(super) fn end(r: &mut Run<'_>) -> Outcome {
    run_op(r, "plan-run-end", Map::new())
}
