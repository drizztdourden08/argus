//! `coord action show <H>` and `coord action list [--mine|--for T] [--archived] [--deleted]
//! [--json]`.

use serde_json::Value;

use crate::cli::Outcome;
use crate::cli::out::{ago, clip};
use crate::verbs::work::crud::{docs_of, find_doc};
use crate::verbs::work::listing::{Table, cell, emit, scoped, visible};
use crate::verbs::work::run::Run;

const ACTIONS: Table = Table {
    head: &["action", "status", "thread", "item", "age", "text"],
    empty: "no manual actions.",
};

/// `coord action show`.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let action = find_doc(board, "actions", r.arg(0))?;
        r.say(&[&serde_json::to_string_pretty(&action).unwrap_or_default()]);
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
        clip(&cell(&action["text"]), 70),
    ]
}

/// `coord action list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let actions: Vec<Value> = docs_of(board, "actions")
            .into_iter()
            .filter(|a| visible(a, r.options))
            .collect();
        let actions = scoped(board, r, actions)?;
        emit(r, actions, &ACTIONS, row);
        Ok(())
    })
}
