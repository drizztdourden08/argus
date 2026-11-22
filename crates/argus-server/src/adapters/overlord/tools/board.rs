//! `thread_message` and `thread_wake`: reach a thread of any board. The board is named by key, alias or
//! project name; an unknown board or thread is refused with a sentence that says what exists.

use argus_board::inbox::queue;
use argus_board::store::with_board;
use argus_core::boards;
use argus_wake::external::external_wake;
use argus_wake::text::clip;
use serde_json::{Value, json};

use crate::Services;

/// The sender of a note in a thread's inbox.
const FROM: &str = "Overlord";
/// How much of a note the wake line repeats.
const LINE_CLIP: usize = 100;
/// How many thread ids a refusal lists.
const LISTED: usize = 20;

/// A board and a thread that both exist.
struct Target {
    board: String,
    thread: String,
}

fn board_of(svc: &Services, named: &str) -> Result<String, String> {
    if let Some(id) = boards::resolve(&svc.paths.coord_home, named) {
        return Ok(id);
    }
    svc.boards()
        .projects()
        .into_iter()
        .find(|p| p.key == named || p.name.eq_ignore_ascii_case(named))
        .map(|p| p.key)
        .ok_or_else(|| format!("no board or project \"{named}\""))
}

fn threads_of(board: &Value) -> Vec<String> {
    board["threads"]
        .as_object()
        .map(|threads| threads.keys().take(LISTED).cloned().collect())
        .unwrap_or_default()
}

fn target(svc: &Services, project: &str, thread: &str) -> Result<Target, String> {
    let id = board_of(svc, project)?;
    let Some(board) = svc.boards().fresh(&id) else {
        return Err(format!("board \"{id}\" has no readable board file"));
    };
    if board["threads"][thread].is_null() {
        let known = threads_of(&board).join(", ");
        return Err(format!("no thread \"{thread}\" on {id}; its threads: {known}"));
    }
    Ok(Target {
        board: id,
        thread: thread.to_owned(),
    })
}

/// Hands one wake to the engine. With the wake service off the hook accepts and drops it.
fn wake(svc: &Services, to: &Target, reason: &str, text: &str) -> Result<(), String> {
    let body = json!({ "reason": reason, "thread": to.thread, "text": text });
    let input = external_wake(&to.board, &body, &|_, _| true).map_err(|e| e.message().to_owned())?;
    svc.wake_hook().submit(&[input])
}

/// Queues a note from `Overlord` in the inbox of `thread`, then wakes it so the relay reaches a sleeping one.
pub(super) fn message(
    svc: &Services,
    project: &str,
    thread: &str,
    text: &str,
) -> Result<String, String> {
    let to = target(svc, project, thread)?;
    let posted = with_board(&svc.paths.coord_home, &to.board, false, |board| {
        Ok(queue(board, &to.thread, FROM, "note", text, None))
    });
    match posted {
        Ok(true) => {}
        Ok(false) => return Err(format!("thread \"{thread}\" is gone from {}", to.board)),
        Err(e) => return Err(format!("the inbox could not be written: {e}")),
    }
    let line = format!("{FROM}: {}", clip(text, LINE_CLIP));
    let sent = match wake(svc, &to, "thread-message", &line) {
        Ok(()) => "and woke it".to_owned(),
        Err(why) => format!("but the wake failed: {why}"),
    };
    Ok(format!("queued for {} on {} {sent}", to.thread, to.board))
}

/// Wakes `thread` with `reason` as the line it hears, and no note.
pub(super) fn wake_only(
    svc: &Services,
    project: &str,
    thread: &str,
    reason: &str,
) -> Result<String, String> {
    let to = target(svc, project, thread)?;
    wake(svc, &to, "lead-message", &format!("{FROM}: {reason}"))
        .map_err(|why| format!("the wake failed: {why}"))?;
    Ok(format!("woke {} on {}", to.thread, to.board))
}
