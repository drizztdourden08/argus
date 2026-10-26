//! A test result of the owner: a failing outcome wakes the test's thread, the pass that finishes a bundle wakes it once.

use super::{ApplyCtx, FAILING};
use crate::board::{BoardView, EntryView, TestView};
use crate::text::clip_full;
use crate::wake::WakeInput;

fn word(outcome: &str) -> &'static str {
    match outcome {
        "fail" => "failed",
        "partial" => "partial",
        _ => "blocked",
    }
}

fn live_manual<'a>(board: Option<&'a BoardView>, thread: &str, build: &str) -> Vec<&'a TestView> {
    board
        .into_iter()
        .flat_map(|b| b.tests.values())
        .filter(|t| {
            t.build.is(build) && t.thread.is(thread) && t.mode.is("manual") && t.rec.is_live()
        })
        .collect()
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
    bundle_complete(ctx, board, case, thread)
        .into_iter()
        .collect()
}

fn bundle_complete(
    ctx: &ApplyCtx<'_>,
    board: &BoardView,
    case: &TestView,
    thread: &str,
) -> Option<WakeInput> {
    let build = case.build.some()?;
    if ctx.entry?.outcome.as_str() != "pass" {
        return None;
    }
    let group = live_manual(Some(board), thread, build);
    let pending_before = live_manual(ctx.before, thread, build)
        .iter()
        .filter(|t| t.status.is("pending"))
        .count();
    if group.is_empty() || group.iter().any(|t| t.status.is("pending")) || pending_before == 0 {
        return None;
    }
    let pass = group.iter().filter(|t| t.status.is("pass")).count();
    let fail = group.iter().filter(|t| t.status.is_any(&FAILING)).count();
    let line = format!("{build}: all manual tests run, {pass} pass {fail} fail");
    Some(WakeInput::about(
        ctx.scope,
        thread,
        "bundle-complete",
        build,
        line,
    ))
}
