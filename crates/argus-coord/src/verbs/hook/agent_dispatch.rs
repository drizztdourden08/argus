//! `hook agent-dispatch`, `PreToolUse` on `Agent` or `Task`: remembers what a registered thread
//! is about to dispatch, for the `SubagentStart` hook to pick up. The same tool call announced
//! twice (a retried hook) is one dispatch.

use argus_board::Board;
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::ctx::Hook;
use super::dispatches::{DISPATCH_TTL_MS, recent, store};
use super::input::clip_or_null;
use super::thread::registered;
use crate::cli::CoordError;

/// Whether the hook is about the `Agent` or `Task` tool.
pub fn is_agent_tool(hook: &Hook<'_>) -> bool {
    matches!(hook.input["tool_name"].as_str(), Some("Agent" | "Task"))
}

fn entry(hook: &Hook<'_>, thread: &str, tool_use_id: &Value) -> Value {
    let ti = &hook.input["tool_input"];
    json!({
        "thread": thread,
        "toolUseId": tool_use_id,
        "description": clip_or_null(&ti["description"], 200),
        "promptHead": clip_or_null(&ti["prompt"], 300),
        "type": clip_or_null(&ti["subagent_type"], 60),
        "model": clip_or_null(&ti["model"], 60),
        "background": ti["run_in_background"] == true,
        "at": now_iso(),
    })
}

fn on_board(board: &mut Board, hook: &Hook<'_>) {
    let Some(thread) = registered(board, hook) else {
        return;
    };
    let tool_use_id = clip_or_null(&hook.input["tool_use_id"], 120);
    let mut list = recent(&board["agentDispatches"], DISPATCH_TTL_MS);
    if !tool_use_id.is_null() && list.iter().any(|d| d["toolUseId"] == tool_use_id) {
        return;
    }
    list.push(entry(hook, &thread, &tool_use_id));
    store(board, "agentDispatches", list);
}

/// Stores a pending dispatch for a registered thread.
pub fn run(hook: &Hook<'_>) -> Result<(), CoordError> {
    if !is_agent_tool(hook) {
        return Ok(());
    }
    let admit = |peek: &Board| registered(peek, hook).is_some();
    hook.mutate_if(admit, |board| {
        on_board(board, hook);
        Ok(())
    })
}
