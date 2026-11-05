//! `coord test run list [--plan I-n] [--json]`: every run with its plan, build, stage and how
//! many cases passed, failed or are still to run.

use serde_json::Value;

use super::rows::rows;
use crate::cli::Outcome;
use crate::cli::out::{as_json, table};
use crate::verbs::work::listing::cell;
use crate::verbs::work::run::Run;

const HEAD: [&str; 8] = [
    "run", "plan", "build", "stage", "pass", "fail", "to run", "assignee",
];
const KEYS: [&str; 8] = [
    "run", "plan", "build", "stage", "pass", "fail", "toRun", "assignee",
];

fn line(row: &Value) -> Vec<String> {
    KEYS.iter().map(|key| cell(&row[*key])).collect()
}

/// `coord test run list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let all = rows(board, r.text_opt("plan"));
        if r.options.flag("json") {
            as_json(&mut *r.inv.stdout, &Value::Array(all));
            return Ok(());
        }
        if all.is_empty() {
            r.say(&["no test runs: coord test plan run <plan> --build <launcher> starts one."]);
            return Ok(());
        }
        let mut lines = vec![HEAD.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>()];
        lines.extend(all.iter().map(line));
        r.say(&[&table(&lines)]);
        Ok(())
    })
}
