//! Closing the builds of an item that closed, and bringing them back when it reopens. Board
//! fields only: the names are those a hand retire writes (`retiredAt`, `retiredBy`, `retireNote`,
//! `current`), plus `retiredByItem`, which remembers the item so a reopen restores these builds
//! and never one retired by hand. A build made for a test run serves the run (`for`), so it
//! closes when the run is done or dropped (passed or abandoned).

use std::collections::BTreeSet;

use serde_json::Value;

use super::master::is_master;
use super::play::{drop_play, restore_play};
use super::state::{for_of, is_retired};
use crate::Board;
use crate::json::{doc_mut, section, set, truthy, unset};
use crate::store::{LogLine, append_log, touch};
use crate::text::clip_text;

const AUDIT_LINE: usize = 120;

/// The names of the launchers whose `for` is `item` and that satisfy `keep`.
fn serving(board: &Board, item: &str, keep: impl Fn(&Value) -> bool) -> Vec<String> {
    section(board, "launchers")
        .iter()
        .filter(|(_, rec)| for_of(rec) == Some(item) && keep(rec))
        .map(|(name, _)| name.clone())
        .collect()
}

fn audit(board: &mut Board, who: &str, text: &str) {
    let line = clip_text(text, AUDIT_LINE);
    append_log(
        board,
        "audit",
        &LogLine {
            thread: who,
            item: None,
            text: &line,
        },
    );
}

/// Retires every live build that serves `item`, in this write: `retiredAt`, `retiredBy`,
/// `retiredByItem`, `current` false, and the Play definition dropped. The master build stays.
/// Returns the names closed.
pub fn close_builds(board: &mut Board, item: &str, who: &str, at: &str) -> Vec<String> {
    let names: Vec<String> = serving(board, item, |rec| !is_retired(rec))
        .into_iter()
        .filter(|name| !is_master(board, name))
        .collect();
    for name in &names {
        if let Some(rec) = doc_mut(board, "launchers", name) {
            set(rec, "retiredAt", at);
            set(rec, "retiredBy", who);
            set(rec, "retireNote", format!("its item {item} closed"));
            set(rec, "retiredByItem", item);
            set(rec, "current", false);
            set(rec, "updatedAt", at);
            set(rec, "by", who);
        }
        drop_play(board, name);
        touch(board, "launchers", name);
    }
    if !names.is_empty() {
        audit(
            board,
            who,
            &format!("closed {} with {item}", names.join(", ")),
        );
    }
    names
}

/// Brings back the builds that closing `item` retired. A build retired by hand stays
/// retired. Returns the names restored.
pub fn reopen_builds(board: &mut Board, item: &str, who: &str, at: &str) -> Vec<String> {
    let names = serving(board, item, |rec| {
        is_retired(rec) && rec["retiredByItem"].as_str() == Some(item)
    });
    for name in &names {
        if let Some(rec) = doc_mut(board, "launchers", name) {
            for field in ["retiredAt", "retiredBy", "retireNote"] {
                set(rec, field, Value::Null);
            }
            unset(rec, "retiredByItem");
            set(rec, "updatedAt", at);
            set(rec, "by", who);
        }
        restore_play(board, name);
        touch(board, "launchers", name);
    }
    if !names.is_empty() {
        audit(
            board,
            who,
            &format!("brought back {} ({item} reopened)", names.join(", ")),
        );
    }
    names
}

/// The builds that are closed because their item closed: what to compare before and after an op
/// to tell which builds it closed or brought back.
#[must_use]
pub fn closed_by_items(board: &Board) -> BTreeSet<String> {
    section(board, "launchers")
        .iter()
        .filter(|(_, rec)| truthy(rec, "retiredByItem") && is_retired(rec))
        .map(|(name, _)| name.clone())
        .collect()
}
