//! What the owner posts about a board item from the Argus page: `edit` of the goal and the note,
//! `archive` and `unarchive`. Anything else is refused; a board item is history, never deleted.

use std::path::Path;

use serde_json::{Map, Value, json};

use super::COLL;
use super::reconcile::reconcile_as;
use crate::Board;
use crate::json::{display, doc, map_mut, set, strict_eq, unset};
use crate::ops::OpResult;
use crate::store::{stamp, touch};

const OWNER: &str = "human";
const EDITABLE: [&str; 2] = ["goal", "note"];

fn edit(p: &mut Board, id: &str, op: &Value) -> OpResult {
    let Some(changes) = op["set"].as_object().filter(|s| !s.is_empty()) else {
        return OpResult::refused("nothing to change.");
    };
    for (field, value) in changes {
        if !EDITABLE.contains(&field.as_str()) {
            return OpResult::refused(format!("{field} cannot be edited on a board item."));
        }
        if !value.is_string() {
            return OpResult::refused(format!("{field} must be text."));
        }
    }
    if let Some(item) = p.get_mut(COLL).and_then(|c| c.get_mut(id)) {
        for (field, value) in changes {
            set(item, field, value.clone());
        }
    }
    let _ = stamp(p, COLL, id, OWNER);
    OpResult::applied(format!("{id} edited"))
}

fn retire(home: &Path, p: &mut Board, key: &str, retired: bool) {
    let row = map_mut(p, "projects")
        .entry(key.to_owned())
        .or_insert_with(|| json!({ "key": key }));
    if retired {
        set(row, "retired", true);
    } else {
        unset(row, "retired");
    }
    reconcile_as(home, p, OWNER);
}

fn dispatch(home: &Path, p: &mut Board, op: &Value, id: &str) -> OpResult {
    let key = doc(p, COLL, id)
        .map(|d| display(&d["key"]))
        .unwrap_or_default();
    match op["kind"].as_str() {
        Some("edit") => edit(p, id, op),
        Some(kind @ ("archive" | "unarchive")) => {
            retire(home, p, &key, kind == "archive");
            OpResult::applied(format!("{id} {kind}d"))
        }
        _ => OpResult::refused("a board item is archived, never deleted."),
    }
}

/// Applies one owner op on a board item of the portfolio. Refuses an author other than the
/// owner, a target that is not a board item, a field other than `goal` and `note`, and a kind
/// other than `edit`, `archive` and `unarchive`. Answers a conflict when `baseRev` is not the
/// current revision. Archive and unarchive change the project's `retired` flag and reconcile in
/// the same write.
pub fn apply_owner_op(home: &Path, p: &mut Board, op: &Value) -> OpResult {
    if op["author"] != OWNER {
        return OpResult::refused("only the owner changes a board item.");
    }
    let id = display(&op["target"]["id"]);
    if op["target"]["c"] != COLL {
        return OpResult::refused("the target is not a board item.");
    }
    let Some(item) = doc(p, COLL, &id) else {
        return OpResult::refused(format!("{id} is gone."));
    };
    let base = &op["baseRev"];
    if !base.is_null() && !strict_eq(base, &item["rev"]) {
        let current = Map::from_iter([
            ("goal".to_owned(), item["goal"].clone()),
            ("note".to_owned(), item["note"].clone()),
        ]);
        return OpResult::conflict(item, current);
    }
    dispatch(home, p, op, &id)
}

/// Applies a post aimed at a board item and keeps it with its `status` (`applied`, `refused` or
/// `conflict`) and `outcome`. Returns the status.
pub(super) fn import(home: &Path, p: &mut Board, id: &str, mut entry: Value) -> String {
    let result = apply_owner_op(home, p, &entry);
    let status = result.status.as_str();
    set(&mut entry, "status", status);
    set(&mut entry, "outcome", result.outcome.as_str());
    if let Some(current) = result.current {
        set(&mut entry, "current", current);
    }
    map_mut(p, "entries").insert(id.to_owned(), entry);
    touch(p, "entries", id);
    status.to_owned()
}
