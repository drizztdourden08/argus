//! One sub-agent, one record. A hook event belongs to the record of its own Claude agent id
//! (the payload's `agent_id`, also the id in the transcript name `agent-<id>.jsonl`) and to no
//! other: never to the newest open record of the thread, nor to one that reads alike. These are
//! the rules the start, stop and result hooks share: the id of a payload, the record of an id,
//! closing a record, and the stop that came before its start.

use argus_agents::hook_logic::close_record;
use argus_board::Board;
use argus_board::agents::is_live;
use argus_board::json::{section, set, truthy};
use argus_core::clock::now_ms;
use argus_core::js;
use serde_json::Value;

use super::ctx::Hook;
use super::dispatches::{DISPATCH_TTL_MS, recent};
use super::input::clip;

fn id_chars(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// The agent id in a transcript path: the `<id>` of a final `agent-<id>.jsonl`.
fn id_of_file(path: &str) -> Option<&str> {
    let name = path.rsplit(['/', '\\']).next()?;
    let id = name.strip_prefix("agent-")?.strip_suffix(".jsonl")?;
    id_chars(id).then_some(id)
}

/// The agent id of a hook payload: `agent_id`, else the id in the name of
/// `agent_transcript_path`; `None` when it has none, or when the two disagree (then it is not
/// known whose event it is).
pub fn payload_agent_id(hook: &Hook<'_>) -> Option<String> {
    let id = clip(&hook.input["agent_id"], 120);
    let path = clip(&hook.input["agent_transcript_path"], 1000);
    let from_file = id_of_file(&path);
    match from_file {
        Some(file) if !id.is_empty() && id != file => None,
        Some(file) if id.is_empty() => Some(file.to_owned()),
        _ => Some(id).filter(|i| !i.is_empty()),
    }
}

/// The order records of one id are preferred in: a live one, then one that is not a
/// duplicate's husk, then one that is not archived or deleted.
fn rank(agent: &Value) -> u8 {
    let gone = truthy(agent, "archivedAt") || truthy(agent, "deletedAt");
    u8::from(!is_live(agent)) * 4 + u8::from(truthy(agent, "duplicateOf")) * 2 + u8::from(gone)
}

/// The record of a Claude agent id (its `hookId` or `claudeId`), in any state.
pub fn record_of(board: &Board, id: &str) -> Option<String> {
    let mut all: Vec<(&String, &Value)> = section(board, "agents")
        .iter()
        .filter(|(_, a)| a["hookId"].as_str() == Some(id) || a["claudeId"].as_str() == Some(id))
        .collect();
    all.sort_by_key(|(_, a)| rank(a));
    all.first().map(|(key, _)| (*key).clone())
}

/// Closes a record: status, the answer, the model it ran on. A record the dashboard closed for
/// silence also loses its `closedBy`.
pub fn close(board: &mut Board, id: &str, last: &str, used: Option<&str>) {
    let record = board
        .get_mut("agents")
        .and_then(|a| a.get_mut(id))
        .and_then(Value::as_object_mut);
    let Some(record) = record else {
        return;
    };
    let reaped = record.get("reaped").is_some_and(js::truthy);
    close_record(record, last, used.filter(|u| !u.is_empty()), now_ms());
    if reaped && !record.contains_key("reaped") {
        record.shift_remove("closedBy");
    }
}

/// A stop that was handled before this start left a note: the record is closed with it, so
/// nothing stays working for good. The note list is written back aged.
pub fn after_stop(board: &mut Board, agent_id: &str, id: &str) {
    let mut stops = recent(&board["agentStops"], DISPATCH_TTL_MS);
    let gone = stops
        .iter()
        .position(|s| s["hookId"].as_str() == Some(agent_id))
        .map(|i| stops.remove(i));
    set(board, "agentStops", Value::Array(stops));
    if let Some(note) = gone {
        let last = note["last"].as_str().unwrap_or_default();
        close(board, id, last, note["used"].as_str());
    }
}
