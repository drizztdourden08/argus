//! Which items are stale: quiet for too long and not waiting on someone else.

use std::cmp::Ordering;

use crate::blockers;
use crate::board::{BoardView, ItemView, LIVE_AGENT, Record};
use crate::policy::coordinator_in;
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

/// The thread that hears about something on an item owned by `owner`: that thread while it is live, else the
/// project's lead.
pub(crate) fn target_for(board: &BoardView, owner: &str) -> Option<String> {
    let live = Some(owner)
        .filter(|id| !id.is_empty())
        .and_then(|id| board.threads.get(id))
        .filter(|t| t.is_live());
    match live {
        Some(thread) => Some(thread.id.some().unwrap_or(owner).to_owned()),
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
pub(crate) fn last_change(board: &BoardView, item: &ItemView) -> i64 {
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

/// Is the item waiting on someone else, so that a reminder to its thread would be wrong?
pub(super) fn waiting(board: &BoardView, item: &ItemView) -> bool {
    if blockers::is_blocked(board, item) {
        return true;
    }
    let id = item.rec.id.as_str();
    let todo = board
        .actions
        .values()
        .any(|a| a.rec.is_live_on(id) && a.status.is("pending"));
    todo || handed_run(board, id) || has_live_agent(board, id)
}

/// Is the item a test run handed to the owner (ready and assigned to him), or the plan of one?
fn handed_run(board: &BoardView, id: &str) -> bool {
    board.json()["items"].as_object().is_some_and(|items| {
        items.values().any(|run| {
            run["type"] == "test-run"
                && run["stage"] == "ready"
                && run["assignee"] == "owner"
                && run["deletedAt"].is_null()
                && (run["id"] == id || run["fields"]["plan"] == id)
        })
    })
}

/// Does a sub-agent still run on the item, or hold it as its assignee?
pub(crate) fn has_live_agent(board: &BoardView, item_id: &str) -> bool {
    board
        .agents
        .values()
        .any(|a| a.rec.is_live_on(item_id) && a.status.is_any(&LIVE_AGENT))
        || held_by_live_agent(board, item_id)
}

/// Is the item's assignee `agent:A-n` of an agent that is still live?
fn held_by_live_agent(board: &BoardView, item_id: &str) -> bool {
    board
        .items
        .get(item_id)
        .and_then(|item| item.assignee.as_str().strip_prefix("agent:"))
        .and_then(|id| board.agents.get(id))
        .is_some_and(|a| a.rec.is_live() && a.status.is_any(&LIVE_AGENT))
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

/// The items quiet for at least `escalate_ms`, by project (the portfolio itself has no digest).
pub(super) fn by_project(
    stale: &[Stale],
    overlord: &str,
    escalate_ms: i64,
) -> Vec<(String, Vec<Stale>)> {
    group(
        stale
            .iter()
            .filter(|s| s.scope != overlord && s.quiet >= escalate_ms)
            .map(|s| (s.scope.clone(), s.clone())),
    )
}
