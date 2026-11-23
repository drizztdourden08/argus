//! The two mirrors of a dependency: `waitsOn` on the requester item and `requestedBy` on the
//! assignee item. Only this module writes them. A mirror is a cache of the ledger that carries
//! the other side's title, so a board page never reads another scope. Each field is a list with
//! one entry per dependency, so an item can wait on several requests and be asked by several
//! boards. A board written before the lists holds one object; every reader takes it as a list
//! of one, and the next write turns it into a list.

use std::path::Path;

use serde_json::{Value, json};

use super::error::DepError;
use super::ledger::{Side, locked};
use super::model::{Dep, DepState, Transition};
use super::notice::{Note, note, write};
use super::sight::{Look, Seen, Sight};
use crate::Board;
use crate::json::{doc_mut, set};
use crate::store::{set_lane, stamp, touch};

/// The `waitsOn` field of the requester item.
pub(crate) fn waits_on(dep: &Dep, title: &str) -> Value {
    json!({
        "dep": dep.id, "project": dep.to.project, "item": dep.to.item, "title": title,
        "state": dep.state, "since": dep.opened_at.as_ref().unwrap_or(&dep.created_at),
        "reason": dep.reason,
    })
}

/// The `requestedBy` field of the assignee item.
pub(crate) fn requested_by(dep: &Dep, title: &str) -> Value {
    json!({
        "dep": dep.id, "project": dep.from.project, "item": dep.from.item,
        "thread": dep.from.thread, "title": title, "state": dep.state,
    })
}

/// The field a side carries.
fn field(side: Side) -> &'static str {
    match side {
        Side::Requester => "waitsOn",
        Side::Assignee => "requestedBy",
    }
}

/// The entries of a mirror field: an object is one entry, an array is its entries, anything else
/// is none.
pub(crate) fn entries(field: &Value) -> Vec<Value> {
    match field {
        Value::Array(list) => list.clone(),
        Value::Object(_) => vec![field.clone()],
        _ => Vec::new(),
    }
}

/// The entry of a dependency in a list of mirror entries.
pub(crate) fn entry_of<'a>(list: &'a [Value], dep: &str) -> Option<&'a Value> {
    list.iter().find(|e| e["dep"].as_str() == Some(dep))
}

/// Add an entry to a mirror field of an item, or replace the one the same dependency has, and
/// mark the item changed. The entries of other dependencies stay as they are.
pub(crate) fn put(board: &mut Board, item: &str, side: Side, value: Value) {
    if let Some(d) = doc_mut(board, "items", item) {
        let mut list = entries(&d[field(side)]);
        match list.iter_mut().find(|e| e["dep"] == value["dep"]) {
            Some(slot) => *slot = value,
            None => list.push(value),
        }
        set(d, field(side), list);
    }
    touch(board, "items", item);
}

fn desired(side: Side, dep: &Dep, title: &str) -> Value {
    match side {
        Side::Requester => waits_on(dep, title),
        Side::Assignee => requested_by(dep, title),
    }
}

/// The entry this dependency has on one side of an item, if any.
fn current<'a>(side: Side, dep: &Dep, seen: &'a Seen) -> Option<&'a Value> {
    let list = match side {
        Side::Requester => &seen.waits_on,
        Side::Assignee => &seen.requested_by,
    };
    entry_of(list, &dep.id)
}

/// What to write on one side of a dependency.
struct Work {
    mirror: Option<Value>,
    drop_idle: bool,
    notes: Vec<Note>,
}

impl Work {
    fn is_empty(&self) -> bool {
        self.mirror.is_none() && !self.drop_idle && self.notes.is_empty()
    }
}

/// The title the mirror carries: the other item's own, else the one it had, else the request's.
fn title_for(side: Side, dep: &Dep, sight: &Sight, seen: &Seen) -> String {
    let other = match side {
        Side::Requester => &sight.assignee,
        Side::Assignee => &sight.requester,
    };
    other
        .item()
        .map(|i| i.title.clone())
        .filter(|t| !t.is_empty())
        .or_else(|| current(side, dep, seen).and_then(|c| c["title"].as_str().map(str::to_owned)))
        .unwrap_or_else(|| dep.title.clone())
}

/// A request the dependency made and nobody started: its item is dropped on a cancel.
fn idle_copy(dep: &Dep, seen: &Seen, moved: &[Transition]) -> bool {
    moved.iter().any(|t| t.to == DepState::Cancelled)
        && seen.status == "todo"
        && seen.created_by == dep.created_by
}

fn look_of(sight: &Sight, side: Side) -> &Look {
    match side {
        Side::Requester => &sight.requester,
        Side::Assignee => &sight.assignee,
    }
}

fn plan(side: Side, dep: &Dep, sight: &Sight, moved: &[Transition]) -> Work {
    let notes: Vec<Note> = moved.iter().map(|t| note(dep, t)).collect();
    let Look::Item(seen) = look_of(sight, side) else {
        return Work {
            mirror: None,
            drop_idle: false,
            notes,
        };
    };
    let want = desired(side, dep, &title_for(side, dep, sight, seen));
    Work {
        mirror: (current(side, dep, seen) != Some(&want)).then_some(want),
        drop_idle: side == Side::Assignee && idle_copy(dep, seen, moved),
        notes,
    }
}

fn apply(
    board: &mut Board,
    (dep, side): (&Dep, Side),
    work: Work,
    seen: Option<&Seen>,
) -> Result<(), DepError> {
    if let Some(seen) = seen {
        if let Some(value) = work.mirror {
            put(board, &seen.id, side, value);
        }
        if work.drop_idle {
            if let Some(d) = doc_mut(board, "items", &seen.id) {
                set_lane(d, "dropped");
            }
            stamp(board, "items", &seen.id, "argus")?;
        }
    }
    write(board, dep, side, &work.notes);
    Ok(())
}

fn side_project(dep: &Dep, side: Side) -> &str {
    match side {
        Side::Requester => &dep.from.project,
        Side::Assignee => &dep.to.project,
    }
}

/// Bring both mirrors to what the ledger says and tell both boards about the transitions just
/// made. A side whose board is gone is skipped; a side that is already right is not locked.
pub(crate) fn settle(
    home: &Path,
    dep: &Dep,
    sight: &Sight,
    moved: &[Transition],
) -> Result<(), DepError> {
    for side in [Side::Requester, Side::Assignee] {
        let look = look_of(sight, side);
        if matches!(look, Look::NoBoard) {
            continue;
        }
        let work = plan(side, dep, sight, moved);
        if work.is_empty() {
            continue;
        }
        let seen = look.item();
        locked(home, side_project(dep, side), |board| {
            apply(board, (dep, side), work, seen)
        })?;
    }
    Ok(())
}
