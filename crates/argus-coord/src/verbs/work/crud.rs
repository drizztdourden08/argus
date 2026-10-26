//! What the entity verbs share (`crud.mjs`): the lookup of a doc by id, the thread an argument
//! names, the op a verb sends to the shared `apply_op`, and the refusals of its answers. Authority
//! never comes from a flag: every change goes through `authorize` and `apply_op`, so a thread
//! changes only what it owns and the coordinator changes anything.

use argus_board::Board;
use argus_board::json::{as_integer, display, doc, nullish, section, shown, truthy};
use argus_board::ops::{Check, Status, apply_op, authorize};
use serde_json::{Map, Value, json};

use super::place::rank_to;
use super::run::{Res, bad};
use crate::cli::Options;
use crate::who::Caller;

pub use crate::verbs::shared::caller::{is_coordinator, who_of};

/// The word a collection goes by in the messages.
#[must_use]
pub fn label(coll: &str) -> &str {
    match coll {
        "items" => "item",
        "tests" => "test",
        "entries" => "entry",
        "actions" => "action",
        "agents" => "agent",
        other => other,
    }
}

/// The doc of a collection by id, refused when it does not exist.
///
/// # Errors
/// Refused with `no <label> "<id>".`
pub fn find_doc(board: &Board, coll: &str, id: Option<&str>) -> Res<Value> {
    let key = id.unwrap_or_default();
    match doc(board, coll, key) {
        Some(found) if !found.is_null() => Ok(found.clone()),
        _ => Err(bad(format!("no {} \"{key}\".", label(coll)))),
    }
}

/// Refuses a doc that is in the trash.
///
/// # Errors
/// Refused with the restore hint.
pub fn assert_live(found: &Value, coll: &str) -> Res<()> {
    if truthy(found, "deletedAt") {
        let id = display(&found["id"]);
        return Err(bad(format!(
            "{id} is in the trash; `coord {} restore {id}` brings it back.",
            label(coll)
        )));
    }
    Ok(())
}

/// The doc of a collection by id, refused when it does not exist or is in the trash.
///
/// # Errors
/// As [`find_doc`] and [`assert_live`].
pub fn find_live(board: &Board, coll: &str, id: Option<&str>) -> Res<Value> {
    let found = find_doc(board, coll, id)?;
    assert_live(&found, coll)?;
    Ok(found)
}

/// A live doc the caller may do `verb` to.
///
/// # Errors
/// As [`find_live`], or as `authorize` refuses.
pub fn checked(
    board: &Board,
    who: &Caller,
    (coll, verb): (&str, &str),
    id: Option<&str>,
) -> Res<Value> {
    let found = find_live(board, coll, id)?;
    authorize(&who_of(who), &Check::new(verb, coll, Some(&found)))?;
    Ok(found)
}

/// The id of a thread an argument names, refused when it is not on the board.
///
/// # Errors
/// Refused with `no thread "<name>".`
pub fn thread_arg(board: &Board, name: &str) -> Res<String> {
    match doc(board, "threads", name) {
        Some(thread) if !thread.is_null() => {
            Ok(nullish(thread, "id").unwrap_or_else(|| "undefined".to_owned()))
        }
        _ => Err(bad(format!("no thread \"{name}\"."))),
    }
}

/// The thread a new record belongs to: `--for <T>` (the coordinator's, `authorize` refuses anyone
/// else) or the caller's own thread.
///
/// # Errors
/// Refused when `--for` names a thread that is not on the board.
pub fn owner_arg(board: &Board, who: &Caller, options: &Options) -> Res<String> {
    match options.text("for") {
        Some(name) => thread_arg(board, name),
        None => Ok(who.thread.clone()),
    }
}

/// An op for `apply_op`. The CLI is last-writer unless `--if-rev <n>` asks for the check.
///
/// # Errors
/// Refused when `--if-rev` is not a whole number.
pub fn op(
    (kind, coll, id): (&str, &str, &str),
    rest: Map<String, Value>,
    options: &Options,
) -> Res<Value> {
    let mut out = Map::new();
    out.insert("kind".into(), json!(kind));
    out.insert("target".into(), json!({"c": coll, "id": id}));
    out.extend(rest);
    match options.json("if-rev") {
        None => out.insert("force".into(), json!(true)),
        Some(given) => {
            let Some(rev) = as_integer(Some(&given)) else {
                return Err(bad(
                    "--if-rev takes the revision number you read (rev in `show`).",
                ));
            };
            out.insert("baseRev".into(), json!(rev))
        }
    };
    Ok(Value::Object(out))
}

/// An op that moves a record to a place or a thread.
///
/// # Errors
/// As [`op`].
pub fn move_op(coll: &str, id: &str, to: Map<String, Value>, options: &Options) -> Res<Value> {
    op(
        ("move", coll, id),
        Map::from_iter([("to".to_owned(), Value::Object(to))]),
        options,
    )
}

/// Applies an op as the caller.
///
/// # Errors
/// Refused with the outcome of a refused op, or the conflict sentence of a changed record.
pub fn apply(board: &mut Board, who: &Caller, operation: &Value) -> Res<()> {
    let res = apply_op(board, operation, &who_of(who), None);
    match res.status {
        Status::Applied => Ok(()),
        Status::Refused => Err(bad(res.outcome)),
        Status::Conflict => Err(bad(format!(
            "{} changed since rev {} (now {}). Read it again and retry.",
            display(&operation["target"]["id"]),
            display(&operation["baseRev"]),
            serde_json::to_string(&res.current.unwrap_or_else(|| json!({}))).unwrap_or_default()
        ))),
    }
}

/// The revision of a doc as a message prints it.
#[must_use]
pub fn rev_of(board: &Board, coll: &str, id: &str) -> String {
    shown(doc(board, coll, id).and_then(|d| d.get("rev")))
}

/// The docs of a collection, in order.
#[must_use]
pub fn docs_of(board: &Board, coll: &str) -> Vec<Value> {
    section(board, coll).values().cloned().collect()
}

/// The `to` of a move that names a thread: `--to <T>`.
#[must_use]
pub fn move_target(options: &Options) -> Map<String, Value> {
    let mut to = Map::new();
    if let Some(thread) = options.text("to") {
        to.insert("thread".into(), json!(thread));
    }
    to
}

/// Moves an item or action: the doc must be live, a named thread must exist, the caller must be
/// allowed, the placement flags join `to`, and the shared op applies it. Returns the doc as it
/// was before and its id.
///
/// # Errors
/// As the checks and the op refuse.
pub fn move_doc(
    board: &mut Board,
    who: &Caller,
    (coll, id): (&str, Option<&str>),
    (mut to, options): (Map<String, Value>, &Options),
) -> Res<(Value, String)> {
    let before = find_live(board, coll, id)?;
    let key = display(&before["id"]);
    let thread = to.get("thread").and_then(Value::as_str).map(str::to_owned);
    if let Some(name) = &thread {
        thread_arg(board, name)?;
    }
    let check = Check::new("move", coll, Some(&before)).to(thread.as_deref());
    authorize(&who_of(who), &check)?;
    if let Some(place) = rank_to(board, (coll, &key), options)? {
        to.extend(place);
    }
    apply(board, who, &move_op(coll, &key, to, options)?)?;
    Ok((before, key))
}
