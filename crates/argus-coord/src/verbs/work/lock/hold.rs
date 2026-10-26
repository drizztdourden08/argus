//! `coord lock claim <resource> [--desc "..."]`, `release <resource>` and `drop <resource>`.

use argus_board::Board;
use argus_board::json::{array_mut, doc, doc_mut, map_mut, nullish, set};
use argus_board::limits::optional_text;
use argus_board::ops::{Check, authorize};
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::{find_lock, hand_on, holder_of, resource_arg, touched};
use crate::cli::Outcome;
use crate::cli::out::ago;
use crate::verbs::work::crud::who_of;
use crate::verbs::work::keys::js_order;
use crate::verbs::work::run::{Run, bad};

fn open_lock(board: &mut Board, resource: &str) {
    let locks = map_mut(board, "locks");
    if locks.get(resource).is_none_or(Value::is_null) {
        let fresh = json!({
            "resource": resource, "description": null, "holder": null, "since": null,
            "queue": [], "updatedAt": now_iso(),
        });
        locks.insert(resource.to_owned(), fresh);
        js_order(locks);
    }
}

fn describe(board: &mut Board, resource: &str, description: Option<String>) {
    if let (Some(lock), Some(text)) = (doc_mut(board, "locks", resource), description) {
        set(lock, "description", text);
    }
    touched(board, resource);
}

fn take(board: &mut Board, resource: &str, me: &str) {
    if let Some(lock) = doc_mut(board, "locks", resource) {
        set(lock, "holder", me);
        set(lock, "since", now_iso());
    }
}

/// Queues the caller once and returns its place, counted from 1.
fn enqueue(board: &mut Board, resource: &str, me: &str) -> usize {
    let Some(lock) = doc_mut(board, "locks", resource) else {
        return 0;
    };
    let queue = array_mut(lock, "queue");
    if !queue.iter().any(|id| id.as_str() == Some(me)) {
        queue.push(json!(me));
    }
    queue.iter().position(|id| id.as_str() == Some(me)).map_or(0, |i| i + 1)
}

/// `coord lock claim`: held at once when free, else queued.
pub(super) fn claim(r: &mut Run<'_>) -> Outcome {
    let resource = resource_arg(r.arg(0))?;
    r.write(|board, r| {
        let me = r.begin(board)?.thread;
        open_lock(board, &resource);
        describe(board, &resource, optional_text("description", &r.opt("desc"))?);
        let lock = find_lock(board, &resource)?;
        let since = ago(lock["since"].as_str(), r.now_ms());
        match holder_of(&lock) {
            Some(holder) if holder == me => {
                r.say(&[&format!("{resource}: you already hold it (since {since}).")]);
            }
            None => {
                take(board, &resource, &me);
                r.say(&[&format!(
                    "{resource}: held by you. Release it with `coord lock release {resource}` when done."
                )]);
            }
            Some(holder) => {
                let place = enqueue(board, &resource, &me);
                r.say(&[
                    &format!("{resource}: held by {holder} (since {since}). You are #{place} in the queue."),
                    "You will get an inbox message when it is yours; carry on with other work meanwhile.",
                ]);
            }
        }
        Ok(())
    })
}

/// `coord lock release`: hands the lock to the head of the queue.
pub(super) fn release(r: &mut Run<'_>) -> Outcome {
    let resource = resource_arg(r.arg(0))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let lock = doc(board, "locks", &resource).cloned().unwrap_or(Value::Null);
        if holder_of(&lock).is_none() {
            return Err(bad(format!("{resource}: nobody holds it.")));
        }
        authorize(&who_of(&who), &Check::new("release", "locks", Some(&lock)))?;
        hand_on(board, &resource, None, &who.actor);
        let Some(next) = holder_of(&find_lock(board, &resource)?) else {
            r.say(&[&format!("{resource}: released, nobody waiting.")]);
            return Ok(());
        };
        let session = doc(board, "threads", &next)
            .and_then(|t| nullish(t, "session"))
            .unwrap_or_else(|| next.clone());
        r.say(&[
            &format!("{resource}: passed to {next}."),
            &format!(
                "Send them a message now (SendMessage to \"{session}\"): \"{resource} is yours, run coord inbox\"."
            ),
        ]);
        Ok(())
    })
}

/// `coord lock drop`: leaves the queue.
pub(super) fn drop_queue(r: &mut Run<'_>) -> Outcome {
    let resource = resource_arg(r.arg(0))?;
    r.write(|board, r| {
        let me = r.begin(board)?.thread;
        find_lock(board, &resource)?;
        if let Some(lock) = doc_mut(board, "locks", &resource) {
            array_mut(lock, "queue").retain(|id| id.as_str() != Some(me.as_str()));
        }
        touched(board, &resource);
        r.say(&[&format!("{resource}: you left the queue.")]);
        Ok(())
    })
}
