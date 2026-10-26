//! `hook agent-result`, `PostToolUse` on `Agent` or `Task`: the exact id of the sub-agent that
//! call launched. The record of that id gets `claudeId` (a record found only by the call's
//! `tool_use_id` counts when it has no agent id of its own); with no record yet the id waits in
//! `agentLaunches`, where `coord agent start` finds it. A record of the same agent whose two ids
//! disagree is flagged `idMismatch` and the tool result wins; a record of another id is never
//! touched.

use argus_agents::hook_logic::launched_id;
use argus_board::json::{section, set, truthy};
use argus_board::store::stamp;
use argus_board::{Board, BoardError};
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Value, json};

use super::agent_dispatch::is_agent_tool;
use super::binding::record_of;
use super::ctx::Hook;
use super::dispatches::{DISPATCH_TTL_MS, LAUNCH_TTL_MS, recent, store};
use super::input::{clip, clip_or_null};
use super::thread::registered;
use crate::cli::CoordError;

/// The record of this call: the record of the id first; a record found only by the tool use id
/// counts when it has no agent id of its own (one with another id is another agent's).
fn record_of_call(board: &Board, tool_use_id: &Value, id: &str) -> Option<String> {
    record_of(board, id).or_else(|| {
        section(board, "agents")
            .iter()
            .find(|(_, a)| {
                !tool_use_id.is_null()
                    && a["toolUseId"] == *tool_use_id
                    && !truthy(a, "claudeId")
                    && !truthy(a, "hookId")
            })
            .map(|(key, _)| key.clone())
    })
}

fn set_claude_id(board: &mut Board, key: &str, id: &str, thread: &str) -> Result<(), BoardError> {
    let record = &mut board["agents"][key];
    if record["claudeId"].as_str() == Some(id) {
        return Ok(());
    }
    let prior = [&record["claudeId"], &record["hookId"]]
        .into_iter()
        .find(|v| js::truthy(v))
        .cloned();
    if prior.is_some_and(|p| p.as_str() != Some(id)) {
        set(record, "idMismatch", true);
    }
    set(record, "claudeId", id);
    stamp(board, "agents", key, thread)
}

fn description_of(board: &Board, hook: &Hook<'_>, tool_use_id: &Value) -> Value {
    let told = clip(&hook.input["tool_response"]["description"], 200);
    if !told.is_empty() {
        return Value::String(told);
    }
    recent(&board["agentDispatches"], DISPATCH_TTL_MS)
        .into_iter()
        .find(|d| !tool_use_id.is_null() && d["toolUseId"] == *tool_use_id)
        .map(|d| d["description"].clone())
        .filter(js::truthy)
        .unwrap_or(Value::Null)
}

fn on_board(board: &mut Board, hook: &Hook<'_>, id: &str) -> Result<(), BoardError> {
    let Some(thread) = registered(board, hook) else {
        return Ok(());
    };
    let tool_use_id = clip_or_null(&hook.input["tool_use_id"], 120);
    if let Some(key) = record_of_call(board, &tool_use_id, id) {
        return set_claude_id(board, &key, id, &thread);
    }
    let description = description_of(board, hook, &tool_use_id);
    let mut kept = recent(&board["agentLaunches"], LAUNCH_TTL_MS);
    kept.retain(|l| l["claudeId"].as_str() != Some(id));
    kept.push(json!({
        "thread": thread, "toolUseId": tool_use_id, "claudeId": id,
        "description": description, "at": now_iso(),
    }));
    store(board, "agentLaunches", kept);
    Ok(())
}

/// Binds the agent id of a launched sub-agent to its record, or leaves a launch note.
pub fn run(hook: &Hook<'_>) -> Result<(), CoordError> {
    if !is_agent_tool(hook) {
        return Ok(());
    }
    let Some(id) = launched_id(&hook.input["tool_response"]) else {
        return Ok(());
    };
    hook.mutate_if(
        |peek| registered(peek, hook).is_some(),
        |board| on_board(board, hook, &id),
    )
}
