//! `dwell`: an item sat in the column longer than `params.ms`. The clock starts when the item entered
//! its lane and restarts when anything touches the item (its tests, to-dos, sub-agents, entries), as the
//! stale reminders always did. An item that waits on someone else is never a break.

use argus_board::kanban_config::Rule;

use super::column::entered_at;
use super::{Ctx, Hit};
use crate::board::ItemView;
use crate::item_wakes::stale::{last_change, waiting};

pub(super) fn check(ctx: &Ctx<'_>, rule: &Rule, item: &ItemView) -> Option<Hit> {
    let limit = rule.params["ms"].as_i64()?;
    let last = last_change(ctx.board, item).max(entered_at(ctx.board, item));
    if last == 0 || ctx.now - last < limit || waiting(ctx.board, item) {
        return None;
    }
    Some(Hit {
        age: ctx.now - last,
        note: String::new(),
    })
}
