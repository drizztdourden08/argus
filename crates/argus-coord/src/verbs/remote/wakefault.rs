//! `coord wake-fault list | clear`: a wake the dashboard could not get to a thread. Anyone may
//! list the open faults; only the Overlord clears one, after it found out what is going on.

use serde_json::{Value, json};

use super::support::link::{connect, list_of};
use super::support::note::bounded;
use super::support::overlord;
use super::support::text::{cell, is_key, truthy};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::take_sub;

const NOTE_MIN: usize = 10;
const NOTE_MAX: usize = 300;
const ADVICE: &str = "A fault is cleared after the Overlord found out what is going on.";
const NAMING: &str = "name the fault: coord wake-fault clear <project>/<thread> --note \"<what was wrong and what you did>\".";
const FIND_OUT: &str = "Find out what is wrong (process list, GET /api/p/<key>/wakes/listeners, restart the thread's wake-stream), then: coord wake-fault clear <project>/<thread> --note \"<what was wrong and what you did>\"";

fn state(fault: &Value) -> String {
    if truthy(fault, "clearedAt") {
        format!(
            "cleared by {}: {}",
            cell(fault.get("clearedBy")),
            cell(fault.get("note"))
        )
    } else {
        "open".to_owned()
    }
}

fn row(fault: &Value, now: i64) -> Vec<String> {
    vec![
        format!(
            "{}/{}",
            cell(fault.get("project")),
            cell(fault.get("thread"))
        ),
        format!("{} ago", out::ago(fault["since"].as_str(), now)),
        cell(fault.get("reason")),
        cell(fault.get("detail")),
        state(fault),
    ]
}

fn list(call: &mut Call<'_>) -> Outcome {
    let path = if call.options.flag("all") {
        "/api/wake-faults?all=1"
    } else {
        "/api/wake-faults"
    };
    let answer = connect(call, false)?.get(path)?;
    let faults = list_of(&answer, "faults");
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, &json!(faults));
        return Ok(());
    }
    if faults.is_empty() {
        call.inv.say(&["no wake faults."]);
        return Ok(());
    }
    let now = call.inv.clock.now_ms();
    let mut rows = vec![
        ["project/thread", "since", "reason", "detail", "state"]
            .map(str::to_owned)
            .to_vec(),
    ];
    rows.extend(faults.iter().map(|f| row(f, now)));
    call.inv.say(&[&out::table(&rows)]);
    if faults.iter().any(|f| !truthy(f, "clearedAt")) {
        call.inv.say(&[FIND_OUT]);
    }
    Ok(())
}

fn is_thread(text: &str) -> bool {
    (1..=60).contains(&text.chars().count())
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~:@+-".contains(c))
}

/// `<project>/<thread>` split.
fn target_of(text: Option<&String>) -> Result<(String, String), CoordError> {
    let (project, thread) = text
        .and_then(|t| t.split_once('/'))
        .filter(|(p, t)| is_key(p) && is_thread(t))
        .ok_or_else(|| CoordError::refuse(NAMING))?;
    Ok((project.to_owned(), thread.to_owned()))
}

fn note_of(call: &Call<'_>) -> Result<String, CoordError> {
    bounded(
        call,
        (NOTE_MIN, NOTE_MAX),
        "say what was wrong and what you did",
    )
}

fn clear(call: &mut Call<'_>) -> Outcome {
    let (project, thread) = target_of(call.positional.first())?;
    let note = note_of(call)?;
    let by = overlord::actor(call, "clear a wake fault", ADVICE)?;
    let body = json!({ "project": project, "thread": thread, "note": note, "by": by });
    let answer = connect(call, false)?.post("/api/wake-faults/clear", &body)?;
    let since = cell(answer["fault"].get("since"));
    call.inv.say(&[&format!(
        "cleared the wake fault of {project}/{thread} (since {since}): {note}"
    )]);
    Ok(())
}

/// `coord wake-fault <list|clear>`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        None | Some("list") => list(call),
        Some("clear") => clear(call),
        Some(other) => Err(CoordError::refuse(format!(
            "unknown wake-fault command \"{other}\" (list, clear)."
        ))),
    }
}
