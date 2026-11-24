//! The delivery of a lead's message: an entry on the target board, a line in the target lead's
//! inbox and a line in its log (`deliver` of `verbs/portfolio.mjs`, speaking for a lead).

use std::path::Path;

use argus_board::BoardError;
use argus_board::inbox::{Message, message_id, push};
use argus_board::json::{array_mut, doc, find_ids, map_mut, nullish, set, slot};
use argus_board::store::{create_doc, read_board, touch, with_board};
use argus_board::text::clip_text;
use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

/// What a delivery needs of the message.
pub(super) struct Note<'a> {
    pub kind: &'a str,
    pub text: &'a str,
    /// The sending project, whose lead speaks.
    pub from: &'a str,
}

fn lead_of(board: &Value) -> String {
    let ids = find_ids(board, "threads", |t| {
        t["role"] == "lead" && t["status"] != "done"
    });
    ids.first()
        .and_then(|id| doc(board, "threads", id).and_then(|t| nullish(t, "id")))
        .unwrap_or_else(|| "lead".to_owned())
}

/// One line of the `leads` log, numbered and never trimmed.
fn log_line(board: &mut Value, text: &str) {
    let key = "log:leads";
    let seq = board["counters"][key].as_i64().unwrap_or(0) + 1;
    map_mut(board, "counters").insert(key.to_owned(), json!(seq));
    let at = now_iso();
    let log = slot(slot(board, "log"), "leads");
    if log.is_null() {
        *log = json!({ "subject": "leads", "entries": [], "updatedAt": at });
    }
    array_mut(log, "entries")
        .push(json!({ "seq": seq, "ts": at, "thread": "leads", "item": null, "text": text }));
    set(log, "updatedAt", now_iso());
    touch(board, "log", "leads");
}

fn entry_fields(note: &Note<'_>, to: &str) -> Map<String, Value> {
    let decision = note.kind == "decision";
    let fields = json!({
        "kind": note.kind, "author": format!("{} lead", note.from), "via": "leads", "ref": null,
        "thread": to, "item": null, "text": note.text,
        "status": if decision { "pending" } else { "info" },
    });
    match fields {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

/// Hands the message to the lead of `key`. Returns the thread it was queued for, `None` when the
/// board does not exist yet.
///
/// # Errors
/// The board cannot be locked, read or written.
pub(super) fn deliver(
    home: &Path,
    key: &str,
    note: &Note<'_>,
) -> Result<Option<String>, BoardError> {
    if read_board(home, key)?.is_none() {
        return Ok(None);
    }
    let actor = format!("{} lead", note.from);
    let queued = with_board(home, key, false, |board| {
        let to = lead_of(board);
        let id = create_doc(board, "entries", "P", entry_fields(note, &to), &actor)?;
        let decision = note.kind == "decision";
        let pick = if decision {
            format!(" Run coord entry pick {id} when you start on it.")
        } else {
            String::new()
        };
        let line = format!("{id} {} from {actor}: {}{pick}", note.kind, note.text);
        let message = Message {
            id: message_id(&id),
            from: &actor,
            kind: note.kind,
            text: &line,
            reference: Some(&id),
        };
        push(board, &to, &message);
        let word = if decision { "decision" } else { "note" };
        let logged = format!("{id} {word} from {actor}: {}", clip_text(note.text, 80));
        log_line(board, &logged);
        Ok(to)
    })?;
    Ok(Some(queued))
}
