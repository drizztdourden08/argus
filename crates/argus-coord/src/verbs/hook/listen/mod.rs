//! `coord hook listen`, the async and `asyncRewake` hook on `SessionStart`, Stop and `PostToolUse`:
//! the session's wake listener, with no model in between. It finds the session's thread like
//! the other hooks, joins the dashboard's wake stream for it (a lead hears every thread of its
//! project) and waits. The first wake prints its line on stderr and exits 2, and Claude Code
//! wakes the model with that text. A newer listener for the thread replaces this one (exit 0,
//! silent); a dashboard that is down is started and waited out; the wait ends silently after
//! `COORD_LISTEN_MS` (default 71 hours, just under the 72 hour hook timeout), or as soon as the process
//! named by `ARGUS_COORD_PARENT` (the shim) is gone.
//!
//! A `PostToolUse` listener is skipped while a listener of the thread started in the last two
//! minutes is alive (the `listen-<project>-<thread>.json` stamp); a `SessionStart` or Stop one
//! always takes over, since the server keeps one listener per thread. The `tool` hook keeps a
//! live listener fresh ([`keep_listener`]), so the one that follows it rarely has to replace it.
//!
//! The listener is a process of the hook host, so the host's `asyncRewake` is the only wake
//! channel: this code never starts one for another process and never needs Node. Under the
//! shim a Node parent waits on it; called as `argus.exe coord hook listen` there is none.

mod line;
mod parent;
mod stamp;
mod stream;

use std::path::Path;

use argus_board::json::truthy;
use argus_board::names::session_name;
use argus_core::clock::now_ms;
use argus_core::js;
use serde_json::Value;

use super::Said;
use super::ctx::Hook;
use super::thread::{doc, find};
use super::wake_line::{is_lead, label_of};
use crate::cli::CoordError;
use crate::who::is_ended;

/// The longest wait when `COORD_LISTEN_MS` does not say: 71 hours, one hour under the 259200 s
/// (72 hours) hook timeout of the rendered hook config, so the listener ends by itself first.
const LISTEN_DEFAULT_MS: i64 = 71 * 3600 * 1000;

/// What a listener listens to: the project, the thread (`*` for a lead) and its label.
pub struct Target {
    /// The project key.
    pub project: String,
    /// The thread id, or `*` for a lead.
    pub thread: String,
    /// The id of the thread of the session, also for a lead.
    pub id: String,
    /// The session title, cut to 80 characters.
    pub label: String,
}

/// `Number(COORD_LISTEN_MS) || default`.
fn listen_ms(hook: &Hook<'_>) -> i64 {
    let raw = hook
        .env
        .get("COORD_LISTEN_MS")
        .map(|v| js::trim(v).parse::<f64>());
    match raw {
        #[allow(clippy::cast_possible_truncation)]
        Some(Ok(ms)) if ms != 0.0 && !ms.is_nan() => ms as i64,
        _ => LISTEN_DEFAULT_MS,
    }
}

fn target_of(home: &Path, board: &Value, project: &str, id: &str) -> Target {
    let thread = doc(board, id);
    let lead = is_lead(thread);
    let name = session_name(home, &js::string(&board["project"]), board["name"].as_str());
    let label = label_of(thread, lead, &name);
    let label = js::slice(&label, 80);
    Target {
        project: project.to_owned(),
        thread: if lead { "*".to_owned() } else { id.to_owned() },
        id: id.to_owned(),
        label: if label.is_empty() { id } else { label }.to_owned(),
    }
}

/// The listener check of the `tool` hook, so the `PostToolUse` listener that follows has
/// nothing to do while a listener waits. A listener that is alive but older than two minutes
/// is kept fresh (one small file read; a write only then), instead of being replaced by a
/// new one every two minutes of tool calls. A dead or missing one is left to that listener,
/// which starts the next. Placeholders and ended threads have none.
pub fn keep_listener(hook: &Hook<'_>, project: &str, thread: &Value) {
    let Some(id) = thread["id"].as_str() else {
        return;
    };
    if truthy(thread, "placeholder") || is_ended(&thread["status"]) {
        return;
    }
    stamp::keep(&stamp::file(&hook.paths.home, project, id), listen_ms(hook));
}

/// Listens for this session's thread until a wake, a replacement or the deadline.
pub fn run(hook: &Hook<'_>) -> Result<Said, CoordError> {
    let Some((project, board)) = hook.project_board()? else {
        return Ok(Said::Nothing);
    };
    let Some(id) = hook.session_id().and_then(|_| find(&board, hook)) else {
        return Ok(Said::Nothing);
    };
    let thread = doc(&board, &id);
    if truthy(thread, "placeholder") || is_ended(&thread["status"]) {
        return Ok(Said::Nothing);
    }
    let file = stamp::file(&hook.paths.home, &project, &id);
    if hook.input["hook_event_name"] == "PostToolUse" && stamp::fresh(&file) {
        return Ok(Said::Nothing);
    }
    stamp::write(&file);
    let deadline = now_ms().saturating_add(listen_ms(hook));
    parent::watch(hook.env);
    Ok(stream::wait(
        hook,
        &target_of(&hook.paths.coord_home, &board, &project, &id),
        deadline,
    ))
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::LISTEN_DEFAULT_MS;

    #[test]
    fn the_default_deadline_is_71_hours_one_under_the_hook_timeout() {
        let hook_timeout_ms = 259_200 * 1000;
        assert_eq!(LISTEN_DEFAULT_MS, 71 * 3_600_000);
        assert_eq!(hook_timeout_ms - LISTEN_DEFAULT_MS, 3_600_000);
    }
}
