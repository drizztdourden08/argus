//! `coord test plan start I-n [--bundle tag]` and `coord test plan end I-n`: open the run of a
//! plan, and close it with its summary. Results go in with `coord test result T-n ... --run I-n`.

use argus_board::json::display;
use serde_json::{Map, Value, json};

use super::{plan_of, send};
use crate::cli::Outcome;
use crate::verbs::work::crud::op;
use crate::verbs::work::run::Run;

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

/// `coord test plan start`.
pub(super) fn start(r: &mut Run<'_>) -> Outcome {
    let mut rest = Map::new();
    if let Some(bundle) = r.text_opt("bundle") {
        rest.insert("bundle".into(), json!(bundle));
    }
    run_op(r, "plan-run-start", rest)
}

/// `coord test plan end`.
pub(super) fn end(r: &mut Run<'_>) -> Outcome {
    run_op(r, "plan-run-end", Map::new())
}
