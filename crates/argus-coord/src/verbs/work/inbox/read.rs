//! `coord inbox [--ack]` and `coord inbox list [--for T] [--ack]`: read, and with `--ack` clear.

use argus_board::json::{display, map_mut};
use serde_json::{Value, json};

use super::ack_post::{Read, post};
use super::{target, waiting};
use crate::cli::out::{ago, pad_end};
use crate::cli::{CoordError, Outcome};
use crate::verbs::work::run::Run;

fn message_line(m: &Value, now: i64) -> String {
    let id = m["id"]
        .as_str()
        .filter(|i| !i.is_empty())
        .map_or_else(String::new, |i| format!("  ({i})"));
    format!(
        "  {} [{}] from {}: {}{id}",
        pad_end(&ago(m["ts"].as_str(), now), 8),
        display(&m["kind"]),
        display(&m["from"]),
        display(&m["text"])
    )
}

/// What `coord inbox` prints for a thread: one line per call of `say`.
fn print(r: &mut Run<'_>, thread: &str, messages: &[Value]) {
    if messages.is_empty() {
        r.say(&[&format!("{thread}: inbox empty.")]);
        return;
    }
    r.say(&[&format!("{thread}: {} message(s)", messages.len())]);
    let now = r.now_ms();
    for m in messages {
        r.say(&[&message_line(m, now)]);
    }
}

/// `coord inbox --ack`: print, then clear. Answers the read when it was the caller's own inbox.
fn take(r: &mut Run<'_>) -> Result<Option<Read>, CoordError> {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let thread = target(board, &who, r, "rm")?;
        let messages = waiting(board, &thread);
        print(r, &thread, &messages);
        map_mut(board, "inbox").insert(thread.clone(), json!([]));
        Ok((thread == who.thread).then(|| Read::of(&who, &messages)))
    })
}

/// `coord inbox`: print only. Answers the read when it was the caller's own inbox.
fn show(r: &mut Run<'_>) -> Result<Option<Read>, CoordError> {
    r.read(|board, r| {
        let who = r.who(board)?;
        let thread = target(board, &who, r, "list")?;
        let messages = waiting(board, &thread);
        print(r, &thread, &messages);
        let own = thread == who.thread;
        if !messages.is_empty() && own {
            r.say(&["Run `coord inbox --ack` once handled."]);
        }
        Ok(own.then(|| Read::of(&who, &messages)))
    })
}

/// `coord inbox`. A read of the caller's own inbox is told to Argus afterwards, best effort.
pub(super) fn read(r: &mut Run<'_>) -> Outcome {
    let own = if r.options.flag("ack") {
        take(r)
    } else {
        show(r)
    }?;
    if let Some(read) = own {
        post(r, &read);
    }
    Ok(())
}
