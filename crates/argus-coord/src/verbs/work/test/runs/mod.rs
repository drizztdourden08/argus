//! `coord test run`: the runs of the test plans, each an item of its own (`test-run`) with its
//! plan, its build (a launcher) and its results. `coord build ...` is an alias of these verbs for
//! one release.
//!
//! ```text
//! coord test run list [--plan I-n] [--json]
//! coord test run hand <run>     coord test run unhand <run>
//! coord test run ready <run>    coord test run draft <run>
//! coord test run end <run> | reopen <run> | drop <run>
//! ```

mod end;
mod hand;
mod list;
mod rows;

pub(in crate::verbs::work::test) use rows::rows;

use argus_board::Board;
use argus_board::json::display;
use argus_board::ops::OpResult;
use serde_json::{Map, Value};

use crate::cli::{Call, Outcome};
use crate::verbs::work::crud::{apply_result, find_live, op};
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

const RUN: &str = "test-run";
const VERBS: &str = "list, hand, unhand, ready, draft, end, reopen, drop";

/// Runs the verb a word names on the words that stay in `r`.
fn dispatch(r: &mut Run<'_>, verb: &str) -> Outcome {
    match verb {
        "list" => list::list(r),
        "hand" => hand::hand(r),
        "unhand" => hand::unhand(r),
        "ready" => hand::stage(r, "ready"),
        "draft" => hand::stage(r, "draft"),
        "end" => end::end(r),
        "reopen" => end::reopen(r),
        "drop" => end::drop_run(r),
        _ => Err(bad(format!("run verb must be one of {VERBS}.")).into()),
    }
}

/// `coord test run <sub-verb> ...`.
pub(super) fn run(r: &mut Run<'_>) -> Outcome {
    let verb = r.shift().unwrap_or_default();
    dispatch(r, &verb)
}

/// An old `coord build` verb: say the new one, then do it. The words are what is left after the
/// old sub-verb.
///
/// # Errors
/// Refused as the new verb refuses.
pub fn alias(call: &mut Call<'_>, (old, new): (&str, &str)) -> Outcome {
    let mut r = Run::of(call)?;
    r.say(&[&format!(
        "coord build {old} is now coord test run {new}; the old word stays one more release."
    )]);
    dispatch(&mut r, new)
}

/// The live test run an argument names.
pub(super) fn run_item(board: &Board, id: Option<&str>) -> Res<Value> {
    let found = find_live(board, "items", id)?;
    if found["type"] != RUN {
        return Err(bad(format!(
            "{} is not a test run: coord test run list shows them.",
            display(&found["id"])
        )));
    }
    Ok(found)
}

/// Sets fields of a run (`stage`, `assignee`) as the caller.
pub(super) fn set_run(
    board: &mut Board,
    who: &Caller,
    (id, fields): (&str, Value),
    r: &Run<'_>,
) -> Res<OpResult> {
    let rest = Map::from_iter([("set".to_owned(), fields)]);
    apply_result(board, who, &op(("edit", "items", id), rest, r.options)?)
}
