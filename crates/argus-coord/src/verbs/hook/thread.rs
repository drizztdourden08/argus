//! The thread a hook is for: by session id first, by checkout only for a thread no other
//! session has claimed. A placeholder is a session that has not registered yet.

use argus_board::Board;
use argus_board::json::{section, truthy};
use serde_json::Value;

use super::ctx::Hook;
use crate::cli::Options;
use crate::who::{is_ended, resolve_thread};

/// The `host` of a thread run by Argus itself; a desktop thread has no `host`.
pub const HOST_ARGUS: &str = "argus";

/// The id of the thread this hook's session is, if the board has one.
pub fn find(board: &Board, hook: &Hook<'_>) -> Option<String> {
    resolve_thread(board, &hook.cwd, &Options::default(), hook.session_id()).ok()
}

/// The thread doc of an id.
pub fn doc<'b>(board: &'b Board, id: &str) -> &'b Value {
    section(board, "threads").get(id).unwrap_or(&Value::Null)
}

/// Whether a thread is still the unnamed placeholder of a session.
pub fn is_placeholder(thread: &Value) -> bool {
    truthy(thread, "placeholder")
}

/// Whether a thread doc is the hosted Overlord's (`host` is `argus`).
pub fn is_hosted(thread: &Value) -> bool {
    thread["host"].as_str() == Some(HOST_ARGUS)
}

/// The registered thread of this session (never a placeholder), for the sub-agent hooks.
pub fn registered(board: &Board, hook: &Hook<'_>) -> Option<String> {
    hook.session_id()?;
    find(board, hook).filter(|id| !is_placeholder(doc(board, id)))
}

/// A session whose every thread on this board is done or archived: its work here ended, and
/// the hooks leave it alone.
pub fn retired(board: &Board, sid: &str) -> bool {
    let held: Vec<&Value> = section(board, "threads")
        .values()
        .filter(|t| t["sessionId"].as_str() == Some(sid))
        .collect();
    !held.is_empty() && held.iter().all(|t| is_ended(&t["status"]))
}
