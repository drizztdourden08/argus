//! `coord stall list [--project k]` and `coord stall ack <project> --note "<what you did>"`: the
//! stalls Argus raised (a thread that did not read, a request left unstarted, an idle lead, a
//! silent item). Anyone may list; only the Overlord acknowledges, after it looked into it.

use serde_json::json;

use super::support::link::{connect, take_sub};
use super::support::note::bounded;
use super::support::overlord;
use super::support::query::Query;
use super::support::text::{cell, is_key};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;

const NOTE_MIN: usize = 8;
const NOTE_MAX: usize = 300;
const ADVICE: &str = "A stall is acknowledged after the Overlord looked into it.";
const NAMING: &str = "name the project: coord stall ack <project> --note \"<what you did>\".";

fn list(call: &mut Call<'_>) -> Outcome {
    let mut query = Query::new();
    if let Some(project) = call.options.text("project") {
        query = query.with("project", project);
    }
    let answer = connect(call, false)?.get(&format!("/api/stalls{}", query.suffix()))?;
    let text = cell(answer.get("text"));
    let line = if text.is_empty() {
        "no open stalls."
    } else {
        &text
    };
    call.inv.say(&[line]);
    Ok(())
}

fn note_of(call: &Call<'_>) -> Result<String, CoordError> {
    bounded(
        call,
        (NOTE_MIN, NOTE_MAX),
        "say what you did about the stall",
    )
}

fn ack(call: &mut Call<'_>) -> Outcome {
    let project = call
        .positional
        .first()
        .filter(|p| is_key(p))
        .cloned()
        .ok_or_else(|| CoordError::refuse(NAMING))?;
    let note = note_of(call)?;
    let by = overlord::actor(call, "acknowledge a stall", ADVICE)?;
    let body = json!({ "project": project, "note": note, "by": by });
    let answer = connect(call, false)?.post("/api/stalls/ack", &body)?;
    call.inv.say(&[&cell(answer.get("text"))]);
    Ok(())
}

/// `coord stall <list|ack>`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        None | Some("list") => list(call),
        Some("ack") => ack(call),
        Some(other) => Err(CoordError::refuse(format!(
            "unknown stall command \"{other}\" (list, ack)."
        ))),
    }
}
