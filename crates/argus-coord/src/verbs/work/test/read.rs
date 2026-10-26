//! `coord test show <T>` and `coord test list [--item I] [--mine|--for T] [--build <tag>]
//! [--archived] [--deleted] [--json]`.

use serde_json::Value;

use crate::cli::Outcome;
use crate::cli::out::clip;
use crate::verbs::work::crud::{docs_of, find_doc};
use crate::verbs::work::listing::{Table, cell, emit, scoped, visible};
use crate::verbs::work::run::{Run, bad};

const TESTS: Table = Table {
    head: &["test", "status", "mode", "build", "item", "thread", "proofs", "title"],
    empty: "no tests.",
};

/// `coord test show`: the test with its `build`, null when it has none.
pub(super) fn show(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let mut test = find_doc(board, "tests", r.arg(0))?;
        if let Some(fields) = test.as_object_mut() {
            fields.entry("build").or_insert(Value::Null);
        }
        r.say(&[&serde_json::to_string_pretty(&test).unwrap_or_default()]);
        Ok(())
    })
}

fn row(test: &Value, _now: i64) -> Vec<String> {
    vec![
        cell(&test["id"]),
        cell(&test["status"]),
        cell(&test["mode"]),
        cell(&test["build"]),
        cell(&test["item"]),
        cell(&test["thread"]),
        test["proofs"].as_array().map_or(0, Vec::len).to_string(),
        clip(&cell(&test["title"]), 50),
    ]
}

/// `coord test list`.
pub(super) fn list(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let mut tests: Vec<Value> = docs_of(board, "tests")
            .into_iter()
            .filter(|t| visible(t, r.options))
            .collect();
        if let Some(item) = r.text_opt("item") {
            tests.retain(|t| t["item"].as_str() == Some(item));
        }
        if r.options.json("build") == Some(Value::Bool(true)) {
            return Err(bad("--build needs a tag: coord test list --build <tag>."));
        }
        if let Some(tag) = r.text_opt("build") {
            tests.retain(|t| t["build"].as_str() == Some(tag));
        }
        let tests = scoped(board, r, tests)?;
        emit(r, tests, &TESTS, row);
        Ok(())
    })
}
