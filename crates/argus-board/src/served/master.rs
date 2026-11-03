//! The master build: the one build every board has, which serves no item and never closes with
//! one. It is the launcher with `master: true`; a board not yet migrated has the launcher named
//! `master` instead, which counts until some launcher carries the flag.

use argus_core::clock::now_iso;
use serde_json::Value;

use super::state::is_retired;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc, doc_mut, section, set, unset};
use crate::launchers::MASTER;
use crate::store::touch;

/// The launchers that carry the master flag.
fn flagged(board: &Board) -> Vec<(&String, &Value)> {
    section(board, "launchers")
        .iter()
        .filter(|(_, rec)| rec["master"] == true)
        .collect()
}

/// The name of the live master build, `None` for a board without one.
#[must_use]
pub fn master_name(board: &Board) -> Option<String> {
    let flags = flagged(board);
    if flags.is_empty() {
        let legacy = doc(board, "launchers", MASTER)?;
        return (!is_retired(legacy)).then(|| MASTER.to_owned());
    }
    flags
        .into_iter()
        .find(|(_, rec)| !is_retired(rec))
        .map(|(name, _)| name.clone())
}

/// Whether the launcher `name` is the master build, live or not.
#[must_use]
pub fn is_master(board: &Board, name: &str) -> bool {
    let flags = flagged(board);
    if flags.is_empty() {
        name == MASTER && doc(board, "launchers", MASTER).is_some()
    } else {
        flags.iter().any(|(flag, _)| flag.as_str() == name)
    }
}

/// Refuses making `name` the master build while another live build is. The role moves with
/// [`hand_over_master`], never by flagging a second one.
///
/// # Errors
/// Refused with the name of the build that already is the master.
pub fn check_make_master(board: &Board, name: &str) -> Result<(), BoardError> {
    match master_name(board) {
        Some(have) if have != name => Err(refuse(format!(
            "{have} is already the master build of this board, and a board has one. \
             Hand the role over with coord launcher set {name} --master instead."
        ))),
        _ => Ok(()),
    }
}

/// Refuses retiring, removing or unflagging the master build: a board always has one.
///
/// # Errors
/// Refused when `name` is the master build.
pub fn check_not_last_master(board: &Board, name: &str) -> Result<(), BoardError> {
    if !is_master(board, name) {
        return Ok(());
    }
    Err(refuse(format!(
        "{name} is the master build of this board, and a board always has one. \
         Hand the role to another build first (coord launcher set <name> --master)."
    )))
}

/// Gives the master role to the live build `to`, taking it from the build that had it. Returns
/// the name of the previous master, if any.
///
/// # Errors
/// Refused when `to` is not a live launcher.
pub fn hand_over_master(
    board: &mut Board,
    to: &str,
    by: &str,
) -> Result<Option<String>, BoardError> {
    match doc(board, "launchers", to) {
        Some(rec) if !is_retired(rec) => {}
        _ => return Err(refuse(format!("no live build \"{to}\" to make master."))),
    }
    let before = master_name(board);
    let at = now_iso();
    if let Some(old) = before.as_deref().filter(|old| *old != to) {
        mark(board, old, false, (by, &at));
    }
    mark(board, to, true, (by, &at));
    Ok(before.filter(|old| old != to))
}

/// Sets or clears the flag on one launcher.
fn mark(board: &mut Board, name: &str, on: bool, (by, at): (&str, &str)) {
    if let Some(rec) = doc_mut(board, "launchers", name) {
        if on {
            set(rec, "master", true);
        } else {
            unset(rec, "master");
        }
        set(rec, "updatedAt", at);
        set(rec, "by", by);
    }
    touch(board, "launchers", name);
}
