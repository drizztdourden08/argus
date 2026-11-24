//! `coord lead <project-key> "<text>" [--kind note|decision]`: lead to lead. One project's lead
//! messages another project's lead, with no thread of its own on the other board. The message
//! is queued in the target lead's inbox; the dashboard is then told, best effort. A decision asks
//! for action and wakes the lead (`lead-message`); a note is information only and stays a passive
//! notice on its board (`lead-info`), handed over with its next prompt, except to the Overlord,
//! whose notes always wake it. A dashboard that is down never fails the verb, but a wake that
//! did not go out is said on stderr.
// test-tag: critical

mod deliver;

use std::fmt::Display;

use argus_board::limits::require_text;
use argus_board::portfolio::project_keys;
use argus_board::roles::is_overlord;
use argus_board::store::{board_path, read_board};
use argus_core::ids::PORTFOLIO;
use argus_core::js;
use serde_json::json;

use super::support::link::connect;
use super::support::scope::coord_home;
use super::support::text::is_key;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::who::{board_ref, find_project, refuse_while_migrating, shown};
use deliver::{Note, deliver};

const WAKE_CLIP: usize = 100;

fn own_key(call: &Call<'_>) -> Result<String, CoordError> {
    let env = &call.inv.env;
    find_project(env, &call.inv.cwd, &call.options, env.session_id()).map_err(|_| {
        CoordError::refuse("coord lead must run inside a project checkout: no .ai-config.json above the cwd, so the sender is unknown. Run it from the project, or pass --project <your key>.")
    })
}

fn kind_of(call: &Call<'_>) -> Result<&'static str, CoordError> {
    match call.options.text("kind").unwrap_or("note") {
        "note" => Ok("followup"),
        "decision" => Ok("decision"),
        other => Err(CoordError::refuse(format!(
            "--kind must be note or decision, not \"{other}\"."
        ))),
    }
}

fn no_board(target: &str) -> CoordError {
    CoordError::refuse(format!(
        "project \"{target}\" has no board yet, so no lead to message."
    ))
}

/// The projects the Overlord's registry and the boards on disk know.
fn known(call: &Call<'_>) -> Result<Vec<String>, CoordError> {
    let home = coord_home(call);
    let registry = read_board(&home, PORTFOLIO)?.unwrap_or_else(|| json!({}));
    Ok(project_keys(
        &home,
        &json!({ "projects": registry["projects"] }),
    ))
}

fn check_target(call: &Call<'_>, from: &str, target: &str) -> Result<(), CoordError> {
    if from == target {
        return Err(CoordError::refuse(format!(
            "{target} is your own project: use coord inbox send <thread> for a thread of this board."
        )));
    }
    let keys = known(call)?;
    if !keys.iter().any(|k| k == target) {
        let others: Vec<&str> = keys
            .iter()
            .map(String::as_str)
            .filter(|k| *k != from)
            .collect();
        let listed = if others.is_empty() {
            "none".to_owned()
        } else {
            others.join(", ")
        };
        return Err(CoordError::refuse(format!(
            "unknown project \"{target}\" (known: {listed}). coord portfolio add it first."
        )));
    }
    if !board_path(&coord_home(call), target).exists() {
        return Err(no_board(target));
    }
    Ok(())
}

/// The wake reason of a message kind: a decision asks the other lead for action, and so does any
/// message to the Overlord, which has no other way to hear of it.
fn reason_of(kind: &str, to_overlord: bool) -> &'static str {
    if kind == "decision" || to_overlord {
        "lead-message"
    } else {
        "lead-info"
    }
}

/// The stderr line for a wake that did not go out, naming what failed.
fn failure_line(key: &str, step: &str, error: &dyn Display) -> String {
    format!("coord: the wake for {key} was not sent ({step}: {error}); the message is queued.")
}

/// The nudge: the target lead is told (a wake for a decision or for the Overlord, a notice for a
/// note). A failure never fails the verb, but it is said on stderr.
fn wake(call: &mut Call<'_>, key: &str, thread: &str, note: &Note<'_>) {
    let to_overlord = is_overlord(&coord_home(call), key);
    let line = format!("{} lead: {}", note.from, js::slice(note.text, WAKE_CLIP));
    let reason = reason_of(note.kind, to_overlord);
    let body = json!({ "thread": thread, "reason": reason, "text": line });
    let failed = match connect(call, false) {
        Err(e) => Some(failure_line(key, "the dashboard is not reachable", &e)),
        Ok(client) => client
            .post(&format!("/api/p/{key}/wakes"), &body)
            .err()
            .map(|e| failure_line(key, "the wake request failed", &e)),
    };
    if let Some(text) = failed {
        call.inv.warn(&text);
    }
}

/// `coord lead <project-key> "<text>"`.
///
/// # Errors
/// A refusal: the arguments, the sender, or a target with no board.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let target = call.positional.first().filter(|t| is_key(t)).cloned();
    let Some(target) = target else {
        return Err(CoordError::refuse(
            "name the project to message: coord lead <project-key> \"<text>\".",
        ));
    };
    let kind = kind_of(call)?;
    let text = require_text("lead", &json!(call.positional.get(1)))?;
    let from = own_key(call)?;
    let home = coord_home(call);
    refuse_while_migrating(&home)?;
    let target = board_ref(&home, &target)?;
    check_target(call, &from, &target)?;
    let shown_from = shown(&home, &from);
    let note = Note {
        kind,
        text: &text,
        from: &shown_from,
    };
    let Some(thread) = deliver(&home, &target, &note)? else {
        return Err(no_board(&target));
    };
    wake(call, &target, &thread, &note);
    call.inv.say(&[&format!(
        "queued for {} lead: {text}",
        shown(&home, &target)
    )]);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_note_to_the_overlord_wakes_it() {
        assert_eq!(reason_of("followup", true), "lead-message");
        assert_eq!(reason_of("followup", false), "lead-info");
        assert_eq!(reason_of("decision", false), "lead-message");
    }

    #[test]
    fn a_failed_wake_is_named_in_the_warning() {
        let line = failure_line("bja1s18", "the wake request failed", &"timed out");
        assert!(line.contains("bja1s18"));
        assert!(line.contains("the wake request failed: timed out"));
    }
}
