//! The `worktree-link` and `worktree-keep` ops: the owner links a worktree to an open item (or
//! unlinks it with a `null` item) and keeps a stale worktree for a while, as `coord worktree link`,
//! `unlink` and `keep` do. The worktree is `target.id`; the payload is in `set`.

use argus_core::clock::{iso_from_ms, now_iso, parse_ms};
use serde_json::{Value, json};

use super::prelude::*;
use crate::json::{doc, map_mut, nullish, text};
use crate::limits::require_text_max;
use crate::served::closure;

const LINKS: &str = "worktreeLinks";
const KEEPS: &str = "worktreeKeeps";
const DAY_MS: i64 = 86_400_000;
const DEFAULT_KEEP_DAYS: i64 = 7;
const MAX_KEEP_DAYS: i64 = 365;
const NOTE_MAX: usize = 300;
const ID_MAX: usize = 40;

/// The worktree the op is about: a short id of letters, digits, `-` and `_`.
fn worktree_of(op: &Value) -> Result<String, BoardError> {
    let id = text(&op["target"], "id").unwrap_or("").trim();
    let clean = !id.is_empty()
        && id.len() <= ID_MAX
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if clean {
        Ok(id.to_owned())
    } else {
        Err(refuse("target.id must be the id of a worktree."))
    }
}

/// The open item a worktree may be linked to.
fn check_item(board: &Board, item: &str) -> Result<(), BoardError> {
    let Some(found) = doc(board, "items", item) else {
        return Err(refuse(format!(
            "no item \"{item}\" on this board: a worktree is linked to a work item (I-n)."
        )));
    };
    match closure(found) {
        Some(why) => Err(refuse(format!(
            "{item} is {why}: link the worktree to an open item."
        ))),
        None => Ok(()),
    }
}

fn unlink(board: &mut Board, who: &Who, id: &str) -> Result<OpResult, BoardError> {
    let Some(gone) = map_mut(board, LINKS).shift_remove(id) else {
        return Err(refuse(format!("worktree {id} has no link made by hand.")));
    };
    let item = nullish(&gone, "item").unwrap_or_default();
    let line = format!("{} unlinked worktree {id} from {item}", who.label());
    audit(board, &who.actor_of(), &line);
    Ok(OpResult::applied(format!("worktree {id} unlinked")))
}

/// Link a worktree to an open item, or unlink it when `set.item` is `null`.
pub(crate) fn handle_link(
    board: &mut Board,
    op: &Value,
    who: &Who,
) -> Result<OpResult, BoardError> {
    let id = worktree_of(op)?;
    let wanted = &op["set"]["item"];
    if wanted.is_null() {
        return unlink(board, who, &id);
    }
    let Some(item) = wanted.as_str().map(str::trim) else {
        return Err(refuse("set.item must be an item id (I-n) or null."));
    };
    check_item(board, item)?;
    let rec = json!({ "item": item, "by": who.actor_of(), "at": now_iso() });
    map_mut(board, LINKS).insert(id.clone(), rec);
    let line = format!("{} linked worktree {id} to {item}", who.label());
    audit(board, &who.actor_of(), &line);
    Ok(OpResult::applied(format!("worktree {id} linked to {item}")))
}

fn days_of(set: &Value) -> Result<i64, BoardError> {
    let days = &set["days"];
    if days.is_null() {
        return Ok(DEFAULT_KEEP_DAYS);
    }
    match days.as_i64() {
        Some(n) if (1..=MAX_KEEP_DAYS).contains(&n) => Ok(n),
        _ => Err(refuse(
            "set.days takes a whole number of days from 1 to 365.",
        )),
    }
}

/// Forgets the keeps that ended more than 30 days ago.
pub fn forget_old_keeps(board: &mut Board, now_ms: i64) {
    let keeps = map_mut(board, KEEPS);
    let old: Vec<String> = keeps
        .iter()
        .filter(|(_, k)| {
            nullish(k, "until")
                .and_then(|t| parse_ms(&t))
                .is_some_and(|until| until < now_ms - 30 * DAY_MS)
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in old {
        keeps.shift_remove(&id);
    }
}

fn clear_keep(board: &mut Board, who: &Who, id: &str) -> Result<OpResult, BoardError> {
    if map_mut(board, KEEPS).shift_remove(id).is_none() {
        return Err(refuse(format!("worktree {id} is not kept.")));
    }
    let line = format!("{} cleared the keep of worktree {id}", who.label());
    audit(board, &who.actor_of(), &line);
    Ok(OpResult::applied(format!("keep of worktree {id} cleared")))
}

/// Keep a worktree: its stale signal is off for `set.days` days (default 7), with the note `set.note`;
/// `set.clear: true` ends the keep.
pub(crate) fn handle_keep(
    board: &mut Board,
    op: &Value,
    who: &Who,
) -> Result<OpResult, BoardError> {
    let id = worktree_of(op)?;
    let at = now_iso();
    let now_ms = parse_ms(&at).unwrap_or_default();
    forget_old_keeps(board, now_ms);
    let set = &op["set"];
    if set["clear"] == true {
        return clear_keep(board, who, &id);
    }
    let days = days_of(set)?;
    let note = require_text_max("note", &set["note"], NOTE_MAX)?;
    let path = board[KEEPS][id.as_str()]["path"].clone();
    let rec = json!({
        "id": id, "path": path, "until": iso_from_ms(now_ms + days * DAY_MS),
        "note": note, "by": who.actor_of(), "at": at,
    });
    map_mut(board, KEEPS).insert(id.clone(), rec);
    let line = format!("{} kept worktree {id} for {days} days: {note}", who.label());
    audit(board, &who.actor_of(), &line);
    Ok(OpResult::applied(format!(
        "worktree {id} kept for {days} days"
    )))
}
