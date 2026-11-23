//! `over-limit`: a column holds more live items than its `limit.max`. A column whose own limit mode is
//! `alert` fires too, to the lead, when no `over-limit` rule names it.

use argus_board::kanban_config::{Alert, Column, KanbanConfig};

use super::{Ctx, Finding, Hit, Kind, column, route, rule_of};

const HOUR_MS: i64 = 3_600_000;

/// The alert of a column in `alert` mode that has no rule: the lead, with a wake, every 12 h.
fn limit_alert() -> Alert {
    Alert {
        to: "lead".to_owned(),
        wake: true,
        repeat_ms: 12 * HOUR_MS,
    }
}

fn alerts_of(config: &KanbanConfig, col: &Column) -> Vec<Alert> {
    let ruled: Vec<Alert> = col
        .rules
        .iter()
        .filter_map(|id| rule_of(config, id))
        .filter(|rule| rule.kind == Kind::OverLimit.key())
        .filter_map(|rule| rule.alert.clone())
        .collect();
    let by_mode = col.limit.as_ref().is_some_and(|l| l.mode == "alert");
    if ruled.is_empty() && by_mode {
        vec![limit_alert()]
    } else {
        ruled
    }
}

fn held(ctx: &Ctx<'_>, col: &Column) -> usize {
    ctx.board
        .items
        .values()
        .filter(|item| item.rec.is_live() && column::holds(col, ctx.board, item))
        .count()
}

pub(super) fn findings(ctx: &Ctx<'_>, config: &KanbanConfig, col: &Column, out: &mut Vec<Finding>) {
    let Some(limit) = &col.limit else {
        return;
    };
    let count = held(ctx, col);
    if i64::try_from(count).is_ok_and(|n| n <= limit.max) {
        return;
    }
    for alert in alerts_of(config, col) {
        let Some(thread) = route::column_recipient(ctx.board, &alert.to) else {
            continue;
        };
        let hit = Hit {
            age: 0,
            note: format!("holds {count} of {}", limit.max),
        };
        out.push(ctx.finding(thread, &col.name, "over", Kind::OverLimit, hit, &alert));
    }
}
