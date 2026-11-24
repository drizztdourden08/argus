//! Older boards predate some collections; normalize fills them in so every verb can rely on
//! them. A board still on schema 1 is migrated and embedded agents are converted.

use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use super::migrate::{convert_agents, migrate_to_2};
use super::migrate_lead;
use crate::Board;
use crate::json::{default_field, set, slot, unset};

const COLLECTIONS: [&str; 18] = [
    "counters",
    "threads",
    "items",
    "tests",
    "entries",
    "locks",
    "actions",
    "agents",
    "log",
    "inbox",
    "runDefs",
    "builds",
    "launchers",
    "prs",
    "worktreeKeeps",
    "worktreeAnswers",
    "dirty",
    "versions",
];

/// Whether a board is the portfolio board (the registry and the dependency ledger).
fn is_portfolio(board: &Board) -> bool {
    board["kind"] == "portfolio" || board["project"] == PORTFOLIO
}

/// Whether a project board is still on a schema before 2 (the portfolio board never is).
pub(crate) fn stale_schema(board: &Board) -> bool {
    if board["kind"] == "portfolio" {
        return false;
    }
    match &board["schema"] {
        Value::Null => true,
        Value::Number(n) => n.as_f64().is_some_and(|f| f < 2.0),
        _ => false,
    }
}

/// Fill in every collection an older board lacks (the portfolio board also gets its `deps` ledger and `boards`), migrate schema 1 to 2 and convert embedded
/// agents into agents docs.
pub fn normalize(board: &mut Board) {
    for key in COLLECTIONS {
        let s = slot(board, key);
        if s.is_null() {
            *s = json!({});
        }
    }
    if is_portfolio(board) {
        default_field(board, "deps", json!({}));
        default_field(board, "boards", json!({}));
    }
    if !board["worktreeRules"].is_array() {
        set(board, "worktreeRules", json!([]));
    }
    unset(board, "checker");
    if stale_schema(board) {
        migrate_to_2(board);
    }
    convert_agents(board);
    migrate_lead::migrate(board);
}
