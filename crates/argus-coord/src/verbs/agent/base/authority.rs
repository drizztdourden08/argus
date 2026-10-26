//! The ownership rules of `authorize` (`ops.mjs`) for the two collections these verbs write: an
//! agent record, and the thread that manages a bundle. `argus-board` keeps its own copy private
//! to its ops (the soft verbs of an agent go through [`argus_board::ops::apply_op`], which uses
//! it), so this is the narrow part the CLI needs; it goes when that function is made public.

use argus_board::json::nullish;
use argus_board::ops::Role;
use serde_json::Value;

use crate::cli::exit::CoordError;
use crate::who::Caller;

fn hint(owner: &str) -> String {
    format!("Ask with coord inbox send {owner} \"...\" or coord message.")
}

fn belongs(id: &str, owner: &str, word: &str) -> String {
    let label = if id == owner {
        format!("the {owner} record")
    } else {
        id.to_owned()
    };
    let allowed = if owner == "coordinator" {
        "the coordinator".to_owned()
    } else {
        format!("{owner} or the coordinator")
    };
    format!(
        "{label} belongs to {owner}; only {allowed} can {word} it. {}",
        hint(owner)
    )
}

fn id_of(doc: &Value, coll: &str) -> String {
    nullish(doc, "id")
        .or_else(|| nullish(doc, "resource"))
        .unwrap_or_else(|| coll.to_owned())
}

/// A new agent record for the thread `to` (`--for`, else the caller itself): a thread makes only
/// its own.
///
/// # Errors
/// A thread creating a record for another thread.
pub fn create_agent(who: &Caller, to: &str) -> Result<(), CoordError> {
    if who.role != Role::Thread || to == who.thread {
        return Ok(());
    }
    Err(CoordError::refuse(format!(
        "you cannot create records for {to}; only the coordinator can. {}",
        hint(to)
    )))
}

/// `verb` (`set`, `done`, `fail` or `stop`) on an agent record: a thread changes only the agents
/// of its own thread.
///
/// # Errors
/// A thread changing the agent of another thread.
pub fn change_agent(who: &Caller, agent: &Value) -> Result<(), CoordError> {
    if who.role != Role::Thread {
        return Ok(());
    }
    let owner = nullish(agent, "thread").or_else(|| nullish(agent, "id"));
    if owner.as_deref() == Some(who.thread.as_str()) {
        return Ok(());
    }
    Err(CoordError::refuse(belongs(
        &id_of(agent, "agents"),
        owner.as_deref().unwrap_or("nobody"),
        "change",
    )))
}

/// `set` on the thread record `doc` (the thread that manages a bundle): a thread may only do it
/// for itself.
///
/// # Errors
/// A thread acting for another thread, or a thread that does not exist.
pub fn manage_thread(who: &Caller, doc: Option<&Value>) -> Result<(), CoordError> {
    if who.role != Role::Thread {
        return Ok(());
    }
    let Some(doc) = doc else {
        return Err(CoordError::refuse("no threads record to set."));
    };
    let id = id_of(doc, "threads");
    if nullish(doc, "id").as_deref() == Some(who.thread.as_str()) {
        return Ok(());
    }
    Err(CoordError::refuse(format!(
        "{} cannot set {id}: only {id} itself or the coordinator can. {}",
        who.thread,
        hint(&id)
    )))
}
