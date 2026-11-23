//! The config a board runs when it sets none: the four columns of before, with the quiet hours of
//! the stale sweep (to do 24 h, doing 4 h, review 6 h) as dwell rules.

use serde_json::{Value, json};

use super::types::{Alert, Column, KanbanConfig, Match, Rule};
use super::validation::parse;
use crate::Board;

const HOUR_MS: i64 = 3_600_000;
const REPEAT_MS: i64 = 12 * HOUR_MS;

fn column(id: &str, name: &str, lanes: &[&str], rules: &[&str]) -> Column {
    Column {
        id: id.to_owned(),
        name: name.to_owned(),
        accent: None,
        matches: Match {
            lanes: lanes.iter().map(|l| (*l).to_owned()).collect(),
            stages: Vec::new(),
        },
        limit: None,
        rules: rules.iter().map(|r| (*r).to_owned()).collect(),
    }
}

fn stale(lane: &str, hours: i64) -> Rule {
    Rule {
        id: format!("stale-{lane}"),
        kind: "dwell".to_owned(),
        params: json!({"ms": hours * HOUR_MS}),
        mode: None,
        alert: Some(Alert {
            to: "item-thread".to_owned(),
            wake: true,
            repeat_ms: REPEAT_MS,
        }),
    }
}

/// The default config: to do, doing, review and done (done also takes dropped).
#[must_use]
pub fn default_config() -> KanbanConfig {
    KanbanConfig {
        columns: vec![
            column("todo", "To do", &["todo"], &["stale-todo"]),
            column("doing", "Doing", &["doing"], &["stale-doing"]),
            column("review", "Review", &["review"], &["stale-review"]),
            column("done", "Done", &["done", "dropped"], &[]),
        ],
        rules: vec![stale("todo", 24), stale("doing", 4), stale("review", 6)],
    }
}

/// The config a board runs: its own `kanban`, or the default when it has none or a damaged one.
#[must_use]
pub fn effective(board: &Board) -> KanbanConfig {
    match board.get("kanban") {
        Some(value) if !value.is_null() => parse(value).unwrap_or_else(|_| default_config()),
        _ => default_config(),
    }
}

/// The config as the page document `meta/kanban`.
#[must_use]
pub fn export(board: &Board) -> Value {
    serde_json::to_value(effective(board)).unwrap_or(Value::Null)
}
