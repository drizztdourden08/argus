//! A session the board has never seen joins it as a placeholder thread (`s-` and the start of
//! its session id), asked to name itself and register. Registering folds the placeholder into
//! the real thread.

use argus_board::json::obj_mut;
use argus_board::store::create_doc;
use argus_board::{Board, BoardError};
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::ctx::Hook;
use crate::who::git_root;

/// The placeholder id of a session: `s-` and its first six letters and digits, lower case.
fn placeholder_id(sid: &str) -> String {
    let head: String = sid
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(6)
        .collect();
    format!("s-{}", head.to_ascii_lowercase())
}

/// The link of the host session, when its id is a plain word.
fn host_link(hook: &Hook<'_>) -> Value {
    let host = hook.env.get("CLAUDE_CODE_HOST_SESSION_ID");
    let plain = |h: &&str| {
        h.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    };
    host.filter(plain).map_or(Value::Null, |h| {
        Value::String(format!("claude://claude.ai/epitaxy/{h}"))
    })
}

/// Puts this session on the board as a placeholder thread and returns its id. A retired
/// placeholder from an earlier session with the same id prefix gives way.
pub fn adopt(
    board: &mut Board,
    hook: &Hook<'_>,
    sid: &str,
    activity: &str,
) -> Result<String, BoardError> {
    let id = placeholder_id(sid);
    obj_mut(&mut board["threads"]).shift_remove(&id);
    let fields = json!({
        "id": id,
        "name": id,
        "role": "thread",
        "placeholder": true,
        "status": "active",
        "activity": activity,
        "registeredAt": now_iso(),
        "lastSeen": now_iso(),
        "sessionId": sid,
        "link": host_link(hook),
        "root": git_root(&hook.cwd).to_string_lossy(),
        "session": null,
        "subject": null,
        "note": "new session, not named yet",
        "flags": { "needsHuman": false, "blocked": false },
    });
    let Value::Object(fields) = fields else {
        unreachable!("a json object")
    };
    create_doc(board, "threads", "", fields, &id)
}
