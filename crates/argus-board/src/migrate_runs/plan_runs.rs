//! The runs kept on test plans (`fields.runs`) become test runs.

use serde_json::{Map, Value, json};

use super::make::{NewRun, run};
use super::status::{all_passed, build_of, counts_line};
use super::{Report, has_legacy, plans_of};
use crate::Board;
use crate::error::BoardError;
use crate::json::{display, doc};

fn legacy_key(plan: &str, n: i64) -> String {
    format!("{plan}#{n}")
}

/// The stage of a legacy run: running while open, else passed when every case passed.
fn stage_of(old: &Value, results: &[Value]) -> &'static str {
    if old["endedAt"].is_null() {
        return "running";
    }
    let counted = old["summary"]["total"].as_u64() > Some(0)
        && old["summary"]["pass"] == old["summary"]["total"];
    if counted || (old["summary"].is_null() && all_passed(results)) {
        "passed"
    } else {
        "reported"
    }
}

fn legacy_fields(old: &Value, key: &str) -> Map<String, Value> {
    let mut legacy = Map::from_iter([("legacyRun".to_owned(), json!(key))]);
    if let Some(tag) = old["bundle"].as_str().filter(|t| !t.is_empty()) {
        legacy.insert("legacyBundle".to_owned(), json!(tag));
    }
    legacy
}

fn migrate_one(
    board: &mut Board,
    (plan, old): (&str, &Value),
    actor: &str,
) -> Result<Option<String>, BoardError> {
    let n = old["n"].as_i64().unwrap_or(1);
    let key = legacy_key(plan, n);
    if has_legacy(board, plan, ("legacyRun", &key)) {
        return Ok(None);
    }
    let results = old["results"].as_array().cloned().unwrap_or_default();
    let stage = stage_of(old, &results);
    let record = old["bundle"].as_str().map(|t| board["builds"][t].clone());
    let (title, thread) = doc(board, "items", plan).map_or_else(
        || (String::new(), String::new()),
        |p| (display(&p["title"]), display(&p["thread"])),
    );
    let by = old["by"].as_str().filter(|b| !b.is_empty()).unwrap_or(actor);
    let new = NewRun {
        plan: plan.to_owned(),
        plan_title: title,
        thread,
        n,
        stage,
        assignee: None,
        build: record.and_then(|rec| build_of(board, &rec)),
        results,
        summary: counts_line(&old["summary"]),
        started: old["startedAt"].clone(),
        ended: old["endedAt"].clone(),
        by: by.to_owned(),
        legacy: legacy_fields(old, &key),
    };
    run(board, new).map(Some)
}

/// Migrate the runs of every live plan.
pub(super) fn migrate(
    board: &mut Board,
    actor: &str,
    report: &mut Report,
) -> Result<(), BoardError> {
    for plan in plans_of(board) {
        let old_runs = board["items"][plan.as_str()]["fields"]["runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for old in &old_runs {
            if let Some(id) = migrate_one(board, (&plan, old), actor)? {
                let n = display(&old["n"]);
                report.lines.push(format!("run {n} of {plan} becomes {id}"));
                report.from_plan_runs += 1;
            }
        }
    }
    Ok(())
}
