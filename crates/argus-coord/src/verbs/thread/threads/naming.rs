//! Session titles follow one convention across every project: the lead is just the app
//! name (the main project's is "Overlord"), a thread is "<Project> - <Subject>" with a subject of
//! four words at most. A board may list its own prefixes when its threads belong to several
//! products. `thread register` tells a thread whose title does not follow it.

use std::path::Path;

use argus_board::Board;
use argus_board::json::{display, truthy};
use argus_board::names::session_name;
use serde_json::Value;

use crate::verbs::thread::docs::{list, words};

const PAUSE_TAG: &str = "[Paused] ";
const MAX_WORDS: usize = 4;

fn prefixes_of(board: &Board, name: &str) -> Vec<String> {
    let listed: Vec<String> = list(&board["prefixes"]).iter().map(display).collect();
    if listed.is_empty() {
        vec![name.to_owned()]
    } else {
        listed
    }
}

fn paused_problem(thread: &Value, title: &str, tagged: bool) -> Option<String> {
    let paused = thread["status"] == "paused";
    if paused && !tagged {
        return Some(format!(
            "this thread is paused: rename the session to \"{PAUSE_TAG}{title}\" (set_session_title), then register again with that --session"
        ));
    }
    if !paused && tagged {
        return Some(format!(
            "this thread is not paused: drop the \"{}\" tag from the session title, then register again",
            PAUSE_TAG.trim()
        ));
    }
    None
}

fn subject_problem(title: &str, prefixes: &[String]) -> Option<String> {
    let tags: Vec<String> = prefixes.iter().map(|p| format!("{p} - ")).collect();
    let prefix = tags
        .iter()
        .find(|p| title.starts_with(p.as_str()))
        .cloned()
        .unwrap_or_else(|| format!("{} - ", prefixes[0]));
    if !title.starts_with(&prefix) {
        return Some(format!(
            "rename this session to \"{}<Subject>\" (set_session_title), then register again with that --session",
            tags.join("<Subject>\" or \"")
        ));
    }
    let count = words(&title[prefix.len()..]).count();
    if (1..=MAX_WORDS).contains(&count) {
        None
    } else {
        Some(format!(
            "keep the subject to four words at most: \"{prefix}<Subject>\""
        ))
    }
}

/// What is wrong with the title of the session a thread registers from, if anything.
#[must_use]
pub fn problem(home: &Path, board: &Board, thread: &Value) -> Option<String> {
    let name = session_name(home, &display(&board["project"]), board["name"].as_str());
    let lead = thread["role"] == "lead";
    if !truthy(thread, "session") {
        let wanted = if lead {
            name.clone()
        } else {
            format!("{name} - <Subject>")
        };
        return Some(format!(
            "no session title recorded; pass --session \"{wanted}\""
        ));
    }
    let title = display(&thread["session"]);
    let tagged = title.starts_with(PAUSE_TAG);
    if let Some(found) = paused_problem(thread, &title, tagged) {
        return Some(found);
    }
    if tagged {
        return None;
    }
    if lead {
        return (title != name).then(|| {
            format!(
                "rename this session to \"{name}\" (set_session_title), then register again with that --session"
            )
        });
    }
    subject_problem(&title, &prefixes_of(board, &name))
}
