//! `coord lock claim <resource> [--desc "..."] [--agent A-n]`, `release <resource> [--agent A-n]`
//! and `drop <resource> [--agent A-n]`.

use argus_board::Board;
use argus_board::json::{array_mut, doc, doc_mut, map_mut, nullish, set};
use argus_board::limits::optional_text;
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::agent::{self, entry_of, holder_agent, holder_text};
use super::{find_lock, hand_on, holder_of, may, resource_arg, touched};
use crate::cli::Outcome;
use crate::cli::out::ago;
use crate::verbs::work::keys::js_order;
use crate::verbs::work::run::{Res, Run, bad};

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

fn take(board: &mut Board, resource: &str, me: &str, agent: Option<&str>) {
    if let Some(lock) = doc_mut(board, "locks", resource) {
        set(lock, "holder", me);
        set(lock, "holderAgent", json!(agent));
        set(lock, "since", now_iso());
    }
}

/// Queues the entry once and returns its place, counted from 1.
fn enqueue(board: &mut Board, resource: &str, entry: &str) -> usize {
    let Some(lock) = doc_mut(board, "locks", resource) else {
        return 0;
    };
    let queue = array_mut(lock, "queue");
    if !queue.iter().any(|id| id.as_str() == Some(entry)) {
        queue.push(json!(entry));
    }
    queue
        .iter()
        .position(|id| id.as_str() == Some(entry))
        .map_or(0, |i| i + 1)
}

/// `coord lock claim`: held at once when free, else queued.
pub(super) fn claim(r: &mut Run<'_>) -> Outcome {
    let resource = resource_arg(r.arg(0))?;
    let agent = agent::label(r);
    r.write(|board, r| {
        let me = r.begin(board)?.thread;
        open_lock(board, &resource);
        describe(board, &resource, optional_text("description", &r.opt("desc"))?);
        let lock = find_lock(board, &resource)?;
        let since = ago(lock["since"].as_str(), r.now_ms());
        match holder_of(&lock) {
            Some(holder) if holder == me && holder_agent(&lock) == agent => {
                r.say(&[&format!("{resource}: you already hold it (since {since}).")]);
            }
            None => {
                take(board, &resource, &me, agent.as_deref());
                let flag = agent
                    .as_ref()
                    .map_or_else(String::new, |a| format!(" --agent {a}"));
                r.say(&[&format!(
                    "{resource}: held by you. Release it with `coord lock release {resource}{flag}` when done."
                )]);
            }
            Some(holder) => {
                let place = enqueue(board, &resource, &entry_of(&me, agent.as_deref()));
                let holder = holder_text(&lock).unwrap_or(holder);
                r.say(&[
                    &format!("{resource}: held by {holder} (since {since}). You are #{place} in the queue."),
                    "You will get an inbox message when it is yours; carry on with other work meanwhile.",
                ]);
            }
        }
        Ok(())
    })
}

/// Refuses a release by an agent other than the one holding the lock.
fn same_agent(resource: &str, lock: &Value, agent: Option<&str>) -> Res<()> {
    match (agent, holder_agent(lock)) {
        (Some(a), Some(h)) if a != h => {
            Err(bad(format!("{resource}: held by agent {h}, not {a}.")))
        }
        _ => Ok(()),
    }
}

/// `coord lock release`: hands the lock to the head of the queue.
pub(super) fn release(r: &mut Run<'_>) -> Outcome {
    let resource = resource_arg(r.arg(0))?;
    let agent = agent::label(r);
    r.write(|board, r| {
        let who = r.begin(board)?;
        let lock = doc(board, "locks", &resource).cloned().unwrap_or(Value::Null);
        if holder_of(&lock).is_none() {
            return Err(bad(format!("{resource}: nobody holds it.")));
        }
        may(&who, "release", &lock)?;
        same_agent(&resource, &lock, agent.as_deref())?;
        hand_on(board, &resource, None, &who.actor);
        let now = find_lock(board, &resource)?;
        let Some(next) = holder_of(&now) else {
            r.say(&[&format!("{resource}: released, nobody waiting.")]);
            return Ok(());
        };
        let session = doc(board, "threads", &next)
            .and_then(|t| nullish(t, "session"))
            .unwrap_or_else(|| next.clone());
        let shown = holder_text(&now).unwrap_or(next);
        r.say(&[
            &format!("{resource}: passed to {shown}."),
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
    let agent = agent::label(r);
    r.write(|board, r| {
        let me = r.begin(board)?.thread;
        find_lock(board, &resource)?;
        let mine = entry_of(&me, agent.as_deref());
        if let Some(lock) = doc_mut(board, "locks", &resource) {
            array_mut(lock, "queue").retain(|id| id.as_str() != Some(mine.as_str()));
        }
        touched(board, &resource);
        r.say(&[&format!("{resource}: you left the queue.")]);
        Ok(())
    })
}
