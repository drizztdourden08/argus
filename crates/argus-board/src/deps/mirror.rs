//! The two mirrors of a dependency: `waitsOn` on the requester item and `requestedBy` on the
//! assignee item. Only this module writes them. A mirror is a cache of the ledger that carries
//! the other side's title, so a board page never reads another scope.

use std::path::Path;

use serde_json::{Value, json};

use super::error::DepError;
use super::ledger::{Side, locked};
use super::model::{Dep, DepState, Transition};
use super::notice::{Note, note, write};
use super::sight::{Look, Seen, Sight};
use crate::Board;
use crate::json::{doc_mut, set};
use crate::store::{stamp, touch};

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

/// Set a mirror field of an item and mark it changed.
pub(crate) fn put(board: &mut Board, item: &str, side: Side, value: Value) {
    if let Some(d) = doc_mut(board, "items", item) {
        set(d, field(side), value);
    }
    touch(board, "items", item);
}

fn desired(side: Side, dep: &Dep, title: &str) -> Value {
    match side {
        Side::Requester => waits_on(dep, title),
        Side::Assignee => requested_by(dep, title),
    }
}

fn current(side: Side, seen: &Seen) -> &Value {
    match side {
        Side::Requester => &seen.waits_on,
        Side::Assignee => &seen.requested_by,
    }
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
        .or_else(|| current(side, seen)["title"].as_str().map(str::to_owned))
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
    let have = current(side, seen);
    let mine = have.is_null() || have["dep"].as_str() == Some(dep.id.as_str());
    let want = desired(side, dep, &title_for(side, dep, sight, seen));
    Work {
        mirror: (mine && *have != want).then_some(want),
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
                set(d, "status", "dropped");
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
