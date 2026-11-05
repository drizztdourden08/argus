//! `coord test show <T>` and `coord test list [--item I] [--mine|--for T] [--archived]
//! [--deleted] [--json]`.

use argus_board::Board;
use serde_json::Value;

use super::spec::mode_label;
use crate::cli::{Outcome, out::clip};
use crate::verbs::work::{
    crud::find_doc,
    listing::{Table, cell, list_docs, print_json, scoped},
    run::{Res, Run},
};

const TESTS: Table = Table {
    head: &["test", "status", "mode", "item", "thread", "proofs", "title"],
    empty: "no tests.",
};

/// `coord test show`.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let test = find_doc(board, "tests", r.arg(0))?;
        print_json(r, &test);
        Ok(())
    })
}

fn row(test: &Value, _now: i64) -> Vec<String> {
    vec![
        cell(&test["id"]),
        cell(&test["status"]),
        mode_label(&test["mode"]).to_owned(),
        cell(&test["item"]),
        cell(&test["thread"]),
        test["proofs"].as_array().map_or(0, Vec::len).to_string(),
        clip(&cell(&test["title"]), 50),
    ]
}

fn narrow(board: &Board, r: &Run<'_>, mut tests: Vec<Value>) -> Res<Vec<Value>> {
    if let Some(item) = r.text_opt("item") {
        tests.retain(|t| t["item"].as_str() == Some(item));
    }
    scoped(board, r, tests)
}

/// `coord test list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    list_docs(r, "tests", &TESTS, narrow, row)
}
