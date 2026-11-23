//! `coord item`: work items, what a thread has been asked to do, with short acceptance criteria.
//! A thread enters and edits its own items; the coordinator can do all of it for any thread
//! (`--for` on add, `--to` on move). An item may be typed (`--type`, `--parent`, `--assignee`,
//! `--stage`, `--field`): the board checks it against the registry, and a call that names none of
//! them is the plain item it always was.

mod ac;
mod add;
mod block;
mod edit;
mod fields;
mod filters;
mod force;
mod images;
mod marks;
mod migrate;
mod migrate_assignees;
mod read;
mod served_line;
mod set;
mod tree;
mod typed;

use argus_board::Board;
use serde_json::Value;

use super::group::{Group, Soft, serve};
use crate::cli::{Call, Outcome};

const STATUSES: [&str; 5] = ["todo", "doing", "review", "done", "dropped"];

const GROUP: Group = Group {
    name: "item",
    verbs: &[
        ("add", add::add),
        ("set", set::set),
        ("status", set::status),
        ("block", block::block),
        ("unblock", block::unblock),
        ("move", set::move_to),
        ("ac", ac::ac),
        ("ac-done", ac::ac_done),
        ("ac-rm", ac::ac_rm),
        ("show", read::show),
        ("list", read::list),
        ("tree", tree::tree),
        ("migrate-blockers", migrate::migrate),
        ("migrate-assignees", migrate_assignees::migrate),
    ],
    soft: Some(Soft {
        coll: "items",
        after: |_: &mut Board, _: &Value| {},
    }),
    names: "add, set, status, block, unblock, move, ac, ac-done, ac-rm, archive, unarchive, rm, restore, show, list, tree, migrate-blockers, migrate-assignees",
    typed: None,
};

/// `coord item <sub-verb> ...`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    serve(call, &GROUP)
}
