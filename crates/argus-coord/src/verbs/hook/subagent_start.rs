//! `hook subagent-start`, `SubagentStart`: one agents doc per sub-agent of a registered thread,
//! working now. Only the record of this agent id is touched: when it exists it is kept (and
//! reopened when the dashboard closed it for silence, or the agent was resumed after a stop);
//! when no record has the id a new one is made from the pending dispatch. The lead's own
//! records, which carry no id, are never taken here.

use argus_board::agents::is_live;
use argus_board::json::{display, set, truthy, unset};
use argus_board::store::{create_doc, stamp};
use argus_board::text::slug;
use argus_board::{Board, BoardError};
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::binding::{after_stop, payload_agent_id, record_of};
use super::ctx::Hook;
use super::dispatches::take_dispatch;
use super::input::clip;
use super::kind::kind_of;
use super::thread::registered;
use super::transcript::file_of;
use crate::cli::CoordError;

const REOPENED: &str = "resumed (its SubagentStart arrived again), reopened automatically";

/// The start as the hook saw it.
struct Start {
    agent_id: String,
    thread: String,
    kind: String,
    file: Value,
    dispatch: Option<Value>,
}

/// Brings a closed record back to working. Returns whether it changed.
fn reopen(record: &mut Value) -> bool {
    if is_live(record) {
        return false;
    }
    set(record, "status", "working");
    set(record, "finishedAt", Value::Null);
    set(record, "result", Value::Null);
    set(record, "note", REOPENED);
    for gone in ["reaped", "closedBy", "lost"] {
        unset(record, gone);
    }
    true
}

/// The start of an agent the board already has a record of. An archived, deleted or
/// duplicate record is left alone.
fn known(
    board: &mut Board,
    id: &str,
    agent_id: &str,
    file: &Value,
    thread: &str,
) -> Result<(), BoardError> {
    let record = &mut board["agents"][id];
    let husk = ["archivedAt", "deletedAt", "duplicateOf"];
    if husk.iter().any(|k| truthy(record, k)) {
        return Ok(());
    }
    let mut touched = reopen(record);
    if !truthy(record, "claudeId") {
        set(record, "claudeId", agent_id);
        touched = true;
    }
    if !truthy(record, "transcriptFile") && !file.is_null() {
        set(record, "transcriptFile", file.clone());
        touched = true;
    }
    after_stop(board, agent_id, id);
    if touched {
        stamp(board, "agents", id, thread)?;
    }
    Ok(())
}

/// The last four UTF-16 units of a text (`slice(-4)`).
fn tail4(text: &str) -> &str {
    let mut units = 0;
    let at = text
        .char_indices()
        .rev()
        .take_while(|(_, c)| {
            units += c.len_utf16();
            units <= 4
        })
        .last()
        .map_or(text.len(), |(i, _)| i);
    &text[at..]
}

fn name_of(start: &Start) -> String {
    let described = start
        .dispatch
        .as_ref()
        .map_or(Value::Null, |d| d["description"].clone());
    let from_description = slug(&if described.is_null() {
        String::new()
    } else {
        display(&described)
    });
    if !from_description.is_empty() {
        return from_description;
    }
    let kind = Some(slug(&start.kind)).filter(|s| !s.is_empty());
    let tail = Some(slug(tail4(&start.agent_id))).filter(|s| !s.is_empty());
    format!(
        "{}-{}",
        kind.as_deref().unwrap_or("agent"),
        tail.as_deref().unwrap_or("x")
    )
}

fn fields(start: &Start) -> Map<String, Value> {
    let d = start.dispatch.clone().unwrap_or(Value::Null);
    let first = [&d["description"], &d["promptHead"]]
        .into_iter()
        .find(|v| js::truthy(v))
        .cloned()
        .unwrap_or_else(|| Value::String(start.kind.clone()));
    let task = clip(&first, 120);
    let kind_text = if js::truthy(&d["description"]) {
        js::string(&d["description"])
    } else {
        task.clone()
    };
    let fields = json!({
        "name": name_of(start), "thread": start.thread, "task": task,
        "model": d["model"].clone(), "modelWhy": null, "kind": kind_of(&kind_text),
        "item": null, "branch": null, "worktree": null, "status": "working", "note": null,
        "result": null, "startedAt": now_iso(), "finishedAt": null,
        "hookId": start.agent_id, "claudeId": start.agent_id,
        "toolUseId": d["toolUseId"].clone(), "source": "hook", "agentType": start.kind,
        "modelUsed": null, "transcriptFile": start.file,
    });
    let Value::Object(fields) = fields else {
        unreachable!("a json object")
    };
    fields
}

fn on_board(board: &mut Board, hook: &Hook<'_>, agent_id: &str) -> Result<(), BoardError> {
    let Some(thread) = registered(board, hook) else {
        return Ok(());
    };
    let file = file_of(hook, agent_id);
    if let Some(id) = record_of(board, agent_id) {
        return known(board, &id, agent_id, &file, &thread);
    }
    let kind = Some(clip(&hook.input["agent_type"], 60)).filter(|k| !k.is_empty());
    let kind = kind.unwrap_or_else(|| "agent".to_owned());
    let dispatch = take_dispatch(board, &thread, &kind);
    let start = Start {
        agent_id: agent_id.to_owned(),
        file,
        thread,
        kind,
        dispatch,
    };
    let made = create_doc(board, "agents", "A", fields(&start), &start.thread)?;
    after_stop(board, agent_id, &made);
    Ok(())
}

/// Records the sub-agent of a registered thread.
pub fn run(hook: &Hook<'_>) -> Result<(), CoordError> {
    let Some(agent_id) = payload_agent_id(hook) else {
        return Ok(());
    };
    hook.mutate_if(
        |peek| registered(peek, hook).is_some(),
        |board| on_board(board, hook, &agent_id),
    )
}
