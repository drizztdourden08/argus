//! Items and to-dos given to someone, or taken away.

use serde_json::Value;

use super::people::{actor, assignee, gone, hearer_of, is_owner, worker};
use super::{EventKind, Hearer, ItemEvent, Moved, Seen};
use crate::json::{section, text};

/// An item given to a thread or to the owner by someone else, and the thread it was taken from.
/// A thread handing an item to itself, or taking its own away, says nothing; neither does giving the
/// item to the owner while the same thread stays accountable for it.
pub(super) fn item(m: &Moved<'_>) -> Vec<ItemEvent> {
    if assignee(m.was) == assignee(m.now) {
        return Vec::new();
    }
    let by = actor(m.now);
    let mut out = Vec::new();
    let kept = worker(m.now);
    if let Some(old) = worker(m.was).filter(|old| *old != by && Some(old) != kept.as_ref()) {
        out.push(m.event(EventKind::Unassigned, Hearer::Thread(old), String::new()));
    }
    let given = assignee(m.now)
        .filter(|new| *new != by)
        .map(|new| hearer_of(&new));
    match given {
        Some(Hearer::Owner) if is_owner(&by) => {}
        Some(hearer @ (Hearer::Owner | Hearer::Thread(_))) => {
            out.push(m.event(EventKind::Assigned, hearer, String::new()));
        }
        _ => {}
    }
    out
}

/// The assignee a to-do names itself; a to-do that names none is the owner's by default and tells nobody.
fn named(todo: &Value) -> Option<&str> {
    text(todo, "assignee").filter(|who| !who.is_empty())
}

/// A pending to-do that was given to someone: the owner, a thread or the lead of another board.
pub(super) fn todos(seen: &Seen<'_>) -> Vec<ItemEvent> {
    let mut out = Vec::new();
    for (id, todo) in section(seen.after, "actions") {
        let was = &seen.before["actions"][id.as_str()];
        out.extend(todo_event(id, was, todo));
    }
    out
}

fn todo_event(id: &str, was: &Value, todo: &Value) -> Option<ItemEvent> {
    let who = named(todo).filter(|who| Some(*who) != named(was))?;
    if gone(todo) || text(todo, "status") != Some("pending") {
        return None;
    }
    let by = actor(todo);
    let to = hearer_of(who);
    let own_act = match &to {
        Hearer::Thread(thread) => *thread == by,
        Hearer::Owner => is_owner(&by),
        Hearer::Lead(_) => false,
    };
    (!own_act).then(|| ItemEvent {
        kind: EventKind::TodoAssigned,
        id: id.to_owned(),
        item: text(todo, "item").unwrap_or_default().to_owned(),
        rev: todo["rev"].as_i64().unwrap_or(0),
        by,
        title: text(todo, "text").unwrap_or_default().to_owned(),
        to,
        detail: String::new(),
    })
}
