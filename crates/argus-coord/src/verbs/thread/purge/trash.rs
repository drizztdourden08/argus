//! `coord trash [--for T] [--json]`: every soft-deleted record, all collections, newest delete
//! first: what `rm` hid and `purge --trash` would remove.

use argus_board::Board;
use argus_board::json::{section, truthy};
use serde_json::{Value, json};

use super::need_lead;
use crate::cli::exit::Outcome;
use crate::cli::out::{clip, table};
use crate::cli::{Opt, Val};
use crate::verbs::thread::docs::{cell, field, object, or_null};
use crate::verbs::thread::frame::Frame;

const TRASHED: [&str; 6] = ["threads", "items", "tests", "entries", "actions", "agents"];
const FOOTER: &str = "Bring one back: coord <item|test|entry|action|agent> restore <id>. Remove them for good: coord purge --trash [--older <days>].";

/// A soft-deleted record and the collection it is in.
struct Trashed<'a> {
    collection: &'static str,
    doc: &'a Value,
}

impl Trashed<'_> {
    fn owner(&self) -> Option<&Value> {
        let key = if self.collection == "threads" {
            "id"
        } else {
            "thread"
        };
        self.doc.get(key)
    }
}

fn found<'a>(board: &'a Board, owner: Option<&str>) -> Vec<Trashed<'a>> {
    let mut found: Vec<Trashed<'a>> = Vec::new();
    for collection in TRASHED {
        for doc in section(board, collection).values() {
            let entry = Trashed { collection, doc };
            let mine = owner.is_none_or(|o| entry.owner().and_then(Value::as_str) == Some(o));
            if truthy(doc, "deletedAt") && mine {
                found.push(entry);
            }
        }
    }
    found.sort_by_key(|t| std::cmp::Reverse(cell(t.doc.get("deletedAt"))));
    found
}

fn as_json(found: &[Trashed<'_>]) -> Value {
    let rows = found.iter().map(|t| {
        object(vec![
            ("collection", Some(json!(t.collection))),
            ("id", field(t.doc, "id")),
            ("thread", Some(or_null(field(t.doc, "thread")))),
            ("deletedAt", field(t.doc, "deletedAt")),
            ("deletedBy", Some(or_null(field(t.doc, "deletedBy")))),
            ("reason", Some(or_null(field(t.doc, "deleteReason")))),
        ])
    });
    Value::Array(rows.collect())
}

/// The first of these fields that is set: `title ?? text ?? task ?? name`.
fn what(doc: &Value) -> String {
    let first = ["title", "text", "task", "name"]
        .iter()
        .find_map(|k| doc.get(*k).filter(|v| !v.is_null()));
    clip(&cell(first), 50)
}

/// The verb `trash [--for T] [--json]`.
///
/// # Errors
/// Refused for a caller that is not the lead, and no board.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let who = frame.caller(&board)?;
    need_lead(&who, "read the trash")?;
    let entries = found(&board, frame.options.text("for").filter(|o| !o.is_empty()));
    if matches!(frame.options.get("json"), Some(Opt::One(Val::Flag))) {
        frame.json(&as_json(&entries));
        return Ok(());
    }
    if entries.is_empty() {
        frame.say(&["the trash is empty."]);
        return Ok(());
    }
    let mut rows = vec![
        ["id", "kind", "thread", "deleted", "by", "reason", "what"]
            .iter()
            .map(|h| (*h).to_owned())
            .collect::<Vec<_>>(),
    ];
    for t in &entries {
        rows.push(vec![
            cell(t.doc.get("id")),
            t.collection.to_owned(),
            cell(t.owner()),
            frame.ago(&t.doc["deletedAt"]),
            cell(t.doc.get("deletedBy")),
            clip(&cell(t.doc.get("deleteReason")), 30),
            what(t.doc),
        ]);
    }
    frame.say(&[&table(&rows), "", FOOTER]);
    Ok(())
}
