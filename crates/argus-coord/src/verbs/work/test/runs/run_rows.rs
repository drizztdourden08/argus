//! The rows of `coord test run list`: one per run, with the plan, the build, the stage and what
//! the plan's cases did in that run. "to run" is every case with no result yet.

use argus_board::Board;
use argus_board::json::{display, doc, section, truthy};
use serde_json::{Value, json};

const FAILING: [&str; 3] = ["fail", "partial", "blocked"];

/// The latest status the run holds for a case.
fn latest<'a>(run: &'a Value, case: &str) -> Option<&'a str> {
    let results = run["fields"]["results"].as_array()?;
    let found = results
        .iter()
        .rev()
        .find(|x| x["test"].as_str() == Some(case));
    found.and_then(|x| x["status"].as_str())
}

/// `(pass, fail, to run)` of a run over the cases of its plan.
fn counts(board: &Board, run: &Value) -> (usize, usize, usize) {
    let plan_id = run["fields"]["plan"].as_str().unwrap_or_default();
    let plan = doc(board, "items", plan_id);
    let cases = plan.and_then(|p| p["fields"]["cases"].as_array());
    let mut seen = (0, 0, 0);
    for case in cases.into_iter().flatten().filter_map(Value::as_str) {
        match latest(run, case) {
            Some("pass") => seen.0 += 1,
            Some(s) if FAILING.contains(&s) => seen.1 += 1,
            _ => seen.2 += 1,
        }
    }
    seen
}

fn row(board: &Board, run: &Value) -> Value {
    let (pass, fail, to_run) = counts(board, run);
    json!({
        "run": run["id"], "title": run["title"], "plan": run["fields"]["plan"],
        "build": run["fields"]["build"], "stage": run["stage"], "n": run["n"],
        "assignee": run["assignee"], "pass": pass, "fail": fail, "toRun": to_run,
    })
}

/// The rows of the runs on the board (of one plan, when `plan` is given), by plan and number.
pub(in crate::verbs::work::test) fn rows(board: &Board, plan: Option<&str>) -> Vec<Value> {
    let mut out: Vec<Value> = section(board, "items")
        .values()
        .filter(|i| i["type"] == "test-run" && !truthy(i, "deletedAt"))
        .filter(|i| plan.is_none_or(|p| i["fields"]["plan"].as_str() == Some(p)))
        .map(|i| row(board, i))
        .collect();
    out.sort_by_key(|r| (display(&r["plan"]), r["n"].as_i64().unwrap_or_default()));
    out
}
