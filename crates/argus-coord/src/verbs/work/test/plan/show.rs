//! `coord test plan show I-n`: the plan, its cases with the latest status of each, and its runs.

use argus_board::Board;
use argus_board::json::{display, doc};
use serde_json::Value;

use super::{cases_of, plan_of};
use crate::cli::Outcome;
use crate::verbs::work::run::Run;
use crate::verbs::work::test::runs::rows;
use crate::verbs::work::test::spec::mode_label;

fn case_line(board: &Board, run: Option<&Value>, case: &str) -> String {
    let test = doc(board, "tests", case).cloned().unwrap_or(Value::Null);
    let in_run = run.and_then(|r| {
        let results = r["fields"]["results"].as_array()?;
        results
            .iter()
            .rev()
            .find(|x| x["test"].as_str() == Some(case))
    });
    let status = in_run.map_or_else(|| display(&test["status"]), |x| display(&x["status"]));
    let mode = mode_label(&test["mode"]);
    format!(
        "  {case}  {status:<8}  {mode:<6}  {}",
        display(&test["title"])
    )
}

fn run_line(row: &Value) -> String {
    let build = row["build"]
        .as_str()
        .map_or_else(String::new, |b| format!(", build {b}"));
    format!(
        "  {} (run {}): {}{build}, {} pass, {} fail, {} to run",
        display(&row["run"]),
        display(&row["n"]),
        display(&row["stage"]),
        display(&row["pass"]),
        display(&row["fail"]),
        display(&row["toRun"]),
    )
}

/// `coord test plan show`.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let plan = plan_of(board, r.arg(0))?;
        let id = display(&plan["id"]);
        let runs = rows(board, Some(&id));
        let latest = runs
            .last()
            .and_then(|row| doc(board, "items", &display(&row["run"])));
        let mut lines = vec![format!(
            "{id} {} ({})",
            display(&plan["title"]),
            display(&plan["stage"])
        )];
        lines.push("cases:".to_owned());
        lines.extend(cases_of(&plan).iter().map(|c| case_line(board, latest, c)));
        lines.push("runs:".to_owned());
        lines.extend(runs.iter().map(run_line));
        let shown: Vec<&str> = lines.iter().map(String::as_str).collect();
        r.say(&shown);
        Ok(())
    })
}
