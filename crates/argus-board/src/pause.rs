//! Pausing and archiving a thread: it stays on the board but out of sight; its locks go to the
//! next in the queue and its running agents become `stopped`.

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use crate::Board;
use crate::agents::stop_agents;
use crate::error::BoardError;
use crate::inbox::{Message, message_id, push};
use crate::json::{doc_mut, ids, nullish, set, shown, text, unset};
use crate::store::touch;

const PAUSE_TAG: &str = "[Paused] ";

fn paused_title(title: &str) -> String {
    if !title.is_empty() && !title.starts_with(PAUSE_TAG) {
        format!("{PAUSE_TAG}{title}")
    } else {
        title.to_owned()
    }
}

fn active_title(title: &str) -> &str {
    title.strip_prefix(PAUSE_TAG).unwrap_or(title)
}

fn lock_signature(lock: &Value) -> String {
    let queue: Vec<String> = lock["queue"]
        .as_array()
        .map(|q| q.iter().map(crate::json::display).collect())
        .unwrap_or_default();
    format!("{}|{}", shown(lock.get("holder")), queue.join(","))
}

fn release_lock(lock: &mut Value, thread: &str) -> Option<String> {
    let queue: Vec<Value> = lock["queue"]
        .as_array()
        .map(|q| {
            q.iter()
                .filter(|id| id.as_str() != Some(thread))
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    set(lock, "queue", Value::Array(queue));
    if lock["holder"].as_str() != Some(thread) {
        return None;
    }
    let next = lock["queue"]
        .as_array_mut()
        .filter(|q| !q.is_empty())
        .map(|q| q.remove(0));
    let granted = next.as_ref().and_then(Value::as_str).map(str::to_owned);
    set(lock, "holder", next.unwrap_or(Value::Null));
    set(
        lock,
        "since",
        granted.as_ref().map_or(Value::Null, |_| json!(now_iso())),
    );
    granted
}

/// Free every lock the thread holds or waits for; the next in each queue gets it and a line in
/// its inbox. Returns the `(resource, new holder)` pairs.
pub fn release_all(board: &mut Board, thread: &str) -> Vec<(String, String)> {
    let mut freed = Vec::new();
    for key in ids(board, "locks") {
        let Some(lock) = doc_mut(board, "locks", &key) else {
            continue;
        };
        let before = lock_signature(lock);
        let resource = text(lock, "resource").unwrap_or(&key).to_owned();
        if let Some(holder) = release_lock(lock, thread) {
            freed.push((resource.clone(), holder));
        }
        if before != lock_signature(lock) {
            touch(board, "locks", &resource);
        }
    }
    for (resource, holder) in &freed {
        let line = format!("{resource} is yours now (released by {thread}).");
        let message = Message {
            id: message_id(resource),
            from: thread,
            kind: "lock-granted",
            text: &line,
            reference: Some(resource),
        };
        push(board, holder, &message);
    }
    freed
}

fn tell(board: &mut Board, thread: &str, from: &str, line: &str) {
    let message = Message {
        id: message_id(thread),
        from,
        kind: "pause",
        text: line,
        reference: Some(thread),
    };
    push(board, thread, &message);
}

fn keep_previous_status(t: &mut Value) {
    match t.get("status").cloned() {
        Some(status) => set(t, "prevStatus", status),
        None => unset(t, "prevStatus"),
    }
}

/// Pause a thread. Returns false when it is already paused. The caller records who did it
/// (`stamp`); `by` is the wording the thread is told: "the owner".
///
/// # Errors
/// A thread, an agent or a lock doc cannot be stamped.
pub fn pause_thread(board: &mut Board, id: &str, by: &str) -> Result<bool, BoardError> {
    let Some(t) = doc_mut(board, "threads", id) else {
        return Ok(false);
    };
    if t["status"] == "paused" {
        return Ok(false);
    }
    keep_previous_status(t);
    set(t, "status", "paused");
    set(t, "pausedAt", now_iso());
    set(t, "pausedBy", by);
    let session = nullish(t, "session");
    stop_agents(board, id, by)?;
    release_all(board, id);
    touch(board, "threads", id);
    let fallback = format!(
        "{} - <Subject>",
        board["name"]
            .as_str()
            .map_or_else(|| crate::json::display(&board["project"]), str::to_owned)
    );
    let title = paused_title(session.as_deref().unwrap_or(&fallback));
    let line = format!(
        "You are paused by {by}. Stop at a safe point: commit or note where you are (coord thread note), then rename this session to \"{title}\" (set_session_title, \"self\") and register again with that --session. Your locks were released. Do no further work until you are resumed."
    );
    tell(board, id, by, &line);
    Ok(true)
}

/// Resume a paused thread. Returns false when it is not paused.
pub fn resume_thread(board: &mut Board, id: &str, by: &str) -> bool {
    let Some(t) = doc_mut(board, "threads", id) else {
        return false;
    };
    if t["status"] != "paused" {
        return false;
    }
    let back = match t["prevStatus"].as_str() {
        Some(prev) if !prev.is_empty() && prev != "paused" => prev.to_owned(),
        _ => "active".to_owned(),
    };
    set(t, "status", back);
    set(t, "resumedAt", now_iso());
    unset(t, "pausedBy");
    let title = nullish(t, "session").map_or_else(
        || "<Project> - <Subject>".to_owned(),
        |s| active_title(&s).to_owned(),
    );
    touch(board, "threads", id);
    let line = format!(
        "You are resumed by {by}. Rename this session back to \"{title}\" (set_session_title, \"self\"), register again with that --session, re-claim any lock you need, and carry on."
    );
    tell(board, id, by, &line);
    true
}

/// Archive a thread, recording `actor`. Returns false when it is already archived.
///
/// # Errors
/// A thread, an agent or a lock doc cannot be stamped.
pub fn archive_thread(board: &mut Board, id: &str, actor: &str) -> Result<bool, BoardError> {
    let Some(t) = doc_mut(board, "threads", id) else {
        return Ok(false);
    };
    if t["status"] == "archived" {
        return Ok(false);
    }
    keep_previous_status(t);
    set(t, "status", "archived");
    set(t, "archivedAt", now_iso());
    set(t, "archivedBy", actor);
    stop_agents(board, id, actor)?;
    release_all(board, id);
    touch(board, "threads", id);
    Ok(true)
}

/// Bring an archived thread back to the status it had. Returns false when it is not archived.
pub fn unarchive_thread(board: &mut Board, id: &str) -> bool {
    let Some(t) = doc_mut(board, "threads", id) else {
        return false;
    };
    if t["status"] != "archived" {
        return false;
    }
    let back = match t["prevStatus"].as_str() {
        Some(prev) if !prev.is_empty() && prev != "archived" => prev.to_owned(),
        _ => "idle".to_owned(),
    };
    set(t, "status", back);
    unset(t, "archivedAt");
    unset(t, "archivedBy");
    touch(board, "threads", id);
    true
}
