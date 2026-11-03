//! The `launcher-link` and `launcher-retire` ops: the owner links a build (a launcher) to the open
//! item it serves, unlinks it (`set.for: null`, the master build only: every other build needs its
//! item), or retires it, as `coord launcher link`, `unlink` and `retire` do. The build is
//! `target.id`; the payload is in `set`. The master build is never retired.

use argus_core::clock::now_iso;
use serde_json::Value;

use super::announce::audit;
use super::authorize::{Check, authorize};
use super::result::OpResult;
use super::who::Who;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc, doc_mut, map_mut, set, text, truthy, unset};
use crate::limits::require_text_max;
use crate::served::{check_for, check_not_last_master, for_of, is_master};

const NOTE_MAX: usize = 300;
const PLAY_PREFIX: &str = "launcher-";

/// The build the op is about, live or retired.
fn build_of<'a>(board: &'a Board, op: &Value) -> Result<(String, &'a Value), BoardError> {
    let name = text(&op["target"], "id").unwrap_or("").trim();
    match doc(board, "launchers", name) {
        Some(rec) => Ok((name.to_owned(), rec)),
        None => Err(refuse(format!("no launcher \"{name}\"."))),
    }
}

/// Whether the op asks to drop the link: `set.for` is there and `null`.
fn asks_unlink(op: &Value) -> bool {
    op["set"].get("for") == Some(&Value::Null)
}

fn retired_refusal(name: &str, rec: &Value, hint: &str) -> Result<(), BoardError> {
    if truthy(rec, "retiredAt") {
        return Err(refuse(format!(
            "launcher {name} is retired: bring it back with coord launcher set {name} {hint} instead."
        )));
    }
    Ok(())
}

/// Drop the item a build serves. Only the master build may go without one.
fn unlink(board: &mut Board, who: &Who, name: &str) -> Result<OpResult, BoardError> {
    let rec =
        doc(board, "launchers", name).ok_or_else(|| refuse(format!("no launcher \"{name}\".")))?;
    if truthy(rec, "retiredAt") {
        return Err(refuse(format!("launcher {name} is retired.")));
    }
    let Some(item) = for_of(rec).map(str::to_owned) else {
        return Ok(OpResult::applied(format!("{name} serves no item")));
    };
    check_for(board, is_master(board, name), None)?;
    let actor = who.actor_of();
    if let Some(rec) = doc_mut(board, "launchers", name) {
        set(rec, "for", Value::Null);
        unset(rec, "relink");
        set(rec, "updatedAt", now_iso());
        set(rec, "by", actor.as_str());
    }
    let line = format!("{} unlinked launcher {name} from {item}", who.label());
    audit(board, &actor, &line);
    Ok(OpResult::applied(format!("{name} unlinked from {item}")))
}

/// Link a build to the open item it serves (`set.for`), or unlink it when `set.for` is `null`.
pub(crate) fn handle_link(
    board: &mut Board,
    op: &Value,
    who: &Who,
) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("hand", "threads", None))?;
    let (name, rec) = build_of(board, op)?;
    if asks_unlink(op) {
        return unlink(board, who, &name);
    }
    let item = op["set"]["for"].as_str().map_or("", str::trim);
    retired_refusal(&name, rec, &format!("--for {item}"))?;
    if for_of(rec) == Some(item) {
        return Ok(OpResult::applied(format!("{name} already serves {item}")));
    }
    check_for(board, is_master(board, &name), Some(item))?;
    let actor = who.actor_of();
    if let Some(rec) = doc_mut(board, "launchers", &name) {
        set(rec, "for", item);
        unset(rec, "relink");
        set(rec, "updatedAt", now_iso());
        set(rec, "by", actor.as_str());
    }
    let line = format!("{} linked launcher {name} to {item}", who.label());
    audit(board, &actor, &line);
    Ok(OpResult::applied(format!("{name} serves {item}")))
}

fn retire_record(rec: &mut Value, actor: &str, note: Option<&str>) {
    let at = now_iso();
    set(rec, "retiredAt", at.as_str());
    set(rec, "retiredBy", actor);
    set(rec, "retireNote", note);
    set(rec, "current", false);
    set(rec, "updatedAt", at.as_str());
    set(rec, "by", actor);
    unset(rec, "retiredByBundle");
    unset(rec, "retiredByItem");
}

/// Retire a build with an optional note (`set.note`); its Play definition goes with it.
pub(crate) fn handle_retire(
    board: &mut Board,
    op: &Value,
    who: &Who,
) -> Result<OpResult, BoardError> {
    authorize(who, &Check::new("hand", "threads", None))?;
    let (name, rec) = build_of(board, op)?;
    if truthy(rec, "retiredAt") {
        return Ok(OpResult::applied(format!("{name} is already retired")));
    }
    check_not_last_master(board, &name)?;
    let note = match &op["set"]["note"] {
        Value::Null => None,
        value => Some(require_text_max("note", value, NOTE_MAX)?),
    };
    let actor = who.actor_of();
    if let Some(rec) = doc_mut(board, "launchers", &name) {
        retire_record(rec, &actor, note.as_deref());
    }
    map_mut(board, "runDefs").shift_remove(&format!("{PLAY_PREFIX}{name}"));
    let why = note.map(|n| format!(": {n}")).unwrap_or_default();
    let line = format!("{} retired launcher {name}{why}", who.label());
    audit(board, &actor, &line);
    Ok(OpResult::applied(format!("{name} retired")))
}
