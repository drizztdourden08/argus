//! `coord entry` and its verbs `ask`, `decide`, `notify`, `message` and `answer`: everything
//! threads and the owner say to each other, one doc per entry.
//!
//! | Kind | Prefix | Direction | |
//! |---|---|---|---|
//! | question | Q | thread to owner | inside an item; with options (or `--decide`) a decision request in the feed, else a free-text question; both need `--context` |
//! | decision | D | either way | a thread records its own (done); the owner's is queued (pending) |
//! | followup | F | owner to thread | more context, no action required |
//! | notice | N | thread to owner | feed only, the owner dismisses it |
//! | message | M | thread to owner | feed only, the owner replies or dismisses |
//!
//! The page only ever adds entries (answers, acks, done ticks, test results, edits); it never edits
//! a thread's doc, so the two writers never collide. A thread changes only its own entries; it
//! picks a decision only when it was routed to it.

mod answer;
mod change;
mod make;
mod read;
mod reword;

use argus_board::Board;
use argus_board::json::doc;
use argus_board::store::create_doc;
use serde_json::{Map, Value};

use super::crud::assert_live;
use super::group::{Group, nothing, serve, Soft};
use super::run::{Res, bad};
use crate::cli::{Call, Outcome};

const KINDS: [(&str, &str); 5] = [
    ("question", "Q"),
    ("followup", "F"),
    ("decision", "D"),
    ("notice", "N"),
    ("message", "M"),
];

const GROUP: Group = Group {
    name: "entry",
    verbs: &[
        ("ask", make::ask),
        ("decide", make::decide),
        ("notify", make::notify),
        ("message", make::message),
        ("answer", answer::answer),
        ("pick", change::pick),
        ("done", change::done),
        ("set", reword::set_text),
        ("move", change::move_to),
        ("show", read::show),
        ("list", read::list),
    ],
    soft: Some(Soft {
        coll: "entries",
        after: nothing,
    }),
    names: "ask, decide, notify, message, answer, pick, done, set, move, archive, unarchive, rm, restore, show, list",
    typed: Some("entry"),
};

/// A decision offers 2 to 5 choices, the recommended one first: without a choice and a field of
/// their own the owner has nothing to decide, only something to read.
const DECISION_OPTIONS: &str = "a decision needs 2 to 5 options, recommended first: --option \"<recommended>\" --option \"<other>\" (the owner can also write their own answer). Without choices it is information: use coord notify or coord message, or ask a plain question with no --decide.";

/// `coord ask | decide | notify | message | answer | entry <sub-verb> ...`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    serve(call, &GROUP)
}

fn prefix_of(kind: &str) -> Option<&'static str> {
    KINDS.iter().find(|(k, _)| *k == kind).map(|(_, p)| *p)
}

/// Questions and decisions live inside a work item; the board shows them nowhere else.
fn item_arg(board: &Board, reference: Option<&str>) -> Res<Value> {
    let Some(id) = reference.filter(|i| !i.is_empty() && *i != "-") else {
        return Err(bad(
            "name the work item this belongs to (coord item add first if there is none).",
        ));
    };
    let Some(item) = doc(board, "items", id).filter(|i| !i.is_null()) else {
        return Err(bad(format!("no item \"{id}\".")));
    };
    assert_live(item, "items")?;
    Ok(item.clone())
}

/// A new entry of a kind, with the defaults every entry carries.
fn create_entry(board: &mut Board, fields: Value, actor: &str) -> Res<String> {
    let mut entry = Map::from_iter([
        ("replyTo".to_owned(), Value::Null),
        ("item".to_owned(), Value::Null),
        ("deliveredAt".to_owned(), Value::Null),
        ("context".to_owned(), Value::Null),
    ]);
    let given = fields.as_object().cloned().unwrap_or_default();
    let prefix = given.get("kind").and_then(Value::as_str).and_then(prefix_of);
    entry.extend(given);
    Ok(create_doc(board, "entries", prefix.unwrap_or("E"), entry, actor)?)
}
