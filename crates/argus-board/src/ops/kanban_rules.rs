//! The write-time rules of the board's Kanban config, applied around an item op. Before the op
//! the board is kept; after it, the items it touched are checked by the column they landed in.
//! A rule that refuses puts the board back and refuses the op; one that warns adds its line to
//! the outcome and to `warnings`. A board with no write-time rule (the default) pays nothing.

use serde_json::{Value, json};

use super::result::{OpResult, Status};
use crate::Board;
use crate::json::section;
use crate::kanban_config::{KanbanConfig, effective, evaluate, has_write_rules};

/// The fields of an item a column judges.
const WATCHED: [&str; 7] = [
    "status", "stage", "parent", "assignee", "thread", "type", "fields",
];

/// The board as it was before an op, and the config that judges the write.
pub(super) struct Guard {
    before: Board,
    config: KanbanConfig,
}

/// Starts a guard for an op that writes items, when the board has rules to apply.
pub(super) fn begin(board: &Board, op: &Value) -> Option<Guard> {
    let writes = matches!(op["kind"].as_str(), Some("create" | "edit" | "move"));
    if !writes {
        return None;
    }
    let config = effective(board);
    has_write_rules(&config).then(|| Guard {
        before: board.clone(),
        config,
    })
}

fn differs(was: Option<&Value>, now: &Value) -> bool {
    was.is_none_or(|was| WATCHED.iter().any(|k| was[*k] != now[*k]))
}

/// The items the op created or changed in a field a column judges.
fn touched(before: &Board, board: &Board) -> Vec<String> {
    let was = section(before, "items");
    section(board, "items")
        .iter()
        .filter(|(id, now)| differs(was.get(*id), now))
        .map(|(id, _)| id.clone())
        .collect()
}

fn with_warnings(mut result: OpResult, warnings: Vec<String>) -> OpResult {
    if warnings.is_empty() {
        return result;
    }
    result.outcome = format!("{} Warning: {}", result.outcome, warnings.join(" "));
    let mut all = match result.extra.get("warnings") {
        Some(Value::Array(list)) => list.clone(),
        _ => Vec::new(),
    };
    all.extend(warnings.into_iter().map(Value::String));
    result.with_extra("warnings", json!(all))
}

/// Judges an applied op: refused (and the board put back) or applied with the warnings.
pub(super) fn finish(board: &mut Board, guard: Option<Guard>, result: OpResult) -> OpResult {
    let Some(guard) = guard.filter(|_| result.status == Status::Applied) else {
        return result;
    };
    let ids = touched(&guard.before, board);
    let verdict = evaluate(board, &guard.config, &ids);
    match verdict.refusal {
        Some(refusal) => {
            *board = guard.before;
            OpResult::refused(refusal)
        }
        None => with_warnings(result, verdict.warnings),
    }
}
