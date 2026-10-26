//! `board show`: the summary of the board as text, or as JSON with `--json`.

use argus_board::json::display;
use serde_json::Value;

use super::summary::summary;
use crate::cli::exit::Outcome;
use crate::cli::out::{clip, table};
use crate::verbs::thread::docs::{cell, list};
use crate::verbs::thread::frame::Frame;

const HEADER: [&str; 6] = ["thread", "status", "agents", "subject", "seen", "note"];

fn joined(ids: &Value) -> String {
    let text = list(ids).iter().map(display).collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        "-".to_owned()
    } else {
        text
    }
}

fn lock_text(lock: &Value) -> String {
    let queue = list(&lock["queue"]);
    let waiting = if queue.is_empty() {
        String::new()
    } else {
        let names: Vec<String> = queue.iter().map(display).collect();
        format!(" [{}]", names.join(","))
    };
    let holder = match &lock["holder"] {
        Value::Null => "free".to_owned(),
        other => display(other),
    };
    format!("{}={holder}{waiting}", display(&lock["resource"]))
}

fn threads_table(frame: &Frame<'_>, s: &Value) -> String {
    let mut rows = vec![HEADER.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>()];
    for t in list(&s["threads"]) {
        rows.push(vec![
            cell(t.get("id")),
            cell(t.get("status")),
            cell(t.get("agents")),
            cell(t.get("subject")),
            frame.ago(&t["lastSeen"]),
            clip(&cell(t.get("note")), 50),
        ]);
    }
    table(&rows)
}

fn counts_line(s: &Value) -> String {
    let handed = match &s["handedBuild"] {
        Value::Null => "-".to_owned(),
        other => display(other),
    };
    format!(
        "items: {}   open questions: {}   pending decisions: {}   tests: {}   handed build: {handed}   manual actions: {}",
        list(&s["items"]).len(),
        joined(&s["openQuestions"]),
        joined(&s["pendingDecisions"]),
        list(&s["tests"]).len(),
        joined(&s["pendingActions"]),
    )
}

/// The verb `board show [--json]`.
///
/// # Errors
/// Refused when there is no board.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let s = summary(&board);
    if frame.options.flag("json") {
        frame.json(&s);
        return Ok(());
    }
    let project = display(&s["project"]);
    let url = frame.dashboard_url(&format!("/p/{project}/"));
    frame.say(&[&format!("# {project}  ({url})"), ""]);
    let rows = threads_table(frame, &s);
    frame.say(&[&rows]);
    frame.say(&["", &counts_line(&s)]);
    let locks: Vec<String> = list(&s["locks"]).iter().map(lock_text).collect();
    if !locks.is_empty() {
        frame.say(&[&format!("locks: {}", locks.join("  "))]);
    }
    let inbox: Vec<String> = s["inbox"]
        .as_object()
        .map(|m| {
            m.iter()
                .map(|(k, n)| format!("{k}({})", display(n)))
                .collect()
        })
        .unwrap_or_default();
    if !inbox.is_empty() {
        frame.say(&[&format!("inbox: {}", inbox.join(" "))]);
    }
    Ok(())
}
