//! Reading threads: `thread list` and `thread show`. Neither locks nor saves the board.

use std::cmp::Ordering;

use argus_board::Board;
use argus_board::agents::live_agent_ids;
use argus_board::json::{to_number, truthy};
use serde_json::Value;

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out::{clip, table};
use crate::verbs::thread::docs::{all, cell, id_of, named_thread};
use crate::verbs::thread::frame::Frame;

const HEADER: [&str; 6] = ["thread", "status", "flags", "subject", "seen", "note"];

fn flags_of(board: &Board, thread: &Value) -> String {
    let running = live_agent_ids(board, Some(&id_of(thread))).len();
    let marks = [
        truthy(&thread["flags"], "needsHuman").then(|| "H".to_owned()),
        truthy(&thread["flags"], "blocked").then(|| "B".to_owned()),
        (running > 0).then(|| format!("a{running}")),
    ];
    marks.into_iter().flatten().collect::<Vec<_>>().join(" ")
}

fn by_rank(a: &Value, b: &Value) -> Ordering {
    to_number(Some(&a["rank"]))
        .partial_cmp(&to_number(Some(&b["rank"])))
        .unwrap_or(Ordering::Equal)
}

/// The verb `thread list [--all]`: archived threads are off the list unless `--all`, in the
/// owner's manual order.
///
/// # Errors
/// Refused when there is no board.
pub fn list(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let every = frame.options.flag("all");
    let mut threads: Vec<&Value> = all(&board, "threads")
        .into_iter()
        .filter(|t| t["status"] != "archived" || every)
        .collect();
    threads.sort_by(|a, b| by_rank(a, b));
    if threads.is_empty() {
        frame.say(&["no threads registered."]);
        return Ok(());
    }
    let mut rows = vec![HEADER.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>()];
    for t in threads {
        rows.push(vec![
            cell(t.get("id")),
            cell(t.get("status")),
            flags_of(&board, t),
            cell(t.get("subject")),
            frame.ago(&t["lastSeen"]),
            clip(&cell(t.get("note")), 60),
        ]);
    }
    frame.say(&[&table(&rows)]);
    Ok(())
}

/// The verb `thread show [name]`: the thread doc as JSON.
///
/// # Errors
/// Refused for an unknown thread, no thread for this session, and no board.
pub fn show(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let thread = match frame.arg(0).filter(|name| !name.is_empty()) {
        Some(name) => named_thread(&board, name)?.clone(),
        None => board["threads"][frame.caller(&board)?.thread.as_str()].clone(),
    };
    let text =
        serde_json::to_string_pretty(&thread).map_err(|e| CoordError::refuse(e.to_string()))?;
    frame.say(&[&text]);
    Ok(())
}
