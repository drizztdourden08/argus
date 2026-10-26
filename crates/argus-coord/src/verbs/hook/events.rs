//! The four session hooks: `session-start` (JSON context), `prompt` (plain text when adopted,
//! still a placeholder, or messages wait), `tool` (JSON context when adopted or messages wait)
//! and `stop` (a block with the messages as the reason). Stop marks the thread idle and never
//! adopts.

use argus_board::json::truthy;
use serde_json::{Value, json};

use super::Said;
use super::context::{context, render};
use super::ctx::Hook;
use super::visit::{Hit, visit};
use crate::cli::CoordError;

fn session_start(hook: &Hook<'_>, hit: &Hit, project: &str) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": context(hook, hit, project, true),
        }
    })
}

fn prompt(hook: &Hook<'_>, hit: &Hit, project: &str) -> Option<Value> {
    let speaks = hit.adopted || truthy(&hit.thread, "placeholder") || !hit.fresh.is_empty();
    speaks.then(|| Value::String(context(hook, hit, project, false)))
}

fn tool(hook: &Hook<'_>, hit: &Hit, project: &str) -> Option<Value> {
    if !hit.adopted && hit.fresh.is_empty() {
        return None;
    }
    let text = if hit.adopted {
        context(hook, hit, project, false)
    } else {
        render(&hit.thread, &hit.fresh)
    };
    Some(json!({
        "hookSpecificOutput": { "hookEventName": "PostToolUse", "additionalContext": text }
    }))
}

fn stop(hit: &Hit) -> Option<Value> {
    (!hit.fresh.is_empty())
        .then(|| json!({ "decision": "block", "reason": render(&hit.thread, &hit.fresh) }))
}

/// Runs one of the four session hooks.
pub fn run(hook: &Hook<'_>, event: &str) -> Result<Said, CoordError> {
    let project = hook.project()?;
    let activity = if event == "stop" { "idle" } else { "working" };
    let Some(hit) = visit(hook, &project, activity, event != "stop")? else {
        return Ok(Said::Nothing);
    };
    let out = match event {
        "session-start" => Some(session_start(hook, &hit, &project)),
        "prompt" => prompt(hook, &hit, &project),
        "tool" => tool(hook, &hit, &project),
        _ => stop(&hit),
    };
    Ok(match out {
        None => Said::Nothing,
        Some(Value::String(text)) => Said::Out(text),
        Some(value) => Said::Out(value.to_string()),
    })
}
