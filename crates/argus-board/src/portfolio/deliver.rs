//! Handing the owner's posts from the Argus page down to the project leads: an entry on the
//! project's board, a line in its inbox (its hooks deliver it), and a line in its log.

use std::path::Path;

use serde_json::{Map, Value, json};

use super::registry::project_keys;
use super::{boards, profile};
use crate::Board;
use crate::error::BoardError;
use crate::inbox::{Message, message_id, push};
use crate::json::{display, doc, doc_mut, find_ids, nullish, set, shown, truthy};
use crate::store::{LogLine, create_doc, push_log_line, read_board, touch, with_board};
use crate::text::clip_text;

const POST_KINDS: [&str; 2] = ["decision", "followup"];

/// Who a delivery speaks for.
struct Sender {
    author: &'static str,
    label: &'static str,
}

fn sender(from_owner: bool) -> Sender {
    if from_owner {
        Sender {
            author: "human",
            label: "owner (Argus)",
        }
    } else {
        Sender {
            author: "portfolio",
            label: "the Overlord",
        }
    }
}

/// Hand one message to a project's coordinator. Returns the entry id and the lead thread it was
/// queued for, or `None` when the board does not exist yet.
fn deliver(
    home: &Path,
    key: &str,
    (kind, text, reference): (&str, &str, &str),
    from_owner: bool,
) -> Result<Option<(String, String)>, BoardError> {
    if read_board(home, key)?.is_none() {
        return Ok(None);
    }
    let sender = sender(from_owner);
    let delivered = with_board(home, key, false, |board| {
        let lead = find_ids(board, "threads", |t| {
            t["role"] == "coordinator" && t["status"] != "done"
        });
        let to = lead
            .first()
            .and_then(|id| doc(board, "threads", id).and_then(|t| nullish(t, "id")))
            .unwrap_or_else(|| "coordinator".to_owned());
        let decision = kind == "decision";
        let fields = json!({
            "kind": kind, "author": sender.author, "via": "portfolio", "ref": reference,
            "thread": to, "item": null, "text": text,
            "status": if decision { "pending" } else { "info" },
        });
        let fields = match fields {
            Value::Object(m) => m,
            _ => Map::new(),
        };
        let id = create_doc(board, "entries", "P", fields, sender.author)?;
        let pick = if decision {
            format!(" Run coord entry pick {id} when you start on it.")
        } else {
            String::new()
        };
        let line = format!("{id} {kind} from the Overlord: {text}{pick}");
        let message = Message {
            id: message_id(&id),
            from: sender.label,
            kind,
            text: &line,
            reference: Some(&id),
        };
        push(board, &to, &message);
        let word = if decision { "decision" } else { "note" };
        let log = format!("{id} {word} from the Overlord: {}", clip_text(text, 80));
        push_log_line(
            board,
            "portfolio",
            &LogLine {
                thread: "portfolio",
                item: None,
                text: &log,
            },
            None,
        );
        Ok((id, to))
    })?;
    Ok(Some(delivered))
}

fn pending_of(home: &Path, p: &Board, entry: &Value) -> Vec<String> {
    if let Some(list) = entry["pendingFor"].as_array() {
        return list.iter().map(display).collect();
    }
    if entry["project"] == "all" {
        project_keys(home, p)
    } else {
        vec![shown(entry.get("project"))]
    }
}

fn unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in values {
        if !out.contains(&v) {
            out.push(v);
        }
    }
    out
}

/// Deliver an entry of the portfolio to the projects it names (`all` or one). Only a change in
/// delivery is recorded; a note still waiting stays as it is.
///
/// # Errors
/// A project board cannot be locked, read or written.
pub fn dispatch_entry(home: &Path, p: &mut Board, entry_id: &str) -> Result<(), BoardError> {
    let Some(entry) = doc(p, "entries", entry_id).cloned() else {
        return Ok(());
    };
    let pending = pending_of(home, p, &entry);
    let from_owner = entry["author"] == "human";
    let text = display(&entry["text"]);
    let kind = display(&entry["kind"]);
    let mut left = Vec::new();
    for key in &pending {
        let sent = deliver(home, key, (&kind, &text, entry_id), from_owner)?;
        if sent.is_none() {
            left.push(key.clone());
        }
    }
    let status = if left.is_empty() {
        "delivered"
    } else {
        "waiting for a board"
    };
    let had = entry["pendingFor"].as_array().map_or(0, Vec::len);
    if entry["status"] == status && left.len() == had && truthy(&entry, "deliveredTo") {
        return Ok(());
    }
    let sent_to = pending.iter().filter(|k| !left.contains(k)).cloned();
    let before = entry["deliveredTo"]
        .as_array()
        .into_iter()
        .flatten()
        .map(display);
    let delivered = unique(before.chain(sent_to));
    if let Some(e) = doc_mut(p, "entries", entry_id) {
        set(e, "deliveredTo", delivered);
        set(e, "pendingFor", left);
        set(e, "status", status);
    }
    touch(p, "entries", entry_id);
    Ok(())
}

/// What importing the owner's portfolio posts did.
#[derive(Debug, Clone, Default)]
pub struct PortfolioImport {
    /// How many posts were new and handed down.
    pub imported: usize,
    /// `(id, status)` of each post.
    pub results: Vec<(String, String)>,
}

/// The owner's posts from the Argus page, each handed down at once. A known id or a kind other
/// than decision or followup is not applied again; its current status is reported.
///
/// # Errors
/// A project board cannot be locked, read or written.
pub fn import_portfolio_entries(
    home: &Path,
    p: &mut Board,
    entries: Vec<Value>,
) -> Result<PortfolioImport, BoardError> {
    let mut out = PortfolioImport::default();
    for e in entries {
        let id = display(&e["id"]);
        let status = if let Some(known) = doc(p, "entries", &id) {
            nullish(known, "status").unwrap_or_else(|| "duplicate".to_owned())
        } else if e["target"]["c"] == "boards" {
            out.imported += 1;
            boards::import(home, p, &id, e)
        } else if e["author"] != "human" {
            "refused".to_owned()
        } else if e["kind"] == "profile" {
            out.imported += 1;
            profile::import(p, &id, e)
        } else if !e["kind"].as_str().is_some_and(|k| POST_KINDS.contains(&k)) {
            "refused".to_owned()
        } else {
            crate::json::map_mut(p, "entries").insert(id.clone(), e.clone());
            dispatch_entry(home, p, &id)?;
            out.imported += 1;
            doc(p, "entries", &id).map_or_else(String::new, |d| display(&d["status"]))
        };
        out.results.push((id, status));
    }
    Ok(out)
}
