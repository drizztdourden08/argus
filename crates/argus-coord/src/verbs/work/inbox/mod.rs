//! `coord inbox`: a thread's inbox, the messages the board queued for it (answers, decisions, lock
//! grants, coordinator notes). A thread reads its own with `coord inbox [--ack]`; the coordinator
//! reads or clears another's with `--for <T>`; `send <T> "<text>"` queues a message for a thread
//! of this board and asks the dashboard, best effort, to wake it; `rm` drops messages.

mod ack_post;
mod read;
mod rm;
mod send;

use argus_board::Board;
use argus_board::ops::{Check, authorize};
use serde_json::Value;

use super::crud::{thread_arg, who_of};
use super::run::{Res, Run};
use crate::cli::{Call, Outcome};
use crate::who::Caller;

/// `coord inbox [list|send|rm] ...`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let mut run = Run::of(call)?;
    match run.arg(0) {
        Some("send") => {
            run.shift();
            send::send(&mut run)
        }
        Some("rm") => {
            run.shift();
            rm::rm(&mut run)
        }
        other => {
            if other == Some("list") {
                run.shift();
            }
            read::read(&mut run)
        }
    }
}

/// The thread whose inbox the call is about: the caller's own, or `--for <T>` (the
/// coordinator's), checked for `verb` (`list` or `rm`).
fn target(board: &Board, who: &Caller, r: &Run<'_>, verb: &str) -> Res<String> {
    let Some(name) = r.text_opt("for") else {
        return Ok(who.thread.clone());
    };
    let thread = thread_arg(board, name)?;
    if thread != who.thread {
        let found = board["threads"][name].clone();
        authorize(&who_of(who), &Check::new(verb, "inbox", Some(&found)))?;
    }
    Ok(thread)
}

/// The messages waiting for a thread.
fn waiting(board: &Board, thread: &str) -> Vec<Value> {
    board["inbox"][thread]
        .as_array()
        .cloned()
        .unwrap_or_default()
}
