//! How boards are named to a caller: the `name (id)` label and the identity fields of a card.

use argus_core::boards::{Boards, display_name, load};
use serde_json::{Value, json};

use crate::backend::McpBackend;

/// `name (id)` for a board of the table, the bare id when the table has no name for it.
#[must_use]
pub fn label(backend: &dyn McpBackend, id: &str) -> String {
    backend
        .home()
        .and_then(|home| display_name(&home, id))
        .map_or_else(|| id.to_owned(), |name| format!("{name} ({id})"))
}

/// A card with its board's `name` and `aliases` from the identity table.
#[must_use]
pub fn describe(backend: &dyn McpBackend, mut card: Value) -> Value {
    let table = backend
        .home()
        .map(|home| load(&home).unwrap_or_default())
        .unwrap_or_default();
    stamp(&table, &mut card);
    card
}

fn stamp(table: &Boards, card: &mut Value) {
    let Some(fields) = card.as_object_mut() else {
        return;
    };
    let id = fields.get("id").and_then(Value::as_str).unwrap_or_default();
    let entry = table.boards.get(id);
    let aliases = entry.map_or_else(|| json!([]), |e| json!(e.aliases));
    if let Some(entry) = entry {
        fields.insert("name".into(), json!(entry.name));
    }
    fields.insert("aliases".into(), aliases);
}
