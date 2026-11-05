//! A test result of the owner: a failing outcome wakes the test's thread, the pass that finishes a run wakes it once.

use serde_json::Value;

use super::{ApplyCtx, FAILING};
use crate::board::{BoardView, EntryView};
use crate::text::clip_full;
use crate::wake::WakeInput;

fn word(outcome: &str) -> &'static str {
    match outcome {
        "fail" => "failed",
        "partial" => "partial",
        _ => "blocked",
    }
}

pub(super) fn from_result(ctx: &ApplyCtx<'_>, entry: &EntryView) -> Vec<WakeInput> {
    let Some(board) = ctx.board else {
        return Vec::new();
    };
    let Some(case) = board.tests.get(entry.reply_to.as_str()) else {
        return Vec::new();
    };
    let Some(thread) = case
        .thread
        .some()
        .filter(|_| case.rec.deleted_at.some().is_none())
    else {
        return Vec::new();
    };
    let outcome = entry.outcome.as_str();
    if FAILING.contains(&outcome) {
        let note = clip_full(entry.rec.id.as_str(), entry.text.as_str());
        let note = if note.is_empty() {
            "no note".to_owned()
        } else {
            note
        };
        let line = format!("{} {}: {note}", case.rec.id.as_str(), word(outcome));
        let reason = format!("test-{outcome}");
        return vec![WakeInput::about(
            ctx.scope,
            thread,
            &reason,
            case.rec.id.as_str(),
            line,
        )];
    }
    if outcome != "pass" {
        return Vec::new();
    }
    run_complete(ctx, board, case.rec.id.as_str(), thread)
        .into_iter()
        .collect()
}

/// Is the item a test run of this thread that ended (reported or passed)?
fn ended_run(item: &Value, thread: &str) -> bool {
    item["type"] == "test-run"
        && item["thread"] == thread
        && item["deletedAt"].is_null()
        && matches!(item["stage"].as_str(), Some("reported" | "passed"))
}

fn results_of(run: &Value) -> &[Value] {
    run["fields"]["results"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
}

fn count(run: &Value, wanted: impl Fn(&str) -> bool) -> usize {
    results_of(run)
        .iter()
        .filter(|r| r["status"].as_str().is_some_and(&wanted))
        .count()
}

/// The run that this pass finished: it holds a result of the test, ended now and had not ended in the last export.
fn run_complete(
    ctx: &ApplyCtx<'_>,
    board: &BoardView,
    test: &str,
    thread: &str,
) -> Option<WakeInput> {
    let before = ctx.before.map(BoardView::json);
    let (id, run) = board.json()["items"]
        .as_object()?
        .iter()
        .find(|(id, run)| {
            ended_run(run, thread)
                && results_of(run).iter().any(|r| r["test"] == test)
                && !before.is_some_and(|b| ended_run(&b["items"][id.as_str()], thread))
        })?;
    let pass = count(run, |s| s == "pass");
    let fail = count(run, |s| FAILING.contains(&s));
    let line = format!(
        "run {}: all tests run, {pass} pass {fail} fail",
        run["n"]
    );
    Some(WakeInput::about(
        ctx.scope,
        thread,
        "run-complete",
        id,
        line,
    ))
}
