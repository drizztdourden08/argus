//! What the list verbs share: which docs are visible, the `--mine` and `--for` scopes, the order,
//! and the choice between `--json` and an aligned table.

use argus_board::Board;
use argus_board::json::{display, truthy};
use serde_json::Value;

use super::place::by_rank;
use super::run::{Res, Run};
use crate::cli::out::{as_json, table};
use crate::cli::Options;

/// The columns and the empty message of a list.
pub struct Table {
    /// The header row.
    pub head: &'static [&'static str],
    /// What prints when nothing is listed.
    pub empty: &'static str,
}

/// A doc a list shows: the live ones by default, only the archived with `--archived`, only the
/// trash with `--deleted`.
#[must_use]
pub fn visible(found: &Value, options: &Options) -> bool {
    if options.flag("deleted") {
        return truthy(found, "deletedAt");
    }
    if truthy(found, "deletedAt") {
        return false;
    }
    if options.flag("archived") {
        truthy(found, "archivedAt")
    } else {
        !truthy(found, "archivedAt")
    }
}

/// Narrows docs to the caller's own with `--mine`, or to a thread with `--for <T>`.
///
/// # Errors
/// Refused as the caller lookup refuses, for `--mine`.
pub fn scoped(board: &Board, r: &Run<'_>, docs: Vec<Value>) -> Res<Vec<Value>> {
    let thread = if r.options.flag("mine") {
        r.who(board)?.thread
    } else if let Some(name) = r.text_opt("for") {
        name.to_owned()
    } else {
        return Ok(docs);
    };
    Ok(docs
        .into_iter()
        .filter(|d| d["thread"].as_str() == Some(thread.as_str()))
        .collect())
}

/// A cell: nothing for `null`, else the value as a template literal prints it.
#[must_use]
pub fn cell(v: &Value) -> String {
    if v.is_null() { String::new() } else { display(v) }
}

/// Prints docs in rank order, as JSON with `--json`, else as a table of `row` lines.
pub fn emit(
    r: &mut Run<'_>,
    mut docs: Vec<Value>,
    layout: &Table,
    row: impl Fn(&Value, i64) -> Vec<String>,
) {
    docs.sort_by(by_rank);
    if r.options.flag("json") {
        as_json(&mut *r.inv.stdout, &Value::Array(docs));
        return;
    }
    let now = r.now_ms();
    let mut rows: Vec<Vec<String>> = docs.iter().map(|d| row(d, now)).collect();
    if rows.is_empty() {
        r.say(&[layout.empty]);
        return;
    }
    rows.insert(0, layout.head.iter().map(|h| (*h).to_owned()).collect());
    r.say(&[&table(&rows)]);
}
