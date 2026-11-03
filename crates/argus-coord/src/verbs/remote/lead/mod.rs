//! `coord lead <project-key> "<text>" [--kind note|decision]`: lead to lead. One project's lead
//! messages another project's lead, with no thread of its own on the other board. The message
//! is queued in the target lead's inbox; the dashboard is then told, best effort. A decision asks
//! for action and wakes the lead (`lead-message`); a note is information only and stays a passive
//! notice on its board (`lead-info`), handed over with its next prompt. A dashboard that is down
//! never fails the verb.

mod deliver;

use argus_board::limits::require_text;
use argus_board::portfolio::project_keys;
use argus_board::store::{board_path, read_board};
use argus_core::ids::PORTFOLIO;
use argus_core::js;
use serde_json::json;

use super::support::link::connect;
use super::support::scope::coord_home;
use super::support::text::is_key;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::who::find_project;
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

/// The wake reason of a message kind: only a decision asks the other lead for action.
fn reason_of(kind: &str) -> &'static str {
    if kind == "decision" {
        "lead-message"
    } else {
        "lead-info"
    }
}

/// The nudge: the target lead is told (a wake for a decision, a notice for a note). Nothing
/// here ever fails the verb.
fn wake(call: &Call<'_>, key: &str, thread: &str, note: &Note<'_>) {
    let Ok(client) = connect(call, false) else {
        return;
    };
    let line = format!("{} lead: {}", note.from, js::slice(note.text, WAKE_CLIP));
    let body = json!({ "thread": thread, "reason": reason_of(note.kind), "text": line });
    client.post(&format!("/api/p/{key}/wakes"), &body).ok();
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
    check_target(call, &from, &target)?;
    let note = Note {
        kind,
        text: &text,
        from: &from,
    };
    let Some(thread) = deliver(&coord_home(call), &target, &note)? else {
        return Err(no_board(&target));
    };
    wake(call, &target, &thread, &note);
    call.inv
        .say(&[&format!("queued for {target} lead: {text}")]);
    Ok(())
}
