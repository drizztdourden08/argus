//! The session registry: no file of its own, the `sessionId` of every live thread on every
//! board. A board is searched for the session id as bytes first, and only a board that holds it
//! is parsed, into a typed peek at `threads.*.sessionId` and `status`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value;

/// A thread that is done or archived no longer owns its session or checkout.
pub const ENDED: [&str; 2] = ["done", "archived"];

#[derive(Deserialize)]
struct ThreadPeek {
    #[serde(rename = "sessionId", default)]
    session_id: Value,
    #[serde(default)]
    status: Value,
}

#[derive(Deserialize)]
struct BoardPeek {
    #[serde(default)]
    threads: Option<HashMap<String, ThreadPeek>>,
}

/// Whether a thread status counts as ended.
#[must_use]
pub fn is_ended(status: &Value) -> bool {
    status.as_str().is_some_and(|s| ENDED.contains(&s))
}

/// The bytes a session id has inside a JSON text (escaped as a JSON string, without quotes).
fn needle(sid: &str) -> Vec<u8> {
    let quoted = serde_json::to_string(sid).unwrap_or_default();
    quoted.trim_matches('"').as_bytes().to_vec()
}

fn holds_live_thread(raw: &[u8], sid: &str) -> bool {
    let Ok(peek) = serde_json::from_slice::<BoardPeek>(raw) else {
        return false;
    };
    peek.threads.is_some_and(|threads| {
        threads
            .values()
            .any(|t| t.session_id.as_str() == Some(sid) && !is_ended(&t.status))
    })
}

/// The project whose board has a live thread holding this session id. Every board under
/// `coord_home` is looked at, in directory order; the first that holds it wins.
#[must_use]
pub fn project_of_session(coord_home: &Path, sid: &str) -> Option<String> {
    let dirs = fs::read_dir(coord_home).ok()?;
    let needle = needle(sid);
    for entry in dirs.flatten() {
        let Ok(raw) = fs::read(entry.path().join("board.json")) else {
            continue;
        };
        if memchr::memmem::find(&raw, &needle).is_some() && holds_live_thread(&raw, sid) {
            return entry.file_name().to_str().map(str::to_owned);
        }
    }
    None
}
