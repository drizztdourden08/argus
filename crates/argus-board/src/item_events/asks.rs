//! A question or a decision that another thread put on an item.

use serde_json::Value;

use super::people::{gone, is_owner, worker};
use super::{EventKind, Hearer, ItemEvent, Seen};
use crate::json::{doc, section, text};

/// Is this entry an ask that holds its item: an open question or a pending decision?
pub(super) fn is_ask(entry: &Value) -> bool {
    matches!(
        (text(entry, "kind"), text(entry, "status")),
        (Some("question"), Some("open")) | (Some("decision"), Some("pending"))
    )
}

/// Who asked: the author, else the thread that wrote it.
fn asker(entry: &Value) -> &str {
    text(entry, "author")
        .filter(|who| !who.is_empty())
        .or_else(|| text(entry, "thread"))
        .unwrap_or_default()
}

/// The asks that are new since the last export, each told to the thread that works the item. The owner's own
/// notes have their own path, and a thread asking on its own item hears nothing.
pub(super) fn new_asks(seen: &Seen<'_>) -> Vec<ItemEvent> {
    let mut out = Vec::new();
    for (id, entry) in section(seen.after, "entries") {
        let fresh = doc(seen.before, "entries", id).is_none();
        if fresh && is_ask(entry) && !gone(entry) {
            out.extend(ask_event(seen, id, entry));
        }
    }
    out
}

fn ask_event(seen: &Seen<'_>, id: &str, entry: &Value) -> Option<ItemEvent> {
    let item_id = text(entry, "item")?;
    let item = doc(seen.after, "items", item_id).filter(|item| !gone(item))?;
    let by = asker(entry);
    let thread = worker(item).filter(|thread| thread != by)?;
    (!is_owner(by)).then(|| ItemEvent {
        kind: EventKind::Ask,
        id: id.to_owned(),
        item: item_id.to_owned(),
        rev: entry["rev"].as_i64().unwrap_or(0),
        by: by.to_owned(),
        title: text(item, "title").unwrap_or_default().to_owned(),
        to: Hearer::Thread(thread),
        detail: text(entry, "text").unwrap_or_default().to_owned(),
    })
}
