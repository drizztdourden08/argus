//! The hooks put an unregistered session on the board as a placeholder (`s-...`). When that
//! session registers under its real name, the placeholder is folded in: its inbox moves to the
//! real thread and it is retired, on this board and on any other.

use std::fs;

use argus_board::Board;
use argus_board::json::{map_mut, set, truthy};
use argus_board::store::{read_board, stamp};
use serde_json::Value;

use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::thread::docs::{all, id_of, list, thread_mut};
use crate::verbs::thread::frame::{Frame, locked};

fn is_open_placeholder(thread: &Value, sid: &str) -> bool {
    truthy(thread, "placeholder")
        && thread["sessionId"].as_str() == Some(sid)
        && !truthy(thread, "mergedInto")
}

fn retire(board: &mut Board, id: &str, into: &str) -> Result<(), CoordError> {
    let thread = thread_mut(board, id)?;
    set(thread, "status", "done");
    set(thread, "mergedInto", into);
    set(thread, "note", format!("registered as {into}"));
    stamp(board, "threads", id, into)?;
    Ok(())
}

/// Folds the placeholders of this session on this board into the thread `name`.
///
/// # Errors
/// A thread cannot be stamped.
pub fn placeholders(board: &mut Board, name: &str, sid: &str) -> Result<(), CoordError> {
    let ids: Vec<String> = all(board, "threads")
        .into_iter()
        .filter(|t| is_open_placeholder(t, sid) && id_of(t) != name)
        .map(id_of)
        .collect();
    for id in ids {
        let combined: Vec<Value> = list(&board["inbox"][id.as_str()])
            .iter()
            .chain(list(&board["inbox"][name]))
            .cloned()
            .collect();
        let inbox = map_mut(board, "inbox");
        inbox.insert(name.to_owned(), Value::Array(combined));
        inbox.shift_remove(&id);
        retire(board, &id, name)?;
    }
    Ok(())
}

/// Folds the placeholders of this session on every other board, once this board is saved.
///
/// # Errors
/// A board cannot be read or written.
pub fn elsewhere(frame: &Frame<'_>, name: &str, sid: &str) -> Outcome {
    let home = frame.home();
    let Ok(dirs) = fs::read_dir(home) else {
        return Ok(());
    };
    let into = format!("{name} on {}", frame.project);
    for entry in dirs.flatten() {
        let key = entry.file_name().to_string_lossy().into_owned();
        if key == frame.project || key.starts_with('_') || !entry.path().join("board.json").exists()
        {
            continue;
        }
        let Some(peek) = read_board(home, &key)? else {
            continue;
        };
        let held = all(&peek, "threads")
            .into_iter()
            .any(|t| is_open_placeholder(t, sid));
        if held {
            locked(home, &key, false, |board| retire_all(board, sid, &into))?;
        }
    }
    Ok(())
}

fn retire_all(board: &mut Board, sid: &str, into: &str) -> Outcome {
    let ids: Vec<String> = all(board, "threads")
        .into_iter()
        .filter(|t| is_open_placeholder(t, sid))
        .map(id_of)
        .collect();
    for id in ids {
        retire(board, &id, into)?;
    }
    Ok(())
}
