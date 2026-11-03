//! The `build` op: the build handed to the owner. A change at the coordinator's level: the board
//! holds `handedBuild` and `handedAt`, and one audit line records it.

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::prelude::*;
use crate::json::{doc, doc_mut, map_mut, nullish, set, truthy};
use crate::launchers::bundle_status;
use crate::limits::require_text;

/// The bundle record of a build tag, made when a tag is handed without one.
pub fn ensure_build(board: &mut Board, tag: &str, actor: &str, status: &str) {
    let at = now_iso();
    let rec = map_mut(board, "builds")
        .entry(tag.to_owned())
        .or_insert_with(|| {
            json!({
                "tag": tag, "status": "draft", "title": null, "note": null, "launcher": null,
                "thread": null, "createdAt": at, "updatedAt": at, "createdBy": actor,
            })
        });
    set(rec, "status", status);
}

fn check_ready(board: &Board, tag: &str) -> Result<(), BoardError> {
    match doc(board, "builds", tag) {
        Some(rec) if bundle_status(board, tag, rec) == "draft" => Err(refuse(format!(
            "bundle {tag} is a draft (not built yet): coord build ready {tag} once it is built and testable, then hand it."
        ))),
        _ => Ok(()),
    }
}

fn record(board: &mut Board, who: &Who, tag: Option<&str>, was: Option<&str>) {
    if let Some(rec) = tag.and_then(|t| doc_mut(board, "builds", t)) {
        set(rec, "lastHandedAt", now_iso());
    }
    set(board, "handedBuild", tag);
    set(board, "handedAt", tag.map(|_| now_iso()));
    let what = match (tag, was) {
        (Some(t), Some(w)) => format!("handed build {t} (was {w})"),
        (Some(t), None) => format!("handed build {t}"),
        (None, w) => format!("unhanded build {}", w.unwrap_or("null")),
    };
    audit(board, &who.actor_of(), &format!("{} {what}", who.label()));
}

/// Hand a build tag to the owner, or take it back with `null`.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let handed = &op["handed"];
    if !handed.is_null() && !handed.is_string() {
        return Err(refuse("handed must be a build tag or null."));
    }
    let tag = if truthy(op, "handed") {
        Some(require_text("build", handed)?)
    } else {
        None
    };
    let was = nullish(board, "handedBuild");
    if let Some(tag) = &tag {
        check_ready(board, tag)?;
        if doc(board, "builds", tag).is_none() {
            ensure_build(board, tag, &who.actor_of(), "ready");
        }
    }
    if tag == was {
        return Ok(OpResult::applied(tag.as_ref().map_or_else(
            || "no build is handed".to_owned(),
            |t| format!("build {t} is already handed"),
        )));
    }
    record(board, who, tag.as_deref(), was.as_deref());
    Ok(OpResult::applied(match (&tag, &was) {
        (Some(t), _) => format!("build {t} handed"),
        (None, w) => format!("build {} unhanded", w.as_deref().unwrap_or("null")),
    }))
}
