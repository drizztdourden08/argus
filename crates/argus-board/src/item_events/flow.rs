//! The work around an item moving on: its last blocker settled, its children all done, a request to
//! another board landed.

use serde_json::Value;

use super::people::{closed, gone, holder};
use super::{EventKind, ItemEvent, Moved, Seen};
use crate::Board;
use crate::blocked::causes;
use crate::json::{doc, section, text};

/// Does the item name anything that may hold it back? The cheap look that spares the full rule.
fn names_a_hold(m: &Moved<'_>) -> bool {
    let linked = ["blockedBy", "waitsOn", "requestedBy", "blocker"]
        .iter()
        .any(|key| match &m.was[*key] {
            Value::Null => false,
            Value::Array(list) => !list.is_empty(),
            Value::Object(fields) => !fields.is_empty(),
            Value::String(word) => !word.is_empty(),
            _ => true,
        });
    linked || m.seen.asked.contains(m.id)
}

/// An open item that had a reason to wait and has none now. The detail lists the ids that settled.
pub(super) fn unblocked(m: &Moved<'_>) -> Vec<ItemEvent> {
    let (Some(to), false) = (holder(m.now), closed(m.now)) else {
        return Vec::new();
    };
    if !names_a_hold(m) {
        return Vec::new();
    }
    let held = causes(m.seen.before, m.id);
    if held.is_empty() || !causes(m.seen.after, m.id).is_empty() {
        return Vec::new();
    }
    let ids: Vec<&str> = held.iter().map(|c| c.id.as_str()).collect();
    let rev = ids.iter().map(|id| rev_of(m.seen.after, id)).max();
    let event = m.event(EventKind::Unblocked, to, ids.join(", "));
    vec![event.at_least(rev.unwrap_or(0))]
}

/// The revision of a record the board keeps, found by the kind its id names; 0 for any other id.
fn rev_of(board: &Board, id: &str) -> i64 {
    let coll = match id.split('-').next() {
        Some("I") => "items",
        Some("T") => "tests",
        Some("H") => "actions",
        Some("Q" | "D") => "entries",
        _ => return 0,
    };
    doc(board, coll, id).map_or(0, |d| d["rev"].as_i64().unwrap_or(0))
}

/// The live children of an item.
fn children<'a>(board: &'a Board, parent: &'a str) -> impl Iterator<Item = &'a Value> {
    section(board, "items")
        .values()
        .filter(move |i| text(i, "parent") == Some(parent) && !gone(i))
}

/// Has the item children, and are they all done or dropped?
fn all_closed(board: &Board, parent: &str) -> bool {
    let mut kids = children(board, parent).peekable();
    kids.peek().is_some() && kids.all(closed)
}

/// The parent of a child that was closed by this change, if it has one.
fn parent_closed_by_change<'a>(seen: &Seen<'_>, id: &str, child: &'a Value) -> Option<&'a str> {
    let was_open = doc(seen.before, "items", id).is_some_and(|was| !closed(was));
    let parent = text(child, "parent")?;
    (closed(child) && was_open).then_some(parent)
}

/// A parent whose children are all done or dropped now and were not before, told to the parent's assignee.
pub(super) fn children_done(seen: &Seen<'_>) -> Vec<ItemEvent> {
    let mut parents: Vec<&str> = Vec::new();
    for (id, child) in section(seen.after, "items")
        .iter()
        .filter(|(_, i)| !gone(i))
    {
        if let Some(parent) = parent_closed_by_change(seen, id, child) {
            if !parents.contains(&parent) {
                parents.push(parent);
            }
        }
    }
    parents
        .into_iter()
        .filter_map(|parent| parent_event(seen, parent))
        .collect()
}

fn parent_event(seen: &Seen<'_>, parent: &str) -> Option<ItemEvent> {
    let item = doc(seen.after, "items", parent).filter(|i| !gone(i) && !closed(i))?;
    if !all_closed(seen.after, parent) || all_closed(seen.before, parent) {
        return None;
    }
    let was = &seen.before["items"][parent];
    let moved = Moved {
        seen,
        id: parent,
        was,
        now: item,
    };
    let kids: Vec<&Value> = children(seen.after, parent).collect();
    let rev = kids.iter().map(|k| k["rev"].as_i64().unwrap_or(0)).max();
    let event = moved.event(
        EventKind::ChildrenDone,
        holder(item)?,
        kids.len().to_string(),
    );
    Some(event.at_least(rev.unwrap_or(0)))
}

/// The state of every request to another board that the item waits on, by dependency id.
fn requests(item: &Value) -> Vec<(String, String)> {
    let waits = &item["waitsOn"];
    let list: Vec<&Value> = match waits {
        Value::Array(all) => all.iter().collect(),
        Value::Object(_) => vec![waits],
        _ => Vec::new(),
    };
    list.into_iter()
        .map(|w| {
            let id = text(w, "dep").unwrap_or_default().to_owned();
            (id, text(w, "state").unwrap_or_default().to_owned())
        })
        .collect()
}

/// The requests of the item that are landed now (done, dropped or gone) and were not before.
pub(super) fn dep_landed(m: &Moved<'_>) -> Vec<ItemEvent> {
    let Some(to) = holder(m.now) else {
        return Vec::new();
    };
    let before = requests(m.was);
    let landed: Vec<String> = requests(m.now)
        .into_iter()
        .filter(|(_, state)| matches!(state.as_str(), "done" | "dropped" | "gone"))
        .filter(|(dep, state)| !before.contains(&(dep.clone(), state.clone())))
        .map(|(dep, _)| dep)
        .collect();
    if landed.is_empty() {
        Vec::new()
    } else {
        vec![m.event(EventKind::DepLanded, to, landed.join(", "))]
    }
}
