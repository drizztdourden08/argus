//! Which column an item sits in, and how many live items a column holds.

use serde_json::Value;

use crate::Board;
use crate::json::{section, truthy};
use crate::kanban_config::{Column, KanbanConfig};

/// Whether the item is on the board: not in the trash and not archived.
fn is_live(item: &Value) -> bool {
    !truthy(item, "deletedAt") && !truthy(item, "archivedAt")
}

/// Whether the column takes an item of this status and stage.
fn takes(column: &Column, item: &Value) -> bool {
    let lane = item["status"].as_str().unwrap_or_default();
    if !column.matches.lanes.iter().any(|l| l == lane) {
        return false;
    }
    let stages = &column.matches.stages;
    let stage = item["stage"].as_str().unwrap_or_default();
    stages.is_empty() || stages.iter().any(|s| s == stage)
}

/// The column the item lands in: the first whose match takes it.
pub(super) fn column_of<'a>(config: &'a KanbanConfig, item: &Value) -> Option<&'a Column> {
    config.columns.iter().find(|c| takes(c, item))
}

/// How many live items the column holds, each item counted in the first column that takes it.
pub(super) fn count_in(board: &Board, config: &KanbanConfig, column: &Column) -> usize {
    section(board, "items")
        .values()
        .filter(|item| is_live(item))
        .filter(|item| column_of(config, item).is_some_and(|c| c.id == column.id))
        .count()
}
