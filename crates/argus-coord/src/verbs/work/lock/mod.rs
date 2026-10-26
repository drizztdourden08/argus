//! `coord lock`: resource locks with a queue. A claim on a free resource holds it at once; a
//! claim on a held one queues the caller. A release hands the resource to the queue head and
//! tells the releasing thread who to message so the next one starts right away. The coordinator
//! can also free a lock by force (`free [--to T]`) or delete it (`rm`).

mod force;
mod hold;
mod read;

use argus_board::Board;
use argus_board::inbox::post;
use argus_board::json::{display, doc, doc_mut, set};
use argus_board::ops::{Check, authorize};
use argus_board::store::touch;
use argus_board::text::slug;
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Value, json};

use super::crud::who_of;
use super::group::{Group, serve};
use super::run::{Res, Run, bad};
use crate::cli::{Call, Outcome};
use crate::who::Caller;

const GROUP: Group = Group {
    name: "lock",
    verbs: &[
        ("claim", hold::claim),
        ("release", hold::release),
        ("drop", hold::drop_queue),
        ("free", force::free),
        ("rm", force::rm),
        ("list", read::list),
    ],
    soft: None,
    names: "claim, release, drop, free, rm, list",
    typed: None,
};

/// `coord lock <sub-verb> ...`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    serve(call, &GROUP)
}

/// The resource a word names: a slug, refused when empty.
fn resource_arg(word: Option<&str>) -> Res<String> {
    let name = slug(word.unwrap_or_default());
    if name.is_empty() {
        return Err(bad(
            "resource name is required (e.g. wasm-build, test-rom, android-avd).",
        ));
    }
    Ok(name)
}

/// The lock of a resource, refused when there is none.
fn find_lock(board: &Board, resource: &str) -> Res<Value> {
    match doc(board, "locks", resource) {
        Some(lock) if !lock.is_null() => Ok(lock.clone()),
        _ => Err(bad(format!("no lock \"{resource}\"."))),
    }
}

/// A change to a lock that exists, which the caller may do `verb` to.
fn on_lock(
    r: &mut Run<'_>,
    verb: &str,
    change: impl FnOnce(&mut Board, &mut Run<'_>, (&Caller, &str, &Value)) -> Res<()>,
) -> Outcome {
    let resource = resource_arg(r.arg(0))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let lock = find_lock(board, &resource)?;
        may(&who, verb, &lock)?;
        change(board, r, (&who, &resource, &lock))
    })
}

/// Refuses unless the caller may do `verb` to the lock.
fn may(who: &Caller, verb: &str, lock: &Value) -> Res<()> {
    authorize(&who_of(who), &Check::new(verb, "locks", Some(lock)))
}

/// Stamps a lock as changed now.
fn touched(board: &mut Board, resource: &str) {
    if let Some(lock) = doc_mut(board, "locks", resource) {
        set(lock, "updatedAt", now_iso());
    }
    touch(board, "locks", resource);
}

/// The thread holding a lock, `None` when it is free.
fn holder_of(lock: &Value) -> Option<String> {
    js::truthy(&lock["holder"]).then(|| display(&lock["holder"]))
}

/// The ids of a lock's queue.
fn queue_of(lock: &Value) -> Vec<Value> {
    lock["queue"].as_array().cloned().unwrap_or_default()
}

/// Hands the lock to `next`, or to the head of the queue; none leaves it free. The new holder is
/// told.
fn hand_on(board: &mut Board, resource: &str, next: Option<&str>, from: &str) {
    let Some(lock) = doc_mut(board, "locks", resource) else {
        return;
    };
    let mut queue = queue_of(lock);
    if let Some(next) = next {
        queue.retain(|id| id.as_str() != Some(next));
    }
    let holder = match next {
        Some(next) => json!(next),
        None if queue.is_empty() => Value::Null,
        None => queue.remove(0),
    };
    set(lock, "queue", Value::Array(queue));
    set(lock, "holder", holder.clone());
    set(
        lock,
        "since",
        if holder.is_null() {
            Value::Null
        } else {
            json!(now_iso())
        },
    );
    touched(board, resource);
    if let Some(holder) = holder.as_str() {
        let text = format!("{resource} is yours now (released by {from}).");
        post(board, holder, from, ("lock-granted", &text), Some(resource));
    }
}
