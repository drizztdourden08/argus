//! `coord test plan show I-n`: the plan, its cases with the latest status of each, and its runs.

use argus_board::Board;
use argus_board::json::{display, doc};
use serde_json::Value;

use super::{cases_of, plan_of};
use crate::cli::Outcome;
use crate::verbs::work::run::Run;

fn case_line(board: &Board, run: Option<&Value>, case: &str) -> String {
    let test = doc(board, "tests", case).cloned().unwrap_or(Value::Null);
    let in_run = run.and_then(|r| {
        let results = r["results"].as_array()?;
        results
            .iter()
            .rev()
            .find(|x| x["test"].as_str() == Some(case))
    });
    let status = in_run.map_or_else(|| display(&test["status"]), |x| display(&x["status"]));
    format!("  {case}  {status:<8}  {}", display(&test["title"]))
}

fn run_line(run: &Value) -> String {
    let state = if run["endedAt"].is_null() {
        "open"
    } else {
        "ended"
    };
    let counted = run["results"].as_array().map_or(0, Vec::len);
    let bundle = run["bundle"]
        .as_str()
        .map_or_else(String::new, |b| format!(", bundle {b}"));
    format!(
        "  run {}: {state}, {counted} result(s){bundle}",
        display(&run["n"])
    )
}

/// `coord test plan show`.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let plan = plan_of(board, r.arg(0))?;
        let runs = plan["fields"]["runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut lines = vec![format!(
            "{} {} ({})",
            display(&plan["id"]),
            display(&plan["title"]),
            display(&plan["stage"])
        )];
        let bundle = plan["fields"]["bundle"].as_str().unwrap_or("none");
        lines.push(format!("bundle: {bundle}"));
        lines.push("cases:".to_owned());
        let latest = runs.last();
        lines.extend(cases_of(&plan).iter().map(|c| case_line(board, latest, c)));
        lines.push("runs:".to_owned());
        lines.extend(runs.iter().map(run_line));
        let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
        r.say(&shown);
        Ok(())
    })
}
