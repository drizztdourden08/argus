//! The rank placement flags of `crud.mjs`: `--before <id>`, `--after <id>`, `--top`, `--bottom`
//! as the `to` of a move op, and the order the lists print in.

use std::cmp::Ordering;

use argus_board::Board;
use argus_board::json::{display, doc, truthy};
use serde_json::{Map, Value};

use super::crud::{docs_of, label};
use super::run::{Res, bad};
use crate::cli::Options;

/// Orders by rank, a missing rank counting as 0. Equal ranks keep their order.
#[must_use]
pub fn by_rank(a: &Value, b: &Value) -> Ordering {
    let rank = |d: &Value| d["rank"].as_f64().unwrap_or(0.0);
    rank(a).partial_cmp(&rank(b)).unwrap_or(Ordering::Equal)
}

/// Whether the call places the record in the order.
#[must_use]
pub fn has_placement(options: &Options) -> bool {
    options.text("before").is_some()
        || options.text("after").is_some()
        || options.flag("top")
        || options.flag("bottom")
}

fn one(key: &str, id: &str) -> Map<String, Value> {
    Map::from_iter([(key.to_owned(), Value::from(id))])
}

fn ranked_except(board: &Board, coll: &str, id: &str) -> Vec<Value> {
    let mut docs: Vec<Value> = docs_of(board, coll)
        .into_iter()
        .filter(|d| display(&d["id"]) != id && !truthy(d, "deletedAt"))
        .collect();
    docs.sort_by(by_rank);
    docs
}

/// The place the flags ask for as the `to` of a move op, `None` when they ask for none.
///
/// # Errors
/// Refused when `--before` or `--after` names a record that does not exist.
pub fn rank_to(
    board: &Board,
    (coll, id): (&str, &str),
    options: &Options,
) -> Res<Option<Map<String, Value>>> {
    for key in ["before", "after"] {
        let Some(target) = options.text(key) else {
            continue;
        };
        if doc(board, coll, target).is_none_or(Value::is_null) {
            return Err(bad(format!(
                "no {} \"{target}\" to place it {key}.",
                label(coll)
            )));
        }
        return Ok(Some(one(key, target)));
    }
    let ranked = ranked_except(board, coll, id);
    if options.flag("top") {
        return Ok(ranked.first().map(|d| one("before", &display(&d["id"]))));
    }
    if options.flag("bottom") {
        return Ok(ranked.last().map(|d| one("after", &display(&d["id"]))));
    }
    Ok(None)
}
