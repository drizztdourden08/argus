//! The limit of a column: the `limit` rule and the column's own `limit`. Only the `warn` and
//! `refuse` modes act when an item is written; `alert` is for the sweep.

use super::column::count_in;
use crate::Board;
use crate::kanban_config::{Column, KanbanConfig};

/// The sentence of the break, or `None` while the column holds no more than its `max`.
pub(super) fn breach(board: &Board, config: &KanbanConfig, column: &Column) -> Option<String> {
    let max = column.limit.as_ref()?.max;
    let held = count_in(board, config, column);
    let over = i64::try_from(held).is_ok_and(|n| n > max);
    over.then(|| format!("{} holds {held} items, the limit is {max}.", column.name))
}
