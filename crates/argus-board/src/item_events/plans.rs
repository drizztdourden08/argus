//! Test plans: ready to run for the owner, and a run that ended for the plan's thread.

use serde_json::Value;

use super::people::{actor, is_owner, type_of};
use super::{EventKind, Hearer, ItemEvent, Moved};
use crate::Board;
use crate::json::{doc, text};
use crate::launchers::bundle_status;

const PLAN: &str = "test-plan";
const STATUSES: [&str; 5] = ["pass", "fail", "partial", "blocked", "skip"];

/// Is the plan in stage `ready` with a bundle that is ready too?
fn ready_to_run(board: &Board, plan: &Value) -> bool {
    if text(plan, "stage") != Some("ready") {
        return false;
    }
    let Some(tag) = text(&plan["fields"], "bundle") else {
        return false;
    };
    doc(board, "builds", tag).is_some_and(|rec| bundle_status(board, tag, rec) == "ready")
}

/// A plan that became ready to run, bundle and all: the owner runs it. Not told when he did it himself.
pub(super) fn ready(m: &Moved<'_>) -> Vec<ItemEvent> {
    let becomes = type_of(m.now) == PLAN
        && ready_to_run(m.seen.after, m.now)
        && !ready_to_run(m.seen.before, m.was);
    if becomes && !is_owner(&actor(m.now)) {
        let bundle = text(&m.now["fields"], "bundle").unwrap_or_default();
        vec![m.event(EventKind::PlanReady, Hearer::Owner, bundle.to_owned())]
    } else {
        Vec::new()
    }
}

fn runs(plan: &Value) -> &[Value] {
    plan["fields"]["runs"]
        .as_array()
        .map_or(&[][..], Vec::as_slice)
}

fn is_over(run: &Value) -> bool {
    !run["endedAt"].is_null() && run["endedAt"] != ""
}

/// The counts of a run by status: `4 pass, 1 fail`.
fn counts(run: &Value) -> String {
    let results = run["results"].as_array().map_or(&[][..], Vec::as_slice);
    let parts: Vec<String> = STATUSES
        .iter()
        .filter_map(|status| {
            let n = results
                .iter()
                .filter(|r| text(r, "status") == Some(*status))
                .count();
            (n > 0).then(|| format!("{n} {status}"))
        })
        .collect();
    parts.join(", ")
}

fn was_over(plan: &Value, n: &Value) -> bool {
    runs(plan).iter().any(|r| r["n"] == *n && is_over(r))
}

/// A run that ended since the last export, told to the thread of the plan with the counts.
pub(super) fn run_done(m: &Moved<'_>) -> Vec<ItemEvent> {
    if type_of(m.now) != PLAN {
        return Vec::new();
    }
    let Some(thread) = text(m.now, "thread").filter(|t| !t.is_empty()) else {
        return Vec::new();
    };
    runs(m.now)
        .iter()
        .filter(|run| is_over(run) && !was_over(m.was, &run["n"]))
        .map(|run| {
            let detail = format!("run {}: {}", run["n"], counts(run));
            m.event(
                EventKind::PlanRunDone,
                Hearer::Thread(thread.to_owned()),
                detail,
            )
        })
        .collect()
}
