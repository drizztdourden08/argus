//! A thread changing itself: `thread note`, `status`, `flag` and `done`. Every one counts as a
//! sign of life (the caller's heartbeat is refreshed first).

use argus_board::agents::stop_agents;
use argus_board::json::{map_mut, set};
use argus_board::limits::require_text;
use argus_board::pause::release_all;
use argus_board::store::stamp;
use serde_json::json;

use super::STATUSES;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::thread::docs::thread_mut;
use crate::verbs::thread::frame::Frame;
use crate::verbs::thread::passed::say_pass;

const FLAGS: [(&str, &str); 2] = [("needs-human", "needsHuman"), ("blocked", "blocked")];

/// The refusal for a status that is not one of the thread statuses.
pub fn status_error() -> CoordError {
    CoordError::refuse(format!("status must be one of {}.", STATUSES.join(", ")))
}

/// The verb `thread note "<text>"`.
///
/// # Errors
/// Refused for a missing or too long note, no thread for this session, and no board.
pub fn note(frame: &mut Frame<'_>) -> Outcome {
    let text = require_text("note", &json!(frame.arg(0)))?;
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        set(thread_mut(board, &who.thread)?, "note", &text);
        stamp(board, "threads", &who.thread, &who.actor)?;
        fr.say(&[&format!("{}: {text}", who.thread)]);
        Ok(())
    })
}

/// The verb `thread status <status>`.
///
/// # Errors
/// Refused for a status that does not exist, no thread for this session, and no board.
pub fn status(frame: &mut Frame<'_>) -> Outcome {
    let next = frame
        .arg(0)
        .filter(|s| STATUSES.contains(s))
        .ok_or_else(status_error)?;
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        set(thread_mut(board, &who.thread)?, "status", next);
        stamp(board, "threads", &who.thread, &who.actor)?;
        fr.say(&[&format!("{}: {next}", who.thread)]);
        Ok(())
    })
}

/// The verb `thread flag <needs-human|blocked> [--clear]`.
///
/// # Errors
/// Refused for a flag that does not exist, no thread for this session, and no board.
pub fn flag(frame: &mut Frame<'_>) -> Outcome {
    let word = frame.arg(0).unwrap_or("undefined");
    let Some((_, key)) = FLAGS.iter().find(|(name, _)| *name == word) else {
        let names: Vec<&str> = FLAGS.iter().map(|(name, _)| *name).collect();
        return Err(CoordError::refuse(format!(
            "flag must be one of {}.",
            names.join(", ")
        )));
    };
    let on = !frame.options.flag("clear");
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        let thread = thread_mut(board, &who.thread)?;
        map_mut(thread, "flags").insert((*key).to_owned(), on.into());
        stamp(board, "threads", &who.thread, &who.actor)?;
        let state = if on { "set" } else { "cleared" };
        fr.say(&[&format!("{}: {word} {state}", who.thread)]);
        Ok(())
    })
}

/// The verb `thread done`.
///
/// # Errors
/// Refused for no thread for this session, and no board.
pub fn done(frame: &mut Frame<'_>) -> Outcome {
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        set(thread_mut(board, &who.thread)?, "status", "done");
        stop_agents(board, &who.thread, &who.actor)?;
        let freed = release_all(board, &who.thread);
        stamp(board, "threads", &who.thread, &who.actor)?;
        fr.say(&[&format!("{}: done", who.thread)]);
        for (resource, holder) in freed {
            say_pass(fr, board, &resource, &holder);
        }
        Ok(())
    })
}
