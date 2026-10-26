//! `coord action` (alias `todo`): manual steps only the human can take (approve a PR, plug in a
//! device, sign in). A thread adds one, the human ticks it on the board, the checker tells the
//! thread. A thread changes only its own; the coordinator any (`--for` and `--to` name the thread).

mod add;
mod change;
mod read;

use argus_board::Board;
use argus_board::ops::refresh_needs_human;
use serde_json::Value;

use super::group::{Group, Soft, serve};
use crate::cli::{Call, Outcome};

fn refresh(board: &mut Board, action: &Value) {
    refresh_needs_human(board, action["thread"].as_str().unwrap_or_default());
}

const GROUP: Group = Group {
    name: "action",
    verbs: &[
        ("add", add::add),
        ("set", change::set),
        ("done", change::done),
        ("move", change::move_to),
        ("show", read::show),
        ("list", read::list),
    ],
    soft: Some(Soft {
        coll: "actions",
        after: refresh,
    }),
    names: "add, set, done, move, archive, unarchive, rm, restore, show, list",
    typed: None,
};

/// `coord action <sub-verb> ...`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    serve(call, &GROUP)
}
