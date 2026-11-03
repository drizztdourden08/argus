//! `coord action` (alias `todo`): manual steps only the human can take (approve a PR, plug in a
//! device, sign in). A thread adds one, the human ticks it on the board, the checker tells the
//! thread. A thread changes only its own; the coordinator any (`--for` and `--to` name the thread).
//! A to-do may be about an item (`I-n`) or a test (`T-n`) and may be for anyone: `--who` names the
//! owner (the default), a thread of the board or `lead:<key>`; the assignee can tick it too.

mod add;
mod change;
mod read;

use argus_board::Board;
use argus_board::content::{Context, check};
use argus_board::json::ids;
use argus_board::ops::{record_arg, refresh_needs_human};
use serde_json::{Value, json};

use super::crud::{assert_live, find_doc};
use super::group::{Group, Soft, serve};
use super::run::{Res, Run};
use crate::cli::{Call, Outcome};

/// The to-do is about a record that exists and is not in the trash: an item, or a test (`T-n`).
fn live_record(board: &Board, id: &str) -> Res<()> {
    if id.starts_with("T-") {
        return record_arg(board, &json!(id)).map(|_| ());
    }
    assert_live(&find_doc(board, "items", Some(id))?, "items")
}

/// `--who`: the owner, a thread of the board or `lead:<key>`. `None` when it is not given.
fn assignee_arg(board: &Board, r: &Run<'_>) -> Res<Option<Value>> {
    let Some(given) = r.options.json("who") else {
        return Ok(None);
    };
    let threads = ids(board, "threads");
    let cx = Context {
        threads: Some(threads.as_slice()),
        ..Context::default()
    };
    let who = check("todoAssignee", &given, &cx)?;
    Ok((!who.is_null()).then_some(who))
}

/// Who a to-do is for: its assignee, the owner when it has none.
fn assignee_of(action: &Value) -> &str {
    action["assignee"].as_str().unwrap_or("owner")
}

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
