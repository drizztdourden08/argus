//! Each bundle tag becomes one test run of its plan.

use serde_json::{Value, json};

use super::make::{NewRun, plan, run};
use super::status::{build_of, stage_of};
use super::{Report, has_legacy, next_n, plans_of};
use crate::Board;
use crate::error::BoardError;
use crate::json::{display, doc, ids, section, truthy};

/// Every bundle tag: the records of `board.builds` and the tags on live tests, sorted.
fn tags_of(board: &Board) -> Vec<String> {
    let mut tags: Vec<String> = section(board, "builds").keys().cloned().collect();
    for test in live_tests(board) {
        if let Some(tag) = test["build"].as_str().filter(|t| !t.is_empty()) {
            tags.push(tag.to_owned());
        }
    }
    tags.sort();
    tags.dedup();
    tags
}

fn live_tests(board: &Board) -> Vec<&Value> {
    let all = section(board, "tests").values();
    all.filter(|t| !truthy(t, "deletedAt")).collect()
}

/// The ids of the live tests that carry the tag, in id order.
fn tagged(board: &Board, tag: &str) -> Vec<String> {
    let mut found = ids(board, "tests");
    found.retain(|id| {
        doc(board, "tests", id)
            .is_some_and(|t| t["build"].as_str() == Some(tag) && !truthy(t, "deletedAt"))
    });
    found
}

/// The latest result of each tagged test, as a run records it.
fn results_of(board: &Board, tests: &[String]) -> Vec<Value> {
    let rows = tests.iter().filter_map(|id| {
        let result = &doc(board, "tests", id)?["result"];
        let status = result["status"].as_str().filter(|s| *s != "pending")?;
        Some(json!({
            "test": id, "status": status, "note": result["text"],
            "at": result["at"], "by": result["by"],
            "proofs": result["proofs"].as_array().cloned().unwrap_or_default(),
        }))
    });
    rows.collect()
}

/// The plan whose `fields.bundle` is the tag, else a new one with the tagged tests as cases.
fn plan_for(
    board: &mut Board,
    (tag, rec, tests): (&str, &Value, &[String]),
    actor: &str,
    report: &mut Report,
) -> Result<Option<String>, BoardError> {
    let named = plans_of(board).into_iter().find(|p| {
        let fields = &board["items"][p.as_str()]["fields"];
        fields["bundle"].as_str() == Some(tag) || fields["legacyBundle"].as_str() == Some(tag)
    });
    if named.is_some() || tests.is_empty() {
        return Ok(named);
    }
    let title = rec["title"]
        .as_str()
        .filter(|t| !t.is_empty())
        .unwrap_or(tag);
    let thread = rec["thread"]
        .as_str()
        .filter(|t| !t.is_empty())
        .unwrap_or(actor);
    let id = plan(
        board,
        (&format!("Bundle {title}"), thread, actor),
        (tag, tests),
    )?;
    report.plans += 1;
    report
        .lines
        .push(format!("bundle {tag} gets the new plan {id}"));
    Ok(Some(id))
}

fn migrate_tag(
    board: &mut Board,
    tag: &str,
    actor: &str,
    report: &mut Report,
) -> Result<(), BoardError> {
    let rec = board["builds"][tag].clone();
    let tests = tagged(board, tag);
    let Some(plan_id) = plan_for(board, (tag, &rec, &tests), actor, report)? else {
        report
            .lines
            .push(format!("bundle {tag} has no test and no plan: skipped"));
        return Ok(());
    };
    if has_legacy(board, &plan_id, ("legacyBundle", tag)) {
        report
            .lines
            .push(format!("bundle {tag} is already a run of {plan_id}"));
        return Ok(());
    }
    let results = results_of(board, &tests);
    let (stage, assignee) = stage_of(board, (tag, &rec), &results);
    let (title, thread) = (
        display(&board["items"][plan_id.as_str()]["title"]),
        display(&board["items"][plan_id.as_str()]["thread"]),
    );
    let n = next_n(board, &plan_id);
    let new = NewRun {
        plan: plan_id.clone(),
        plan_title: title,
        thread,
        n,
        stage,
        assignee,
        build: build_of(board, &rec),
        results,
        summary: None,
        started: rec["createdAt"].clone(),
        ended: rec["closedAt"].clone(),
        by: rec["createdBy"].as_str().unwrap_or(actor).to_owned(),
        legacy: serde_json::Map::from_iter([("legacyBundle".to_owned(), json!(tag))]),
    };
    let id = run(board, new)?;
    report.from_bundles += 1;
    report.lines.push(format!(
        "bundle {tag} becomes {id}, run {n} of {plan_id} ({stage})"
    ));
    Ok(())
}

/// Migrate every bundle tag.
pub(super) fn migrate(
    board: &mut Board,
    actor: &str,
    report: &mut Report,
) -> Result<(), BoardError> {
    for tag in tags_of(board) {
        migrate_tag(board, &tag, actor, report)?;
    }
    Ok(())
}
