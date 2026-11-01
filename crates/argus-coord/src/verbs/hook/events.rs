//! The four session hooks: `session-start` (JSON context), `prompt` (plain text when adopted,
//! still a placeholder, messages wait, or board notices came since the last turn: one digest
//! line, see [`super::notices`]), `tool` (JSON context when adopted or messages wait)
//! and `stop` (a block with the messages as the reason). Stop marks the thread idle and never
//! adopts. A call from a sub-agent (its payload has an `agent_id`) hands no messages over, since
//! they are its lead's; a `tool` call of the thread itself also keeps its wake listener fresh.

use argus_board::json::truthy;
use serde_json::{Value, json};

use super::Said;
use super::context::{context, render};
use super::ctx::Hook;
use super::listen::keep_listener;
use super::notices::notice_line;
use super::visit::{Hit, Mode, visit};
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
    let base = speaks.then(|| context(hook, hit, project, false));
    let text = match (base, notice_line(hook, project, hit)) {
        (Some(base), Some(notices)) => format!("{base}\n{notices}"),
        (Some(text), None) | (None, Some(text)) => text,
        (None, None) => return None,
    };
    Some(Value::String(text))
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

/// The activity an event marks the thread with: `stop` ends the turn, a sub-agent's tool call
/// marks nothing (its lead may be idle while it works), every other event is the lead at work.
fn activity_of(hook: &Hook<'_>, event: &str) -> Option<&'static str> {
    match event {
        "stop" => Some("idle"),
        "tool" if hook.is_sub_agent() => None,
        _ => Some("working"),
    }
}

/// Runs one of the four session hooks.
pub fn run(hook: &Hook<'_>, event: &str) -> Result<Said, CoordError> {
    let project = hook.project()?;
    let mode = Mode {
        activity: activity_of(hook, event),
        may_adopt: event != "stop",
        deliver: !hook.is_sub_agent(),
    };
    let Some(hit) = visit(hook, &project, mode)? else {
        return Ok(Said::Nothing);
    };
    if event == "tool" && mode.deliver {
        keep_listener(hook, &project, &hit.thread);
    }
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
