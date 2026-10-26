//! `coord entry show <E>` and `coord entry list [--open] [--kind k] [--mine|--for T] [--archived]
//! [--deleted] [--json]`.

use argus_board::Board;
use argus_board::json::{display, truthy};
use serde_json::{Value, json};

use super::prefix_of;
use crate::cli::Outcome;
use crate::cli::out::{ago, clip};
use crate::verbs::work::crud::{docs_of, find_doc};
use crate::verbs::work::listing::{Table, cell, list_docs, print_json, scoped};
use crate::verbs::work::run::{Res, Run};

const ENTRIES: Table = Table {
    head: &["entry", "kind", "status", "thread", "item", "age", "text"],
    empty: "no entries.",
};

/// `coord entry show`: the entry and its replies.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let mut e = find_doc(board, "entries", r.arg(0))?;
        let id = display(&e["id"]);
        let replies: Vec<Value> = docs_of(board, "entries")
            .into_iter()
            .filter(|x| display(&x["replyTo"]) == id && !truthy(x, "deletedAt"))
            .collect();
        if let Some(fields) = e.as_object_mut() {
            fields.insert("replies".into(), json!(replies));
        }
        print_json(r, &e);
        Ok(())
    })
}

fn row(e: &Value, now: i64) -> Vec<String> {
    vec![
        cell(&e["id"]),
        cell(&e["kind"]),
        cell(&e["status"]),
        cell(&e["thread"]),
        cell(&e["item"]),
        ago(e["createdAt"].as_str().filter(|s| !s.is_empty()), now),
        clip(&cell(&e["text"]), 60),
    ]
}

fn narrow(board: &Board, r: &Run<'_>, mut entries: Vec<Value>) -> Res<Vec<Value>> {
    entries.retain(|e| e["kind"].as_str().is_some_and(|k| prefix_of(k).is_some()));
    if r.options.flag("open") {
        entries.retain(|e| matches!(e["status"].as_str(), Some("open" | "pending" | "doing")));
    }
    if let Some(kind) = r.text_opt("kind") {
        entries.retain(|e| e["kind"].as_str() == Some(kind));
    }
    scoped(board, r, entries)
}

/// `coord entry list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    list_docs(r, "entries", &ENTRIES, narrow, row)
}
