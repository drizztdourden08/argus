//! `coord action show <H>` and `coord action list [--mine|--for T] [--archived] [--deleted]
//! [--json]`.

use argus_board::Board;
use serde_json::Value;

use super::assignee_of;
use crate::cli::Outcome;
use crate::cli::out::{ago, clip};
use crate::verbs::work::crud::find_doc;
use crate::verbs::work::listing::{Table, cell, list_docs, print_json, scoped};
use crate::verbs::work::run::{Res, Run};

const ACTIONS: Table = Table {
    head: &["action", "status", "thread", "item", "age", "text"],
    empty: "no manual actions.",
};

/// `coord action show`.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        print_json(r, &find_doc(board, "actions", r.arg(0))?);
        Ok(())
    })
}

fn row(action: &Value, now: i64) -> Vec<String> {
    let made = action["createdAt"].as_str().filter(|s| !s.is_empty());
    vec![
        cell(&action["id"]),
        cell(&action["status"]),
        cell(&action["thread"]),
        cell(&action["item"]),
        ago(made, now),
        clip(&for_whom(action), 70),
    ]
}

/// The text of a to-do, led by who it is for when that is not the owner.
fn for_whom(action: &Value) -> String {
    let text = cell(&action["text"]);
    match assignee_of(action) {
        "owner" => text,
        who => format!("[{who}] {text}"),
    }
}

/// The scope of the list, and `--who` keeps the to-dos for one assignee.
fn narrow(board: &Board, r: &Run<'_>, docs: Vec<Value>) -> Res<Vec<Value>> {
    let mut docs = scoped(board, r, docs)?;
    if let Some(who) = r.text_opt("who") {
        docs.retain(|a| assignee_of(a) == who);
    }
    Ok(docs)
}

/// `coord action list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    list_docs(r, "actions", &ACTIONS, narrow, row)
}
