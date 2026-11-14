//! Who holds the ball: the assignee of an item, and of a to-do.

use argus_core::js;
use serde_json::{Map, Value};

use super::known::Known;
use crate::Board;
use crate::content::check as content_check;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc};

/// The prefix of an assignee that names an agent record.
const AGENT_PREFIX: &str = "agent:";

/// The assignee of an item: a thread of the board, the owner, or `agent:A-n` of an agent record
/// of the board. A blank value is `null`.
pub(crate) fn check(board: &Board, v: &Value) -> Result<Value, BoardError> {
    if let Some(id) = v
        .as_str()
        .map(js::trim)
        .and_then(|t| t.strip_prefix(AGENT_PREFIX))
    {
        return agent(board, id);
    }
    content_check("assignee", v, &Known::of(board, Vec::new()).context())
}

/// `agent:<id>` when `id` is an agent record of the board, else a refusal naming the id.
fn agent(board: &Board, id: &str) -> Result<Value, BoardError> {
    if doc(board, "agents", id).is_some() {
        return Ok(Value::String(format!("{AGENT_PREFIX}{id}")));
    }
    Err(refuse(format!(
        "assignee {AGENT_PREFIX}{id}: {id} is not an agent of this board."
    )))
}

/// The assignee of a to-do: a thread of the board, the owner, or `lead:<key>`. A blank value is
/// `null`.
pub(crate) fn todo(board: &Board, v: &Value) -> Result<Value, BoardError> {
    content_check("todoAssignee", v, &Known::of(board, Vec::new()).context())
}

/// The thread an assignee names, when it names one: the owner, the leads and the agents are no
/// thread, so no wake goes to them.
pub(crate) fn thread_of(board: &Board, assignee: &Value) -> Option<String> {
    assignee
        .as_str()
        .filter(|a| *a != "owner" && !a.starts_with(AGENT_PREFIX))
        .filter(|a| doc(board, "threads", a).is_some())
        .map(str::to_owned)
}

/// The thread a new item belongs to when its assignee is a thread: the assignee and the thread
/// move together, so both may be named only when they are the same. An `agent:A-n` assignee names
/// no thread.
pub(crate) fn hint(board: &Board, set: &Map<String, Value>) -> Result<Option<Value>, BoardError> {
    let Some(v) = set.get("assignee").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let assignee = check(board, v)?;
    let Some(thread) = thread_of(board, &assignee) else {
        return Ok(None);
    };
    match set.get("thread").filter(|t| js::truthy(t)) {
        Some(named) if display(named) != thread => Err(refuse(format!(
            "thread {} and assignee {thread} differ; name one thread.",
            display(named)
        ))),
        _ => Ok(Some(Value::String(thread))),
    }
}

/// The assignee of a new to-do, written when one is given.
pub(crate) fn stamp_todo(
    board: &Board,
    set: &Map<String, Value>,
    fields: &mut Map<String, Value>,
) -> Result<(), BoardError> {
    if let Some(v) = set.get("assignee") {
        let who = todo(board, v)?;
        if !who.is_null() {
            fields.insert("assignee".to_owned(), who);
        }
    }
    Ok(())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn board() -> Board {
        json!({
            "threads": {"ui": {"id": "ui"}},
            "agents": {"A-1": {"id": "A-1", "status": "running"}}
        })
    }

    #[test]
    fn a_known_agent_is_a_valid_assignee() {
        let got = check(&board(), &json!("agent:A-1")).unwrap();
        assert_eq!(got, json!("agent:A-1"));
    }

    #[test]
    fn an_unknown_agent_is_refused_naming_its_id() {
        let text = check(&board(), &json!("agent:A-9"))
            .unwrap_err()
            .to_string();
        assert!(text.contains("A-9 is not an agent of this board"), "{text}");
    }

    #[test]
    fn an_agent_assignee_names_no_thread() {
        assert_eq!(thread_of(&board(), &json!("agent:A-1")), None);
        assert_eq!(thread_of(&board(), &json!("ui")), Some("ui".to_owned()));
    }

    #[test]
    fn the_hint_gives_no_thread_for_an_agent() {
        let set = json!({"assignee": "agent:A-1"});
        let set = set.as_object().unwrap();
        assert_eq!(hint(&board(), set).unwrap(), None);
    }
}
