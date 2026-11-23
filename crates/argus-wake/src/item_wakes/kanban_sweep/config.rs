//! The config a board sweeps with: its own `kanban`, or the default built from the stale timing of
//! `config.json`, so a board that sets none keeps the reminders it always had.

use argus_board::kanban_config::{KanbanConfig, default_config, parse};
use serde_json::json;

use crate::board::BoardView;
use crate::item_wakes::ItemWakeConfig;

fn quiet_ms(base: &ItemWakeConfig, rule_id: &str) -> Option<i64> {
    match rule_id {
        "stale-todo" => Some(base.stale_todo_ms),
        "stale-doing" => Some(base.stale_doing_ms),
        "stale-review" => Some(base.stale_review_ms),
        _ => None,
    }
}

fn default_for(base: &ItemWakeConfig) -> KanbanConfig {
    let mut config = default_config();
    for rule in &mut config.rules {
        if let Some(ms) = quiet_ms(base, &rule.id) {
            rule.params = json!({ "ms": ms });
        }
        if let Some(alert) = &mut rule.alert {
            alert.repeat_ms = base.repeat_ms;
        }
    }
    config
}

/// The config of a board: its own when it holds a valid one, else the default.
pub(super) fn in_use(board: &BoardView, base: &ItemWakeConfig) -> KanbanConfig {
    board
        .json()
        .get("kanban")
        .filter(|value| !value.is_null())
        .and_then(|value| parse(value).ok())
        .unwrap_or_else(|| default_for(base))
}
