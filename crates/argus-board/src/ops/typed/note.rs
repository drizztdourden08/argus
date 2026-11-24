//! The note: an informal remark on an item, kept as an entry of kind `note`. It never blocks.

use serde_json::{Value, json};

use super::images::numbers;
use super::known::Known;
use crate::Board;
use crate::content::check;
use crate::error::{BoardError, refuse};
use crate::json::doc;
use crate::ops::target::{record_arg, record_thread};
use crate::ops::{Check, Who, authorize};
use crate::store::create_doc;

/// The numbers of the images of the record a note is about.
fn pictures_of(board: &Board, id: &str) -> Vec<u32> {
    doc(board, "items", id).map_or_else(Vec::new, |i| numbers(&i["images"]))
}

/// The new text of a note entry, checked like a new note on the same record.
pub(crate) fn rewrite(board: &Board, entry: &Value, text: &Value) -> Result<Value, BoardError> {
    let record = entry["item"].as_str().unwrap_or_default();
    let known = Known::of(board, pictures_of(board, record));
    let body = check("note", text, &known.context())?;
    if body.is_null() {
        return Err(refuse("note text is required."));
    }
    Ok(body)
}

/// Add a note on an item or a test: an entry of kind `note` with the next `N-n` id, its text
/// checked as rich text of at most 300 characters. Only the thread that owns the record (or the
/// lead, or the owner) adds one.
///
/// # Errors
/// Refused when the record does not exist, the caller may not add to it, or the text is blank or
/// breaks the note rules.
pub fn create_note(
    board: &mut Board,
    who: &Who,
    item: &str,
    text: &Value,
) -> Result<String, BoardError> {
    let record = record_arg(board, &json!(item))?;
    let thread = record_thread(board, &record);
    authorize(who, &Check::new("add", "entries", None).to(thread.as_str()))?;
    let body = rewrite(board, &json!({"item": record}), text)?;
    let entry = json!({
        "kind": "note", "item": record, "thread": thread, "author": who.actor_of(),
        "text": body, "status": "done", "replyTo": null, "deliveredAt": null, "context": null,
    });
    let fields = entry.as_object().cloned().unwrap_or_default();
    create_doc(board, "entries", "N", fields, &who.actor_of())
}
