//! The soft ops: archive, unarchive, delete (to the trash) and restore. Nothing is lost; only
//! `purge` removes records for good.

use argus_core::clock::now_iso;
use serde_json::Value;

use super::announce::{announce, refresh_needs_human};
use super::authorize::{Check, authorize, owner_of};
use super::result::OpResult;
use super::target::target_of;
use super::who::Who;
use crate::Board;
use crate::agents::is_live;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, doc_mut, set, truthy, unset};
use crate::limits::optional_text;
use crate::pause::{archive_thread, unarchive_thread};
use crate::store::stamp;

/// What a soft op does, by its kind.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Archive,
    Unarchive,
    Delete,
    Restore,
}

impl Kind {
    fn of(name: &str) -> Option<Kind> {
        match name {
            "archive" => Some(Kind::Archive),
            "unarchive" => Some(Kind::Unarchive),
            "delete" => Some(Kind::Delete),
            "restore" => Some(Kind::Restore),
            _ => None,
        }
    }

    fn verb(self) -> &'static str {
        match self {
            Kind::Archive => "archive",
            Kind::Unarchive => "unarchive",
            Kind::Delete => "rm",
            Kind::Restore => "restore",
        }
    }

    fn past(self) -> &'static str {
        match self {
            Kind::Archive => "archived",
            Kind::Unarchive => "unarchived",
            Kind::Delete => "moved to the trash",
            Kind::Restore => "restored",
        }
    }
}

fn thread_soft(board: &mut Board, kind: Kind, id: &str, actor: &str) -> Result<(), BoardError> {
    match kind {
        Kind::Delete | Kind::Restore => Err(refuse("a thread is removed only with purge.")),
        Kind::Archive if archive_thread(board, id, actor)? => Ok(()),
        Kind::Archive => Err(refuse(format!("{id} is already archived."))),
        Kind::Unarchive if unarchive_thread(board, id) => Ok(()),
        Kind::Unarchive => Err(refuse(format!("{id} is not archived."))),
    }
}

fn record_soft(
    d: &mut Value,
    kind: Kind,
    id: &str,
    (actor, reason): (&str, Option<&str>),
) -> Result<(), BoardError> {
    match kind {
        Kind::Archive if truthy(d, "archivedAt") => {
            Err(refuse(format!("{id} is already archived.")))
        }
        Kind::Archive => {
            set(d, "archivedAt", now_iso());
            set(d, "archivedBy", actor);
            Ok(())
        }
        Kind::Unarchive if !truthy(d, "archivedAt") => {
            Err(refuse(format!("{id} is not archived.")))
        }
        Kind::Unarchive => {
            unset(d, "archivedAt");
            unset(d, "archivedBy");
            Ok(())
        }
        Kind::Delete => {
            set(d, "deletedAt", now_iso());
            set(d, "deletedBy", actor);
            set(d, "deleteReason", reason);
            Ok(())
        }
        Kind::Restore if !truthy(d, "deletedAt") => {
            Err(refuse(format!("{id} is not in the trash.")))
        }
        Kind::Restore => {
            unset(d, "deletedAt");
            unset(d, "deletedBy");
            unset(d, "deleteReason");
            Ok(())
        }
    }
}

fn change(
    board: &mut Board,
    (coll, id): (&str, &str),
    kind: Kind,
    (actor, reason): (&str, Option<&str>),
) -> Result<(), BoardError> {
    if coll == "threads" {
        return thread_soft(board, kind, id, actor);
    }
    let Some(d) = doc_mut(board, coll, id) else {
        return Err(refuse("gone"));
    };
    record_soft(d, kind, id, (actor, reason))
}

/// Apply an `archive`, `unarchive`, `delete` or `restore` op.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let kind = op["kind"]
        .as_str()
        .and_then(Kind::of)
        .ok_or_else(|| refuse("unknown op"))?;
    let (coll, id) = target_of(board, op, kind == Kind::Restore)?;
    let actor = who.actor_of();
    let current = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    authorize(who, &Check::new(kind.verb(), &coll, Some(&current)))?;
    let reason = optional_text("reason", &op["reason"])?;
    let was_live = coll == "agents" && is_live(&current);
    change(board, (&coll, &id), kind, (&actor, reason.as_deref()))?;
    if was_live && matches!(kind, Kind::Archive | Kind::Delete) {
        if let Some(d) = doc_mut(board, &coll, &id) {
            set(d, "status", "stopped");
            set(d, "finishedAt", now_iso());
        }
    }
    stamp(board, &coll, &id, &actor)?;
    if coll == "actions" {
        let thread = doc(board, &coll, &id)
            .map(|d| display(&d["thread"]))
            .unwrap_or_default();
        refresh_needs_human(board, &thread);
    }
    let updated = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    let because = reason.map_or_else(String::new, |r| format!(" ({r})"));
    let said = format!("{} {id}{because}", kind.past());
    let notify = announce(board, who, &[owner_of(&coll, &updated)], &said, Some(&id));
    Ok(OpResult::applied(format!("{id} {}", kind.past())).with_notify(notify))
}
