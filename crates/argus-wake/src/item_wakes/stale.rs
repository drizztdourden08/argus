//! Which items are stale: quiet for too long and not waiting on someone else.

use std::cmp::Ordering;

use super::ItemWakeConfig;
use crate::board::{BoardView, ItemView, LIVE_AGENT, Record};
use crate::policy::coordinator_in;
use crate::reasons::MAIN_SCOPE;
use crate::source::BoardSource;
use crate::time::ms_or_zero;

/// A stale item and who must be told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stale {
    /// Project key.
    pub scope: String,
    /// The thread to remind.
    pub thread: String,
    /// The item id.
    pub id: String,
    /// `todo`, `doing` or `review`.
    pub status: String,
    /// How long it has been quiet (ms).
    pub quiet: i64,
}

fn limit(config: &ItemWakeConfig, status: &str) -> Option<i64> {
    match status {
        "doing" => Some(config.stale_doing_ms),
        "review" => Some(config.stale_review_ms),
        "todo" => Some(config.stale_todo_ms),
        _ => None,
    }
}

fn target_of(board: &BoardView, item: &ItemView) -> Option<String> {
    let owner = item
        .thread
        .some()
        .and_then(|id| board.threads.get(id))
        .filter(|t| t.is_live());
    match owner {
        Some(thread) => Some(
            thread
                .id
                .some()
                .unwrap_or_else(|| item.thread.as_str())
                .to_owned(),
        ),
        None => coordinator_in(Some(board)),
    }
}

fn touches(rec: &Record, item_id: &str) -> bool {
    rec.item.is(item_id) && rec.deleted_at.some().is_none()
}

fn stamp(rec: &Record) -> i64 {
    ms_or_zero(rec.updated_at.some())
        .max(ms_or_zero(rec.created_at.some()))
        .max(ms_or_zero(rec.finished_at.some()))
}

/// When the item last changed: itself, and anything that touches it.
fn last_change(board: &BoardView, item: &ItemView) -> i64 {
    let id = item.rec.id.as_str();
    let own = ms_or_zero(item.rec.updated_at.some()).max(ms_or_zero(item.rec.created_at.some()));
    let records = board
        .tests
        .values()
        .map(|t| &t.rec)
        .chain(board.actions.values().map(|a| &a.rec))
        .chain(board.agents.values().map(|a| &a.rec))
        .chain(board.entries.values().map(|e| &e.rec));
    records
        .filter(|r| touches(r, id))
        .map(stamp)
        .fold(own, i64::max)
}

fn mine(rec: &Record, item_id: &str) -> bool {
    rec.item.is(item_id) && rec.is_live()
}

/// Is the item waiting on someone else, so that a reminder to its thread would be wrong?
fn waiting(board: &BoardView, item: &ItemView) -> bool {
    let id = item.rec.id.as_str();
    let asks = board.entries.values().any(|e| {
        mine(&e.rec, id)
            && ((e.kind.is("question") && e.status.is("open"))
                || (e.kind.is("decision") && e.status.is("pending")))
    });
    let todo = board
        .actions
        .values()
        .any(|a| mine(&a.rec, id) && a.status.is("pending"));
    let handed = board.handed_build.some().is_some_and(|build| {
        board.tests.values().any(|t| {
            mine(&t.rec, id) && t.mode.is("manual") && t.status.is("pending") && t.build.is(build)
        })
    });
    let agent = board
        .agents
        .values()
        .any(|a| mine(&a.rec, id) && a.status.is_any(&LIVE_AGENT));
    asks || todo || handed || agent
}

fn stale_in(config: &ItemWakeConfig, now: i64, scope: &str, board: &BoardView) -> Vec<Stale> {
    let mut out = Vec::new();
    for item in board.items.values() {
        let Some(limit) = limit(config, item.status.as_str()) else {
            continue;
        };
        if !item.rec.is_live() {
            continue;
        }
        let last = last_change(board, item);
        if last == 0 || now - last < limit {
            continue;
        }
        let Some(thread) = target_of(board, item) else {
            continue;
        };
        if board
            .threads
            .get(&thread)
            .is_some_and(|t| t.status.is("paused"))
            || waiting(board, item)
        {
            continue;
        }
        out.push(Stale {
            scope: scope.to_owned(),
            thread,
            id: item.rec.id.as_str().to_owned(),
            status: item.status.as_str().to_owned(),
            quiet: now - last,
        });
    }
    out
}

/// Every stale item of every board.
pub(super) fn stale_items(
    config: &ItemWakeConfig,
    now: i64,
    boards: &dyn BoardSource,
) -> Vec<Stale> {
    boards
        .scopes()
        .iter()
        .filter_map(|scope| Some(stale_in(config, now, scope, boards.board(scope)?.as_ref())))
        .flatten()
        .collect()
}

fn id_number(id: &str) -> i64 {
    let digits: String = id.chars().filter(char::is_ascii_digit).collect();
    digits.parse().unwrap_or(0)
}

/// Longest quiet first, then the lower id number.
pub(super) fn worst_first(a: &Stale, b: &Stale) -> Ordering {
    b.quiet
        .cmp(&a.quiet)
        .then_with(|| id_number(&a.id).cmp(&id_number(&b.id)))
}

fn group<K: PartialEq>(items: impl IntoIterator<Item = (K, Stale)>) -> Vec<(K, Vec<Stale>)> {
    let mut groups: Vec<(K, Vec<Stale>)> = Vec::new();
    for (key, stale) in items {
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(stale),
            None => groups.push((key, vec![stale])),
        }
    }
    groups
}

/// The stale items grouped by the project and thread to remind, in order of first appearance.
pub(super) fn by_thread(stale: &[Stale]) -> Vec<((String, String), Vec<Stale>)> {
    group(
        stale
            .iter()
            .map(|s| ((s.scope.clone(), s.thread.clone()), s.clone())),
    )
}

/// The items quiet for at least `escalate_ms`, by project (the portfolio itself has no digest).
pub(super) fn by_project(stale: &[Stale], escalate_ms: i64) -> Vec<(String, Vec<Stale>)> {
    group(
        stale
            .iter()
            .filter(|s| s.scope != MAIN_SCOPE && s.quiet >= escalate_ms)
            .map(|s| (s.scope.clone(), s.clone())),
    )
}
