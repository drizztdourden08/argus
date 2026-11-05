//! Test runs: ready to run for the owner, and a run that ended for the plan's thread.

use serde_json::Value;

use super::people::{actor, is_owner, type_of};
use super::{EventKind, Hearer, ItemEvent, Moved};
use crate::Board;
use crate::json::text;
use crate::launchers::build_ready;

const RUN: &str = "test-run";
const STATUSES: [&str; 5] = ["pass", "fail", "partial", "blocked", "skip"];

/// Is the run in stage `ready` with a build that is live?
fn ready_to_run(board: &Board, run: &Value) -> bool {
    text(run, "stage") == Some("ready")
        && text(&run["fields"], "build").is_some_and(|name| build_ready(board, name))
}

/// A run that became ready to run, build and all: the owner runs it. Not told when he did it himself.
pub(super) fn ready(m: &Moved<'_>) -> Vec<ItemEvent> {
    let becomes = type_of(m.now) == RUN
        && ready_to_run(m.seen.after, m.now)
        && !ready_to_run(m.seen.before, m.was);
    if becomes && !is_owner(&actor(m.now)) {
        let build = text(&m.now["fields"], "build").unwrap_or_default();
        vec![m.event(EventKind::RunReady, Hearer::Owner, build.to_owned())]
    } else {
        Vec::new()
    }
}

fn is_over(run: &Value) -> bool {
    matches!(text(run, "stage"), Some("reported" | "passed"))
}

/// The counts of a run by status: `4 pass, 1 fail`.
fn counts(run: &Value) -> String {
    let results = run["fields"]["results"]
        .as_array()
        .map_or(&[][..], Vec::as_slice);
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

/// A run that reached reported or passed since the last export, told to the thread of its plan with
/// the counts.
pub(super) fn run_done(m: &Moved<'_>) -> Vec<ItemEvent> {
    if type_of(m.now) != RUN || !is_over(m.now) || is_over(m.was) {
        return Vec::new();
    }
    let Some(thread) = text(m.now, "thread").filter(|t| !t.is_empty()) else {
        return Vec::new();
    };
    let detail = format!("run {}: {}", m.now["n"], counts(m.now));
    vec![m.event(
        EventKind::PlanRunDone,
        Hearer::Thread(thread.to_owned()),
        detail,
    )]
}
