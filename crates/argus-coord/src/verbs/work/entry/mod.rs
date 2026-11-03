//! `coord entry` and its verbs `ask`, `decide`, `notify`, `message` and `answer`: everything
//! threads and the owner say to each other, one doc per entry.
//!
//! | Kind | Prefix | Direction | |
//! |---|---|---|---|
//! | question | Q | thread to owner | inside an item; with options (or `--decide`) a decision request in the feed, else a free-text question; both need `--context` |
//! | decision | D | either way | a thread records its own (done); the owner's is queued (pending) |
//! | followup | F | owner to thread | more context, no action required |
//! | notice | N | thread to owner | feed only, the owner dismisses it |
//! | note | N | a thread on its own record | `coord note <I-n|T-n> "<text>"`, an informal remark, never blocks |
//! | message | M | thread to owner | feed only, the owner replies or dismisses |
//!
//! The page only ever adds entries (answers, acks, done ticks, test results, edits); it never edits
//! a thread's doc, so the two writers never collide. A thread changes only its own entries; it
//! picks a decision only when it was routed to it.

mod answer;
mod change;
mod images;
mod make;
mod note;
mod read;
mod reword;

use argus_board::Board;
use argus_board::json::doc;
use argus_board::ops::{Check, authorize};
use argus_board::store::create_doc;
use serde_json::{Map, Value};

use super::crud::{assert_live, label, stored_id, who_of};
use super::group::{Group, Soft, nothing, serve};
use super::run::{Res, Run, bad};
use crate::cli::{Call, Outcome};
use crate::who::Caller;

const KINDS: [(&str, &str); 6] = [
    ("question", "Q"),
    ("followup", "F"),
    ("decision", "D"),
    ("notice", "N"),
    ("message", "M"),
    ("note", "N"),
];

const GROUP: Group = Group {
    name: "entry",
    verbs: &[
        ("ask", make::ask),
        ("decide", make::decide),
        ("notify", make::notify),
        ("message", make::message),
        ("note", note::note),
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
    names: "ask, decide, notify, message, note, answer, pick, done, set, move, archive, unarchive, rm, restore, show, list",
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

/// Questions and decisions live inside a work item (or a test, `T-n`); the board shows them
/// nowhere else.
fn item_arg(board: &Board, reference: Option<&str>) -> Res<Value> {
    let Some(id) = reference.filter(|i| !i.is_empty() && *i != "-") else {
        return Err(bad(
            "name the work item this belongs to (coord item add first if there is none).",
        ));
    };
    let coll = if id.starts_with("T-") {
        "tests"
    } else {
        "items"
    };
    let id = stored_id(board, coll, id)?;
    let Some(item) = doc(board, coll, &id).filter(|i| !i.is_null()) else {
        return Err(bad(format!("no {} \"{id}\".", label(coll))));
    };
    assert_live(item, coll)?;
    Ok(item.clone())
}

/// The caller and the live item a question or decision attaches to. Only the thread that owns
/// the item asks inside it (the coordinator can for anyone).
fn on_item(board: &mut Board, r: &Run<'_>) -> Res<(Caller, Value)> {
    let who = r.begin(board)?;
    let item = item_arg(board, r.arg(0))?;
    let check = Check::new("add", "entries", None).to(item["thread"].as_str());
    authorize(&who_of(&who), &check)?;
    Ok((who, item))
}

/// A new entry of a kind, with the defaults every entry carries.
fn create_entry(board: &mut Board, fields: &Value, actor: &str) -> Res<String> {
    let mut entry = Map::from_iter([
        ("replyTo".to_owned(), Value::Null),
        ("item".to_owned(), Value::Null),
        ("deliveredAt".to_owned(), Value::Null),
        ("context".to_owned(), Value::Null),
    ]);
    let given = fields.as_object().cloned().unwrap_or_default();
    let prefix = given
        .get("kind")
        .and_then(Value::as_str)
        .and_then(prefix_of);
    entry.extend(given);
    create_doc(board, "entries", prefix.unwrap_or("E"), entry, actor)
}
