//! `coord thread <verb>` (`verbs/thread.mjs`): a thread registers, keeps its note and status,
//! flags and ends; the lead adds, edits, orders, pauses and archives any thread. A thread
//! changes only itself, the lead any thread (the checks live in `argus-board`).

mod away;
mod edit;
mod fold;
mod naming;
mod register;
mod set;
mod view;

use argus_board::Board;

use super::docs::{id_of, named_thread};
use super::frame::{Frame, with_frame};
use super::{dispatch, purge};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::who::Caller;

/// The statuses a thread may have.
pub(super) const STATUSES: [&str; 7] = [
    "dispatched",
    "active",
    "blocked",
    "waiting",
    "idle",
    "paused",
    "done",
];

type Verb = fn(&mut Frame<'_>) -> Outcome;

fn add(frame: &mut Frame<'_>) -> Outcome {
    dispatch::run_frame(frame)
}

fn rm(frame: &mut Frame<'_>) -> Outcome {
    let mut words = vec!["thread".to_owned()];
    words.extend(frame.args.iter().cloned());
    purge::run_verb(frame, "purge", &words)
}

const VERBS: [(&str, Verb); 14] = [
    ("register", register::run),
    ("note", edit::note),
    ("status", edit::status),
    ("flag", edit::flag),
    ("done", edit::done),
    ("list", view::list),
    ("show", view::show),
    ("set", set::run),
    ("add", add),
    ("rm", rm),
    ("pause", away::pause),
    ("resume", away::resume),
    ("archive", away::archive),
    ("unarchive", away::unarchive),
];

/// The thread a verb works on: the one named, else the caller.
pub(super) fn target(board: &Board, frame: &Frame<'_>, who: &Caller) -> Result<String, CoordError> {
    match frame.arg(0).filter(|name| !name.is_empty()) {
        Some(name) => Ok(id_of(named_thread(board, name)?)),
        None => Ok(who.thread.clone()),
    }
}

/// The verb `thread`.
///
/// # Errors
/// Refused as the verb it runs refuses.
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, |frame| {
        let names: Vec<&str> = VERBS.iter().map(|(name, _)| *name).collect();
        let found = frame
            .arg(0)
            .and_then(|sub| VERBS.iter().find(|(name, _)| name == &sub));
        let Some((_, verb)) = found else {
            return Err(CoordError::refuse(format!(
                "thread verb must be one of {}.",
                names.join(", ")
            )));
        };
        frame.args = &frame.args[1..];
        verb(frame)
    })
}
