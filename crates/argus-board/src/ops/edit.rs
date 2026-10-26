//! The `edit` op: change the whitelisted fields of one record.

use argus_core::clock::now_iso;
use serde_json::{Map, Value};

use super::announce::{announce, refresh_needs_human};
use super::authorize::{Check, authorize, owner_of};
use super::fields::{EditCtx, check, editable};
use super::result::{OpResult, version_check};
use super::target::target_of;
use super::who::Who;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, doc_mut, find_ids, obj, same, set};
use crate::store::stamp;

/// The fields an edit asks for, refused when there are none.
fn requested(op: &Value) -> Result<&Map<String, Value>, BoardError> {
    match op["set"].as_object() {
        Some(set) if !set.is_empty() => Ok(set),
        _ => Err(refuse("nothing to change.")),
    }
}

fn validate(
    cx: &EditCtx,
    coll: &str,
    set: &Map<String, Value>,
) -> Result<Map<String, Value>, BoardError> {
    let mut next = Map::new();
    for (field, value) in set {
        if !editable(coll, field) {
            return Err(refuse(format!("{field} cannot be edited on {coll}.")));
        }
        next.insert(field.clone(), check(coll, field, value, cx)?);
    }
    Ok(next)
}

fn checked_fields(op: &Value, next: &Map<String, Value>) -> Vec<String> {
    let mut fields: Vec<String> = next.keys().cloned().collect();
    for key in obj(&op["prev"]).keys() {
        if !fields.contains(key) {
            fields.push(key.clone());
        }
    }
    fields
}

fn write_fields(
    board: &mut Board,
    (coll, id): (&str, &str),
    next: &Map<String, Value>,
    fields: &[String],
) {
    let Some(d) = doc_mut(board, coll, id) else {
        return;
    };
    for f in fields {
        set(d, f, next[f].clone());
    }
    if coll == "entries" {
        set(d, "editedAt", now_iso());
    }
    if coll == "actions" && fields.iter().any(|f| f == "status") {
        let done = d["status"] == "done";
        let done_at = if done && !d["doneAt"].is_null() {
            d["doneAt"].clone()
        } else if done {
            Value::String(now_iso())
        } else {
            Value::Null
        };
        set(d, "doneAt", done_at);
        set(
            d,
            "ackedAt",
            if done {
                Value::String(now_iso())
            } else {
                Value::Null
            },
        );
    }
}

fn follow_subject(board: &mut Board, item_id: &str, actor: &str) -> Result<(), BoardError> {
    let subject = doc(board, "items", item_id).map_or(Value::Null, |i| i["subject"].clone());
    for test_id in find_ids(board, "tests", |t| t["item"].as_str() == Some(item_id)) {
        if let Some(t) = doc_mut(board, "tests", &test_id) {
            set(t, "subject", subject.clone());
        }
        stamp(board, "tests", &test_id, actor)?;
    }
    Ok(())
}

fn side_effects(
    board: &mut Board,
    (coll, id): (&str, &str),
    fields: &[String],
    actor: &str,
) -> Result<(), BoardError> {
    stamp(board, coll, id, actor)?;
    if coll == "items" && fields.iter().any(|f| f == "subject") {
        follow_subject(board, id, actor)?;
    }
    if coll == "actions" {
        let thread = doc(board, coll, id).map(|d| display(&d["thread"]));
        refresh_needs_human(board, &thread.unwrap_or_default());
    }
    Ok(())
}

fn said(coll: &str, updated: &Value, id: &str, fields: &[String]) -> String {
    let stop_asked = coll == "agents"
        && updated["status"] == "stop-requested"
        && fields.iter().any(|f| f == "status");
    if stop_asked {
        format!(
            "asked to stop agent {id} ({}); stop it, then run coord agent stop {id}",
            display(&updated["name"])
        )
    } else {
        format!("edited {id} ({})", fields.join(", "))
    }
}

/// Apply an `edit` op: check the caller, validate every field, check the revision the writer
/// saw, apply, stamp, and tell the owning thread.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let (coll, id) = target_of(board, op, false)?;
    let set = requested(op)?;
    let current = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    let names: Vec<String> = set.keys().cloned().collect();
    authorize(
        who,
        &Check::new("set", &coll, Some(&current)).fields(&names),
    )?;
    let cx = EditCtx {
        board,
        doc: &current,
        who,
    };
    let next = validate(&cx, &coll, set)?;
    let checked = checked_fields(op, &next);
    if let Some(collision) = version_check(&current, op, &checked, &next) {
        return Ok(OpResult::conflict(&current, collision));
    }
    let fields: Vec<String> = next
        .keys()
        .filter(|f| !same(current.get(f.as_str()), next.get(f.as_str())))
        .cloned()
        .collect();
    if fields.is_empty() {
        return Ok(OpResult::applied(format!("{id} already up to date")));
    }
    write_fields(board, (&coll, &id), &next, &fields);
    side_effects(board, (&coll, &id), &fields, &who.actor_of())?;
    let updated = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    let owner = owner_of(&coll, &updated);
    let notify = announce(
        board,
        who,
        &[owner],
        &said(&coll, &updated, &id, &fields),
        Some(&id),
    );
    Ok(OpResult::applied(format!("{id} edited ({})", fields.join(", "))).with_notify(notify))
}
