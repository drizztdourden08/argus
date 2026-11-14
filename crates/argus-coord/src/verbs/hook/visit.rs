//! The board side of `session-start`, `prompt`, `tool` and `stop`: one locked read, change and
//! save. Heartbeat, activity, the session id follow, the messages handed over now (marked
//! delivered), and the adoption of a session the board has not seen (never on Stop).

use argus_board::json::section;
use argus_board::names::session_name;
use argus_board::store::touch;
use argus_board::{Board, BoardError};
use argus_core::clock::{now_iso, now_ms, parse_ms};
use argus_core::js;
use serde_json::{Value, json};

use super::adopt::adopt;
use super::ctx::Hook;
use super::thread::{HOST_ARGUS, doc, find, is_hosted, retired};
use crate::cli::CoordError;

/// A thread whose `lastSeen` is older than this gets a new one.
const HEARTBEAT_MS: i64 = 60_000;

/// What a visit found.
pub struct Hit {
    /// The thread doc after the visit.
    pub thread: Value,
    /// The messages handed over now.
    pub fresh: Vec<Value>,
    /// Whether the session was just put on the board as a placeholder.
    pub adopted: bool,
    /// What the session titles of this project start with.
    pub name: String,
    /// The thread's `lastSeen` before this visit: the end of its last turn, which the prompt
    /// hook's notice digest counts from.
    pub seen_before: Option<String>,
}

/// What a visit may do beyond the heartbeat.
#[derive(Clone, Copy)]
pub struct Mode {
    /// The activity to mark the thread with; `None` leaves it (a sub-agent's call).
    pub activity: Option<&'static str>,
    /// Whether a session the board has not seen is put on it as a placeholder.
    pub may_adopt: bool,
    /// Whether the thread's waiting messages are handed over now (and marked delivered). A
    /// sub-agent never takes its lead's inbox: the lead would never see those messages.
    pub deliver: bool,
}

/// Visits the board for this hook's session. `None` when there is no board, no session, the
/// session's work here ended, or (without `may_adopt`) the session has no thread.
pub fn visit(hook: &Hook<'_>, project: &str, mode: Mode) -> Result<Option<Hit>, CoordError> {
    let Some(sid) = hook.session_id() else {
        return Ok(None);
    };
    let Some(peek) = hook.board(project)? else {
        return Ok(None);
    };
    if retired(&peek, sid) || (find(&peek, hook).is_none() && !mode.may_adopt) {
        return Ok(None);
    }
    hook.mutate(project, |board| on_board(board, hook, sid, mode))
}

fn on_board(
    board: &mut Board,
    hook: &Hook<'_>,
    sid: &str,
    mode: Mode,
) -> Result<Option<Hit>, BoardError> {
    if retired(board, sid) {
        return Ok(None);
    }
    let (id, adopted) = match find(board, hook) {
        Some(id) => (id, false),
        None if mode.may_adopt => (
            adopt(board, hook, sid, mode.activity.unwrap_or("idle"))?,
            true,
        ),
        None => return Ok(None),
    };
    let seen_before = doc(board, &id)["lastSeen"].as_str().map(str::to_owned);
    heartbeat(board, &id, sid, mode.activity);
    mark_host(board, &id, hook.hosted());
    let fresh = if mode.deliver {
        deliver(board, &id)
    } else {
        Vec::new()
    };
    let name = session_name(
        &hook.paths.coord_home,
        &js::string(&board["project"]),
        board["name"].as_str(),
    );
    Ok(Some(Hit {
        thread: doc(board, &id).clone(),
        fresh,
        adopted,
        name,
        seen_before,
    }))
}

/// `!lastSeen` or older than a minute; a `lastSeen` that is not a date is not stale.
fn stale(thread: &Value) -> bool {
    let seen = &thread["lastSeen"];
    if !js::truthy(seen) {
        return true;
    }
    parse_ms(&js::string(seen)).is_some_and(|ms| now_ms() - ms > HEARTBEAT_MS)
}

fn heartbeat(board: &mut Board, id: &str, sid: &str, activity: Option<&str>) {
    let Some(thread) = board["threads"].get_mut(id) else {
        return;
    };
    let was_stale = stale(thread);
    if thread["sessionId"].as_str() != Some(sid) {
        thread["sessionId"] = json!(sid);
    }
    let changed = activity.is_some_and(|a| thread["activity"].as_str() != Some(a) || a == "idle");
    if was_stale || changed {
        thread["lastSeen"] = json!(now_iso());
        if let Some(activity) = activity {
            thread["activity"] = json!(activity);
        }
        touch(board, "threads", id);
    }
}

/// A hosted session sets `host: argus` on its thread; a desktop session on a thread that was
/// hosted takes the field off again, so `host` is absent for a desktop thread.
fn mark_host(board: &mut Board, id: &str, hosted: bool) {
    let Some(thread) = board["threads"].get_mut(id) else {
        return;
    };
    if hosted && !is_hosted(thread) {
        thread["host"] = json!(HOST_ARGUS);
    } else if !hosted && is_hosted(thread) {
        if let Some(fields) = thread.as_object_mut() {
            fields.remove("host");
        }
    } else {
        return;
    }
    touch(board, "threads", id);
}

/// Marks the thread's undelivered messages delivered now and returns them.
fn deliver(board: &mut Board, id: &str) -> Vec<Value> {
    let has_inbox = section(board, "inbox").contains_key(id);
    let list = if has_inbox {
        board["inbox"][id].as_array_mut()
    } else {
        None
    };
    let Some(list) = list else {
        return Vec::new();
    };
    list.iter_mut()
        .filter(|m| m.is_object() && !js::truthy(&m["deliveredAt"]))
        .map(|m| {
            m["deliveredAt"] = json!(now_iso());
            m.clone()
        })
        .collect()
}
