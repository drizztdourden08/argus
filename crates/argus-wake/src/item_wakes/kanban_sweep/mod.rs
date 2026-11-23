//! The Kanban sweep: the alert rules of a board's Kanban config, run against its items. The config in
//! use is the board's own `kanban`, or the default (a `dwell` rule per lane that reproduces the stale
//! reminders: to do 24 h, doing 4 h, review 6 h, told to the item thread every 12 h).
//!
//! One module per rule kind ([`dwell`], [`assignee_gone`], [`over_limit`], [`unassigned`]); [`config`]
//! picks the config, [`route`] finds who hears of a break and [`words`] says it. A finding is only a
//! break; whether and when it is told again is the business of the tick that owns the repeat state.

mod assignee_gone;
mod column;
mod config;
mod dwell;
mod over_limit;
mod route;
mod unassigned;
mod words;

use argus_board::kanban_config::{Alert, Column, KanbanConfig, Rule};

pub(super) use route::{Group, groups};
pub(super) use words::alert_text;

use super::ItemWakeConfig;
use super::stale::Stale;
use crate::board::{BoardView, ItemView};
use crate::source::BoardSource;

/// The recipient of an alert that goes to the owner.
pub(super) const OWNER: &str = "owner";

/// The sweep rule kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Dwell,
    AssigneeGone,
    OverLimit,
    Unassigned,
}

impl Kind {
    fn of(name: &str) -> Option<Self> {
        match name {
            "dwell" => Some(Self::Dwell),
            "assignee-gone" => Some(Self::AssigneeGone),
            "over-limit" => Some(Self::OverLimit),
            "unassigned" => Some(Self::Unassigned),
            _ => None,
        }
    }

    /// The name of the kind, as in the config.
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::Dwell => "dwell",
            Self::AssigneeGone => "assignee-gone",
            Self::OverLimit => "over-limit",
            Self::Unassigned => "unassigned",
        }
    }
}

/// One break of a rule and the thread (or `owner`) that hears of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Finding {
    pub(super) stale: Stale,
    pub(super) kind: Kind,
    /// What follows the id in the line told (empty for a dwell, which the stale words say).
    pub(super) note: String,
    pub(super) wake: bool,
    pub(super) repeat_ms: i64,
}

/// What a kind found: how long it has gone on and the words for the line.
pub(super) struct Hit {
    pub(super) age: i64,
    pub(super) note: String,
}

/// What a rule reads.
pub(super) struct Ctx<'a> {
    pub(super) now: i64,
    pub(super) scope: &'a str,
    pub(super) board: &'a BoardView,
}

impl Ctx<'_> {
    fn finding(
        &self,
        thread: String,
        id: &str,
        status: &str,
        kind: Kind,
        hit: Hit,
        alert: &Alert,
    ) -> Finding {
        Finding {
            stale: Stale {
                scope: self.scope.to_owned(),
                thread,
                id: id.to_owned(),
                status: status.to_owned(),
                quiet: hit.age,
            },
            kind,
            note: hit.note,
            wake: alert.wake,
            repeat_ms: alert.repeat_ms,
        }
    }
}

fn item_hit(kind: Kind, ctx: &Ctx<'_>, rule: &Rule, item: &ItemView) -> Option<Hit> {
    match kind {
        Kind::Dwell => dwell::check(ctx, rule, item),
        Kind::AssigneeGone => assignee_gone::check(ctx, item),
        Kind::Unassigned => unassigned::check(ctx, rule, item),
        Kind::OverLimit => None,
    }
}

fn rule_of<'a>(config: &'a KanbanConfig, id: &str) -> Option<&'a Rule> {
    config.rules.iter().find(|r| r.id == id)
}

fn item_findings(
    ctx: &Ctx<'_>,
    config: &KanbanConfig,
    column: &Column,
    item: &ItemView,
    out: &mut Vec<Finding>,
) {
    for rule in column.rules.iter().filter_map(|id| rule_of(config, id)) {
        let (Some(kind), Some(alert)) = (Kind::of(&rule.kind), &rule.alert) else {
            continue;
        };
        let Some(hit) = item_hit(kind, ctx, rule, item) else {
            continue;
        };
        let Some(thread) = route::recipient(ctx, item, &alert.to) else {
            continue;
        };
        if route::paused(ctx.board, &thread) {
            continue;
        }
        let (id, status) = (item.rec.id.as_str(), item.status.as_str());
        out.push(ctx.finding(thread, id, status, kind, hit, alert));
    }
}

fn board_findings(now: i64, scope: &str, board: &BoardView, base: &ItemWakeConfig) -> Vec<Finding> {
    let config = config::in_use(board, base);
    let ctx = Ctx { now, scope, board };
    let mut out = Vec::new();
    for item in board.items.values().filter(|i| i.rec.is_live()) {
        for column in config
            .columns
            .iter()
            .filter(|c| column::holds(c, board, item))
        {
            item_findings(&ctx, &config, column, item, &mut out);
        }
    }
    for column in &config.columns {
        over_limit::findings(&ctx, &config, column, &mut out);
    }
    out
}

/// Every break of every board: the items first, in board order, then the columns over their limit.
pub(super) fn sweep(config: &ItemWakeConfig, now: i64, boards: &dyn BoardSource) -> Vec<Finding> {
    boards
        .scopes()
        .iter()
        .filter_map(|scope| {
            let board = boards.board(scope)?;
            Some(board_findings(now, scope, board.as_ref(), config))
        })
        .flatten()
        .collect()
}

/// The dwell findings that go to a thread: the stale items of the reminders and of the digest.
pub(super) fn stale_of(findings: &[Finding]) -> Vec<Stale> {
    findings
        .iter()
        .filter(|f| f.kind == Kind::Dwell && f.stale.thread != OWNER)
        .map(|f| f.stale.clone())
        .collect()
}
