//! An item that changed stage under its assignee, and a bug the owner reported that is ready to verify.

use serde_json::Value;

use super::people::{actor, assignee, is_owner, lane, type_of, worker};
use super::{EventKind, Hearer, ItemEvent, Moved};
use crate::json::text;

const VERIFYING: &str = "verifying";

/// The stage an item sits in, as it names it, else its lane.
fn place(item: &Value) -> &str {
    text(item, "stage")
        .filter(|stage| !stage.is_empty())
        .unwrap_or_else(|| lane(item))
}

/// The stage or the lane of an item that was already there changed, by someone other than the thread that works it.
/// A change of hands is told by the assignment events, not twice.
pub(super) fn changed(m: &Moved<'_>) -> Vec<ItemEvent> {
    if !m.was.is_object() || assignee(m.was) != assignee(m.now) {
        return Vec::new();
    }
    let (from, to) = (place(m.was), place(m.now));
    let Some(thread) = worker(m.now).filter(|thread| *thread != actor(m.now)) else {
        return Vec::new();
    };
    if from == to && lane(m.was) == lane(m.now) {
        return Vec::new();
    }
    vec![m.event(
        EventKind::StageChanged,
        Hearer::Thread(thread),
        format!("{from} to {to}"),
    )]
}

/// Who reported a bug: its `reporter` field, else who created it.
fn reporter(bug: &Value) -> &str {
    text(&bug["fields"], "reporter")
        .or_else(|| text(bug, "createdBy"))
        .unwrap_or_default()
}

/// A bug the owner reported reached `verifying` by someone else's hand: it waits for him.
pub(super) fn verify(m: &Moved<'_>) -> Vec<ItemEvent> {
    let reaches = place(m.now) == VERIFYING && place(m.was) != VERIFYING;
    let told = type_of(m.now) == "bug" && is_owner(reporter(m.now)) && !is_owner(&actor(m.now));
    if reaches && told {
        vec![m.event(EventKind::Verify, Hearer::Owner, String::new())]
    } else {
        Vec::new()
    }
}
