//! A test result of the owner: a failing outcome wakes the test's thread. A pass wakes nobody; a
//! run that ends wakes the plan's thread through the `plan-run-done` item event.

use super::{ApplyCtx, FAILING};
use crate::board::EntryView;
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
    if !FAILING.contains(&outcome) {
        return Vec::new();
    }
    let note = clip_full(entry.rec.id.as_str(), entry.text.as_str());
    let note = if note.is_empty() {
        "no note".to_owned()
    } else {
        note
    };
    let line = format!("{} {}: {note}", case.rec.id.as_str(), word(outcome));
    let reason = format!("test-{outcome}");
    vec![WakeInput::about(
        ctx.scope,
        thread,
        &reason,
        case.rec.id.as_str(),
        line,
    )]
}
