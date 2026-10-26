//! `coord inbox [--ack]` and `coord inbox list [--for T] [--ack]`: read, and with `--ack` clear.

use argus_board::json::{display, map_mut};
use serde_json::{Value, json};

use super::{target, waiting};
use crate::cli::Outcome;
use crate::cli::out::{ago, pad_end};
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

/// `coord inbox`.
pub(super) fn read(r: &mut Run<'_>) -> Outcome {
    if r.options.flag("ack") {
        return r.write(|board, r| {
            let who = r.begin(board)?;
            let thread = target(board, &who, r, "rm")?;
            print(r, &thread, &waiting(board, &thread));
            map_mut(board, "inbox").insert(thread, json!([]));
            Ok(())
        });
    }
    r.read(|board, r| {
        let who = r.who(board)?;
        let thread = target(board, &who, r, "list")?;
        let messages = waiting(board, &thread);
        print(r, &thread, &messages);
        if !messages.is_empty() && thread == who.thread {
            r.say(&["Run `coord inbox --ack` once handled."]);
        }
        Ok(())
    })
}
