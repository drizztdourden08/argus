//! A thread going out of sight: `thread pause` and `resume` (yourself, or any thread when the
//! lead asks for the owner), `thread archive` and `unarchive` (a thread whose session is
//! gone: its items, questions, decisions, actions and tests stay visible).

use argus_board::Board;
use argus_board::json::{display, nullish};
use argus_board::limits::optional_text;
use argus_board::ops::{Status, apply_op, check_authority};
use argus_board::pause::{pause_thread, resume_thread};
use argus_board::store::stamp;
use serde_json::{Value, json};

use super::target;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::thread::frame::{Frame, is_lead, who_of};
use crate::verbs::thread::passed::{lock_holders, say_passed};
use crate::who::Caller;

/// Who the thread is told did it: the lead's session when it acts for the owner.
fn by_whom(board: &Board, who: &Caller, id: &str) -> String {
    if !is_lead(who) {
        return id.to_owned();
    }
    let session = nullish(&board["threads"][who.actor.as_str()], "session")
        .unwrap_or_else(|| "the lead".to_owned());
    format!("{session} (for the owner)")
}

fn toggle(frame: &mut Frame<'_>, verb: &str, past: &str) -> Outcome {
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        let id = target(board, fr, &who)?;
        check_authority(&who_of(&who), verb, "threads", board["threads"].get(&id))?;
        let by = by_whom(board, &who, &id);
        let changed = if verb == "pause" {
            pause_thread(board, &id, &by)?
        } else {
            resume_thread(board, &id, &by)
        };
        if !changed {
            let state = if board["threads"][id.as_str()]["status"] == "paused" {
                "paused"
            } else {
                "not paused"
            };
            fr.say(&[&format!("{id} is already {state}.")]);
            return Ok(());
        }
        stamp(board, "threads", &id, &who.actor)?;
        fr.say(&[&format!(
            "{id}: {past}. Its inbox tells it what to do; wake it if it is idle."
        )]);
        Ok(())
    })
}

/// The verb `thread pause [name]`.
///
/// # Errors
/// Refused for an unknown thread, a thread that is not the caller's to pause, and no board.
pub fn pause(frame: &mut Frame<'_>) -> Outcome {
    toggle(frame, "pause", "paused")
}

/// The verb `thread resume [name]`.
///
/// # Errors
/// Refused for an unknown thread, a thread that is not the caller's to resume, and no board.
pub fn resume(frame: &mut Frame<'_>) -> Outcome {
    toggle(frame, "resume", "resumed")
}

fn archive_op(id: &str, verb: &str, reason: Option<&str>) -> Value {
    let mut op = json!({"kind": verb, "target": {"c": "threads", "id": id}});
    if let Some(reason) = reason {
        op["reason"] = json!(reason);
    }
    op
}

fn archive_or_not(frame: &mut Frame<'_>, verb: &str) -> Outcome {
    let reason = optional_text("reason", &frame.value("reason"))?;
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        let id = target(board, fr, &who)?;
        if verb == "archive" && board["threads"][id.as_str()]["role"] == "lead" {
            return Err(CoordError::refuse(
                "the lead thread is never archived.",
            ));
        }
        let acting = who_of(&who);
        check_authority(&acting, verb, "threads", board["threads"].get(&id))?;
        let archived = board["threads"][id.as_str()]["status"] == "archived";
        if archived == (verb == "archive") {
            let state = if archived { "archived" } else { "not archived" };
            fr.say(&[&format!("{id} is already {state}.")]);
            return Ok(());
        }
        let before = lock_holders(board);
        let op = archive_op(&id, verb, reason.as_deref());
        let result = apply_op(board, &op, &acting, None);
        if result.status != Status::Applied {
            return Err(CoordError::refuse(result.outcome));
        }
        if verb == "unarchive" {
            let status = display(&board["threads"][id.as_str()]["status"]);
            fr.say(&[&format!("{id}: unarchived ({status}). It is listed again, and woken and checked like any thread.")]);
            return Ok(());
        }
        fr.say(&[&format!("{id}: archived. Off the thread list and counts, locks released; its items, questions and tests stay visible.")]);
        say_passed(fr, board, &before);
        Ok(())
    })
}

/// The verb `thread archive [name] [--reason "..."]`.
///
/// # Errors
/// Refused for the lead thread, an unknown thread, a thread that is not the caller's to
/// archive, a reason over its limit and no board.
pub fn archive(frame: &mut Frame<'_>) -> Outcome {
    archive_or_not(frame, "archive")
}

/// The verb `thread unarchive [name]`.
///
/// # Errors
/// Refused for an unknown thread, a thread that is not the caller's to unarchive, and no board.
pub fn unarchive(frame: &mut Frame<'_>) -> Outcome {
    archive_or_not(frame, "unarchive")
}
