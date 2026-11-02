//! Boards and a runner for the stall rule tests: project `rotp` with a live lead idle for 20 minutes.

use serde_json::{Map, Value, json};

use super::{Found, IdleWatch, Limits, RuleInput, find};
use crate::memory::MemoryBoards;
use crate::testkit::{HOUR, MIN, START, board, merge, thread};
use crate::time::iso;

/// The moment every rule test looks at the boards.
pub(super) const NOW: i64 = START;

/// A work item of the lead, last written two hours before `NOW`.
pub(super) fn item(id: &str, status: &str, extra: &Value) -> Value {
    let mut record = json!({
        "id": id, "title": id, "status": status, "thread": "lead",
        "createdAt": iso(NOW - 3 * HOUR), "updatedAt": iso(NOW - 2 * HOUR),
    });
    merge(&mut record, extra);
    record
}

/// A project board: the lead (idle, last seen 20 minutes before `NOW`) and the items, plus the `extra` keys.
pub(super) fn project(lead: &Value, items: &[Value], extra: &Value) -> Value {
    let items: Map<String, Value> = items
        .iter()
        .map(|item| {
            (
                item["id"].as_str().unwrap_or_default().to_owned(),
                item.clone(),
            )
        })
        .collect();
    let mut lead_fields = json!({"lastSeen": iso(NOW - 20 * MIN)});
    merge(&mut lead_fields, lead);
    let mut value = json!({
        "threads": {"lead": thread("lead", "coordinator", &lead_fields)},
        "items": items,
    });
    merge(&mut value, extra);
    value
}

/// Boards by project key.
pub(super) fn boards(projects: &[(&str, Value)]) -> MemoryBoards {
    let boards = MemoryBoards::new();
    for (key, value) in projects {
        boards.set(key, board(value));
    }
    boards
}

/// The rules at `now`, with the default limits and every project busy or not.
pub(super) fn run(boards: &MemoryBoards, watch: &IdleWatch, now: i64, busy: bool) -> Vec<Found> {
    let is_busy = move |_: &str| busy;
    find(
        now,
        &RuleInput {
            boards,
            busy: &is_busy,
            idle: watch,
            limits: Limits::default(),
        },
    )
}
