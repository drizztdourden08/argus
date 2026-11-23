//! Who holds an item and who made a change.

use serde_json::Value;

use super::Hearer;
use crate::json::{text, truthy};

/// The owner as the board writes him in `assignee` and in `updatedBy`.
pub(super) const OWNER: &str = "owner";
const HUMAN: &str = "human";
const LEAD_PREFIX: &str = "lead:";
const AGENT_PREFIX: &str = "agent:";

/// A record that was deleted or archived.
pub(super) fn gone(record: &Value) -> bool {
    truthy(record, "deletedAt") || truthy(record, "archivedAt")
}

/// Who made the last change: a thread id, or `human` for the owner; empty when the board does not say.
pub(super) fn actor(record: &Value) -> String {
    text(record, "updatedBy").unwrap_or_default().to_owned()
}

/// Is this actor the owner?
pub(super) fn is_owner(actor: &str) -> bool {
    actor == HUMAN || actor == OWNER
}

/// Who holds the ball on an item: its `assignee`, else its `thread`.
pub(super) fn assignee(item: &Value) -> Option<String> {
    ["assignee", "thread"]
        .iter()
        .find_map(|key| text(item, key).filter(|who| !who.is_empty()))
        .map(str::to_owned)
}

/// The person as a hearer: the owner, the lead of another board, an agent, or a thread.
pub(super) fn hearer_of(who: &str) -> Hearer {
    if who == OWNER {
        Hearer::Owner
    } else if let Some(key) = who.strip_prefix(LEAD_PREFIX) {
        Hearer::Lead(key.to_owned())
    } else if let Some(id) = who.strip_prefix(AGENT_PREFIX) {
        Hearer::Agent(id.to_owned())
    } else {
        Hearer::Thread(who.to_owned())
    }
}

/// The assignee of an item as a hearer.
pub(super) fn holder(item: &Value) -> Option<Hearer> {
    assignee(item).map(|who| hearer_of(&who))
}

/// The thread that works the item: its assignee when that is a thread, else the thread that is
/// accountable for it (the assignee is the owner then).
pub(super) fn worker(item: &Value) -> Option<String> {
    match holder(item)? {
        Hearer::Thread(thread) => Some(thread),
        _ => text(item, "thread")
            .filter(|thread| !thread.is_empty())
            .map(str::to_owned),
    }
}

/// The lane an item sits in; a missing status reads `todo`.
pub(super) fn lane(item: &Value) -> &str {
    text(item, "status").unwrap_or("todo")
}

/// Is the item finished, done or dropped?
pub(super) fn closed(item: &Value) -> bool {
    matches!(lane(item), "done" | "dropped")
}

/// The type of an item; a missing type reads `task`.
pub(super) fn type_of(item: &Value) -> &str {
    text(item, "type").unwrap_or("task")
}
