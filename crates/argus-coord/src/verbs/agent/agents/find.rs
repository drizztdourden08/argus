//! Which agent a name or an id means (`find` of `agent.mjs`), and the state an ended agent leaves
//! behind when it is resumed.

use std::cmp::Ordering;

use argus_board::Board;
use argus_board::agents::is_live;
use argus_board::json::{display, doc, doc_mut, section, set, shown, truthy, unset};
use argus_board::text::slug;
use serde_json::Value;

use crate::cli::Options;
use crate::cli::exit::CoordError;
use crate::verbs::agent::base::Run;
use crate::verbs::agent::base::authority::change_agent;
use crate::who::Caller;

/// The marks the reaper and the reconciler leave on a record they closed.
const ENDING_MARKS: [&str; 7] = [
    "reaped",
    "lost",
    "orphanSince",
    "idleSince",
    "orphanWokeAt",
    "idleWokeAt",
    "duplicateOf",
];

/// Which agents a name may mean, per verb.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Want {
    /// The running ones.
    Live,
    /// Any not hidden.
    Open,
    /// Only the archived ones (what `unarchive` brings back).
    Archived,
    /// Only the trashed ones (what `restore` brings back).
    Deleted,
    /// Every one.
    Any,
}

impl Want {
    fn admits(self, agent: &Value) -> bool {
        let (archived, deleted) = (truthy(agent, "archivedAt"), truthy(agent, "deletedAt"));
        match self {
            Want::Live => is_live(agent),
            Want::Open => !archived && !deleted,
            Want::Archived => archived && !deleted,
            Want::Deleted => deleted,
            Want::Any => true,
        }
    }
}

/// `/^A-\d+$/i`.
#[must_use]
pub fn is_id(text: &str) -> bool {
    let upper = text.to_ascii_uppercase();
    upper
        .strip_prefix("A-")
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

fn number_of(id: &str) -> i64 {
    id.get(2..).and_then(|n| n.parse().ok()).unwrap_or(0)
}

/// The newest first: the later start, then the higher number.
fn newest(a: &Value, b: &Value) -> Ordering {
    shown(b.get("startedAt"))
        .cmp(&shown(a.get("startedAt")))
        .then_with(|| number_of(&display(&b["id"])).cmp(&number_of(&display(&a["id"]))))
}

fn by_id(board: &Board, reference: &str, want: Want) -> Result<String, CoordError> {
    let id = reference.to_ascii_uppercase();
    let Some(agent) = doc(board, "agents", &id) else {
        return Err(CoordError::refuse(format!("no agent \"{reference}\".")));
    };
    if want == Want::Live && !is_live(agent) {
        let why = if truthy(agent, "deletedAt") {
            "in the trash".to_owned()
        } else if truthy(agent, "archivedAt") {
            "archived".to_owned()
        } else {
            display(&agent["status"])
        };
        return Err(CoordError::refuse(format!(
            "{} is {why}, not running.",
            display(&agent["id"])
        )));
    }
    Ok(id)
}

/// The thread a call works for: `--for T` (which must exist), else the caller's own.
///
/// # Errors
/// A thread that does not exist, or no caller.
pub fn owner_of(
    board: &Board,
    who: Option<&Caller>,
    options: &Options,
) -> Result<String, CoordError> {
    if let Some(name) = options.text("for") {
        if doc(board, "threads", name).is_none() {
            return Err(CoordError::refuse(format!("no thread \"{name}\".")));
        }
        return Ok(name.to_owned());
    }
    who.map(|w| w.thread.clone())
        .ok_or_else(|| CoordError::refuse("no thread is calling: this needs a registered session."))
}

/// An agent by A-id, or by name: the newest of the caller's thread (or `--for T`) that is
/// `want`. `who` may be `None` when the reference is an id or `--for` names the thread.
///
/// # Errors
/// No reference, no such agent, an agent that is not running when it must be.
pub fn find(
    board: &Board,
    who: Option<&Caller>,
    reference: Option<&str>,
    options: &Options,
    want: Want,
) -> Result<String, CoordError> {
    let Some(reference) = reference.filter(|r| !r.is_empty()) else {
        return Err(CoordError::refuse("name the agent: its A-id or its name."));
    };
    if is_id(reference) {
        return by_id(board, reference, want);
    }
    let owner = owner_of(board, who, options)?;
    let name = slug(reference);
    let mut found: Vec<&Value> = section(board, "agents")
        .values()
        .filter(|a| a["thread"].as_str() == Some(&owner) && a["name"].as_str() == Some(&name))
        .filter(|a| want.admits(a))
        .collect();
    found.sort_by(|a, b| newest(a, b));
    let running = if want == Want::Live { "running " } else { "" };
    found.first().map(|a| display(&a["id"])).ok_or_else(|| {
        CoordError::refuse(format!("no {running}agent named \"{name}\" on {owner}."))
    })
}

/// What an agent that ended leaves behind goes when it runs again: no end time, no result and
/// none of the reaper's or reconciler's marks.
pub fn clear_ending(agent: &mut Value) {
    set(agent, "finishedAt", Value::Null);
    set(agent, "result", Value::Null);
    for mark in ENDING_MARKS {
        unset(agent, mark);
    }
}

/// An agent record to change.
///
/// # Errors
/// The record is gone (it was found a moment ago, so only a broken board).
pub fn agent_mut<'a>(board: &'a mut Board, id: &str) -> Result<&'a mut Value, CoordError> {
    doc_mut(board, "agents", id).ok_or_else(|| CoordError::refuse(format!("no agent \"{id}\".")))
}

/// The agent the first word of the verb names (`find(board, who, positional[0], options, want)`).
///
/// # Errors
/// As [`find`].
pub fn named(board: &Board, who: &Caller, run: &Run<'_>, want: Want) -> Result<String, CoordError> {
    let reference = run.positional.first().map(String::as_str);
    find(board, Some(who), reference, run.options, want)
}

/// The agent the verb names and may change, ready to change.
///
/// # Errors
/// As [`find`], or an agent of another thread.
pub fn own<'a>(
    board: &'a mut Board,
    who: &Caller,
    run: &Run<'_>,
    want: Want,
) -> Result<(String, &'a mut Value), CoordError> {
    let id = named(board, who, run, want)?;
    let agent = agent_mut(board, &id)?;
    change_agent(who, agent)?;
    Ok((id, agent))
}
