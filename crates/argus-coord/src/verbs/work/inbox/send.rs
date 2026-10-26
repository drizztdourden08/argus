//! `coord inbox send <T> "<text>" [--kind k]`: queues a message for a thread of this board, then
//! asks the dashboard to wake the thread (`POST /api/p/:key/wakes`, reason `thread-message`).
//! Best effort: the message is queued either way, so a dashboard that is down, or has no such
//! thread, never fails the verb.

use argus_board::inbox::post;
use argus_board::json::{doc, nullish};
use argus_board::limits::require_text;
use argus_core::js;
use serde_json::json;

use crate::cli::Outcome;
use crate::client::dashboard_client;
use crate::verbs::work::crud::thread_arg;
use crate::verbs::work::run::Run;

const WAKE_CLIP: usize = 100;

/// A text on one line: runs of white space become one space, the ends are trimmed.
fn one_line(text: &str) -> String {
    text.split(|c: char| (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}')
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Asks the dashboard to wake the thread. Only a nudge: every failure is ignored.
fn wake(r: &Run<'_>, thread: &str, from: &str, text: &str) {
    let paths = r.inv.env.paths(&r.inv.cwd);
    let Ok(client) = dashboard_client(&r.inv.env, &paths, false) else {
        return;
    };
    let line = format!("{from}: {}", js::slice(&one_line(text), WAKE_CLIP));
    let body = json!({"thread": thread, "reason": "thread-message", "text": line});
    client
        .post(&format!("/api/p/{}/wakes", r.project), &body)
        .ok();
}

/// `coord inbox send`.
pub(super) fn send(r: &mut Run<'_>) -> Outcome {
    let text = require_text("followup", &r.arg_json(1))?;
    let kind = r.text_opt("kind").unwrap_or("note").to_owned();
    let woken = r.write(|board, r| {
        let who = r.begin(board)?;
        let to = thread_arg(board, r.arg(0).unwrap_or("undefined"))?;
        post(board, &to, &who.actor, (&kind, &text), None);
        let session = doc(board, "threads", &to)
            .and_then(|t| nullish(t, "session"))
            .unwrap_or_else(|| "?".to_owned());
        r.say(&[&format!("queued for {to} (session \"{session}\"): {text}")]);
        Ok((to != who.thread).then_some((to, who.actor)))
    })?;
    if let Some((thread, from)) = woken {
        wake(r, &thread, &from, &text);
    }
    Ok(())
}
