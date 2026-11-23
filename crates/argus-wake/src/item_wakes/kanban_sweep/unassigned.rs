//! `unassigned`: an item had no assignee for longer than `params.ms` in its column.

use argus_board::kanban_config::Rule;
use argus_core::clock::age_text;

use super::column::entered_at;
use super::{Ctx, Hit};
use crate::board::ItemView;

pub(super) fn check(ctx: &Ctx<'_>, rule: &Rule, item: &ItemView) -> Option<Hit> {
    let limit = rule.params["ms"].as_i64()?;
    let since = entered_at(ctx.board, item);
    if item.assignee.some().is_some() || since == 0 || ctx.now - since < limit {
        return None;
    }
    let age = ctx.now - since;
    Some(Hit {
        age,
        note: format!(
            "{}, no assignee for {}",
            item.status.as_str(),
            age_text(age)
        ),
    })
}
