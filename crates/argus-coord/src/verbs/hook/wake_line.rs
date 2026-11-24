//! The wake line of a session start. A folder whose rendered settings carry the `hook listen`
//! background hook is woken with no Monitor; any other session is told to run its wake-stream
//! under the Monitor tool (a lead listens to every thread of its project, a thread to its own id).

use std::fs;

use argus_board::json::shown;
use argus_core::js;
use serde_json::Value;

use super::ctx::Hook;
use super::visit::Hit;
use crate::who::git_root;

const AUTOMATIC: &str = "Waking: automatic. A background hook is your wake listener (no Monitor, no tokens while idle): a line `WAKE ...` that reaches you, even as Stop hook feedback, is a wake: run coord inbox. Do NOT start a Monitor wake-stream and do not use a coord tick cron; if you have a tick cron, delete it (CronList, then CronDelete).";

/// What a hosted thread is told in place of the listener and the board tab.
pub const HOSTED: &str = "Waking: hosted. Wakes come as user messages; no listener, no board tab.";

/// Whether the rendered settings of the hook's folder or its checkout carry `hook listen`.
fn auto_wake(hook: &Hook<'_>) -> bool {
    [hook.cwd.clone(), git_root(&hook.cwd)].iter().any(|dir| {
        fs::read_to_string(dir.join(".claude").join("settings.json"))
            .is_ok_and(|text| text.contains("hook listen"))
    })
}

/// The label of a wake-stream: the session title, else the project name for a lead, else the
/// thread id, with quotes, backslashes and line breaks made spaces.
pub fn label_of(thread: &Value, lead: bool, name: &str) -> String {
    let raw = match thread.get("session") {
        None | Some(Value::Null) if lead => name.to_owned(),
        None | Some(Value::Null) => shown(thread.get("id")),
        Some(session) => shown(Some(session)),
    };
    let mut out = String::with_capacity(raw.len());
    let mut in_run = false;
    for c in raw.chars() {
        if matches!(c, '"' | '\\' | '\r' | '\n') {
            if !in_run {
                out.push(' ');
            }
            in_run = true;
        } else {
            out.push(c);
            in_run = false;
        }
    }
    js::trim(&out).to_owned()
}

/// Whether a thread doc is the project's lead.
pub fn is_lead(thread: &Value) -> bool {
    thread["role"] == "lead"
}

/// The wake line of a registered thread.
pub fn wake_line(hook: &Hook<'_>, hit: &Hit, project: &str) -> String {
    if hook.hosted() {
        return HOSTED.to_owned();
    }
    if auto_wake(hook) {
        return AUTOMATIC.to_owned();
    }
    let lead = is_lead(&hit.thread);
    let label = label_of(&hit.thread, lead, &hit.name);
    let thread = if lead {
        "'*'".to_owned()
    } else {
        shown(hit.thread.get("id"))
    };
    let cmd =
        format!("dashboard wake-stream --project {project} --thread {thread} --id \"{label}\"");
    format!(
        "Waking: start your wake-stream now with the Monitor tool (persistent true if it offers it, else its longest timeout): `{cmd}`. Each line it prints is a wake: run coord inbox. If the Monitor ends at its timeout you are notified: re-arm the same command at once (missed wakes are replayed). Do not use a coord tick cron; if you have one, delete it (CronList, then CronDelete)."
    )
}
