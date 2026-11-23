//! The assignee an agent holds (I-206): a start writes `agent:A-n` on each item it works on, as the
//! calling thread, and the end of the agent (done, fail, stop, delete, replaced) hands the item back
//! to the agent's thread, only while the assignee still names that agent.

use argus_board::Board;
use argus_board::agent_links::items_of;
use argus_board::agents::is_live;
use argus_board::holds::{handbacks, tag};
use argus_board::json::{display, doc};
use serde_json::{Value, json};

use crate::cli::exit::CoordError;
use crate::verbs::agent::base::apply::apply;
use crate::who::Caller;

fn assignee_of(board: &Board, item: &str) -> String {
    doc(board, "items", item)
        .map(|i| display(&i["assignee"]))
        .unwrap_or_default()
}

fn write(board: &mut Board, who: &Caller, item: &str, assignee: &str) -> Result<(), CoordError> {
    let operation = json!({
        "kind": "edit",
        "target": {"c": "items", "id": item},
        "set": {"assignee": assignee},
        "force": true,
    });
    apply(board, who, &operation).map(|_| ())
}

/// The live agent, other than `own`, that holds the item.
fn held_by(board: &Board, item: &str, own: Option<&str>) -> Option<String> {
    let holder = assignee_of(board, item);
    let id = holder.strip_prefix("agent:")?;
    let live = doc(board, "agents", id).is_some_and(is_live);
    (live && Some(id) != own).then(|| id.to_owned())
}

/// Refuses a start on an item a live agent holds, unless `take`.
///
/// # Errors
/// The first item held by another live agent.
pub fn check_free(
    board: &Board,
    items: &[String],
    own: Option<&str>,
    take: bool,
) -> Result<(), CoordError> {
    if take {
        return Ok(());
    }
    for item in items {
        if let Some(holder) = held_by(board, item, own) {
            return Err(CoordError::refuse(format!(
                "{item} is held by {holder}: stop {holder} or pass --take."
            )));
        }
    }
    Ok(())
}

/// Writes `agent:<id>` as the assignee of each item the agent works on.
///
/// # Errors
/// The refusal of the edit.
pub fn claim(board: &mut Board, who: &Caller, agent_id: &str) -> Result<(), CoordError> {
    let agent = doc(board, "agents", agent_id)
        .cloned()
        .unwrap_or(Value::Null);
    for item in items_of(&agent) {
        write(board, who, &item, &tag(agent_id))?;
    }
    Ok(())
}

/// Hands each item the agent still holds back to the agent's thread.
///
/// # Errors
/// The refusal of the edit.
pub fn release(board: &mut Board, who: &Caller, agent_id: &str) -> Result<(), CoordError> {
    for (item, thread) in handbacks(board, agent_id) {
        write(board, who, &item, &thread)?;
    }
    Ok(())
}
