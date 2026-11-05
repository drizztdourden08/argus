//! The runs of a plan on the board and the facts read from them.

use serde_json::{Value, json};

use crate::Board;
use crate::json::{section, truthy};

pub(super) const PLAN: &str = "test-plan";
pub(super) const RUN: &str = "test-run";

/// The cases of a plan, in order.
pub(crate) fn cases_of(plan: &Value) -> Vec<String> {
    let cases = plan["fields"]["cases"].as_array();
    let listed = cases.into_iter().flatten();
    listed
        .filter_map(|c| c.as_str().map(str::to_owned))
        .collect()
}

/// The runs of a plan that are not in the trash, oldest first.
pub(crate) fn runs_of(board: &Board, plan: &str) -> Vec<Value> {
    let mut runs: Vec<Value> = section(board, "items")
        .values()
        .filter(|i| i["type"] == RUN && !truthy(i, "deletedAt"))
        .filter(|i| i["fields"]["plan"].as_str() == Some(plan))
        .cloned()
        .collect();
    runs.sort_by_key(|r| r["n"].as_i64().unwrap_or_default());
    runs
}

/// Whether a run is under way: started and not ended.
pub(super) fn is_open(run: &Value) -> bool {
    run["stage"] == "running"
}

/// Whether a run has an end time.
pub(crate) fn is_ended(run: &Value) -> bool {
    !run["endedAt"].is_null()
}

/// The latest status a run holds for a case, `None` while the case has no result.
pub(crate) fn last_status<'a>(run: &'a Value, case: &str) -> Option<&'a str> {
    let results = run["fields"]["results"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
    let latest = results
        .iter()
        .rev()
        .find(|r| r["test"].as_str() == Some(case));
    latest.and_then(|r| r["status"].as_str())
}

/// The counts of a run by status, over the cases of the plan; a case without a result is `notRun`.
pub(super) fn summary_of(run: &Value, cases: &[String]) -> Value {
    let count = |status: Option<&str>| {
        cases
            .iter()
            .filter(|c| last_status(run, c) == status)
            .count()
    };
    json!({
        "pass": count(Some("pass")), "fail": count(Some("fail")),
        "partial": count(Some("partial")), "blocked": count(Some("blocked")),
        "skip": count(Some("skip")), "notRun": count(None), "total": cases.len(),
    })
}

/// How many cases of the plan the run has a result for, and how many cases there are.
pub(super) fn progress(run: &Value, cases: &[String]) -> (usize, usize) {
    let done = cases
        .iter()
        .filter(|c| last_status(run, c).is_some())
        .count();
    (done, cases.len())
}
