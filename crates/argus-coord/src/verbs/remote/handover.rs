//! `coord handover estimate | write | status`: a lead's side of the handover before an account
//! switch (I-135). Argus wakes a working lead with `handover-estimate` or `handover-request`;
//! the lead answers here and Argus keeps its `H-n` record.
//!
//! - `estimate --percent P --minutes M --agents N [--note "<why>"]`: what the handover would
//!   cost, then the lead keeps working
//! - `write [--file <path>] [--agent "<id>|<branch>|<stop point>"]...`: the handover file is
//!   written (default `~/.coord/<board>/handover.md`; refused when missing or empty), the
//!   record is ready
//! - `status`: the record and what Argus expects next
//!
//! Only the lead of the board (its lead thread, the Overlord on `portfolio`) answers.

use std::path::PathBuf;

use argus_core::js;
use serde_json::{Value, json};

use super::support::link::{connect, project_of, take_sub};
use super::support::scope::{coord_home, require_board};
use super::support::text::{cell, collapse, int_within, number_of};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::out;
use crate::cli::route::Call;
use crate::verbs::shared::caller::is_lead;
use crate::who::caller;

const NOTE_MAX: usize = 300;
const AGENT_MAX: usize = 300;
const AGENTS_MAX: f64 = 50.0;
const PATH: &str = "/api/accounts/handover";
const ESTIMATE_USAGE: &str =
    "coord handover estimate --percent <points> --minutes <m> --agents <n> [--note \"<why>\"]";

/// The lead of the call's board.
fn lead(call: &Call<'_>) -> Result<(String, String), CoordError> {
    let board_key = project_of(call).to_owned();
    let board = require_board(call, &board_key)?;
    let who = caller(&board, &call.inv.cwd, &call.inv.env, &call.options)?;
    if !is_lead(&who) {
        return Err(CoordError::refuse(format!(
            "coord handover is for the lead of {board_key}; {} is not its lead.",
            who.thread
        )));
    }
    Ok((board_key, who.thread))
}

fn number(call: &Call<'_>, name: &str) -> Result<f64, CoordError> {
    let n = call.options.text(name).map_or(f64::NAN, number_of);
    if n.is_finite() && n >= 0.0 {
        Ok(n)
    } else {
        Err(CoordError::refuse(format!(
            "--{name} must be a number of 0 or more: {ESTIMATE_USAGE}"
        )))
    }
}

fn note(call: &Call<'_>) -> Result<Option<String>, CoordError> {
    let Some(raw) = call.options.text("note") else {
        return Ok(None);
    };
    let note = collapse(raw);
    if js::len(&note) > NOTE_MAX {
        return Err(CoordError::refuse(format!(
            "--note is at most {NOTE_MAX} characters (it is {}).",
            js::len(&note)
        )));
    }
    Ok((!note.is_empty()).then_some(note))
}

fn estimate_body(call: &Call<'_>) -> Result<Value, CoordError> {
    let agents = call
        .options
        .text("agents")
        .and_then(|t| int_within(number_of(t), 0.0, AGENTS_MAX))
        .ok_or_else(|| {
            CoordError::refuse(format!(
                "--agents must be a whole number 0 to 50: {ESTIMATE_USAGE}"
            ))
        })?;
    Ok(json!({
        "action": "estimate",
        "percent": number(call, "percent")?,
        "minutes": number(call, "minutes")?,
        "agents": agents,
        "note": note(call)?,
    }))
}

/// The handover file: `--file`, else `~/.coord/<board>/handover.md`; it must hold text.
fn handover_file(call: &Call<'_>, board: &str) -> Result<PathBuf, CoordError> {
    let file = call.options.text("file").map_or_else(
        || coord_home(call).join(board).join("handover.md"),
        |typed| call.inv.cwd.join(typed),
    );
    let text = std::fs::read_to_string(&file).map_err(|_| {
        CoordError::refuse(format!(
            "no handover file at {}: write it first, a section per agent (branch, stop point, next step).",
            file.display()
        ))
    })?;
    if text.trim().is_empty() {
        return Err(CoordError::refuse(format!(
            "the handover file {} is empty: write it first.",
            file.display()
        )));
    }
    Ok(file)
}

/// One `--agent "<id>|<branch>|<stop point>"`.
fn agent_of(typed: &str) -> Result<Value, CoordError> {
    let mut parts = typed.splitn(3, '|').map(str::trim);
    let id = parts.next().filter(|id| !id.is_empty());
    let Some(id) = id.filter(|_| js::len(typed) <= AGENT_MAX) else {
        return Err(CoordError::refuse(format!(
            "--agent is \"<id>|<branch>|<stop point>\", at most {AGENT_MAX} characters."
        )));
    };
    let field = |p: Option<&str>| p.filter(|t| !t.is_empty()).map(str::to_owned);
    Ok(json!({ "id": id, "branch": field(parts.next()), "stop": field(parts.next()) }))
}

fn write_body(call: &Call<'_>, board: &str) -> Result<Value, CoordError> {
    let file = handover_file(call, board)?;
    let agents = call
        .options
        .many("agent")
        .iter()
        .map(|v| agent_of(&js::string(&v.to_json())))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "action": "write",
        "file": file.display().to_string(),
        "agentsDetail": agents,
    }))
}

fn report(call: &mut Call<'_>, answer: &Value) {
    if call.options.flag("json") {
        out::as_json(&mut call.inv.stdout, answer);
        return;
    }
    let h = &answer["handover"];
    let head = format!(
        "{} of {} ({}/{}): {}. Next: {}.",
        cell(h.get("id")),
        cell(h.get("cycle")),
        cell(h.get("board")),
        cell(h.get("thread")),
        cell(h.get("state")).replace('_', " "),
        cell(answer.get("next")),
    );
    let expects = cell(answer.get("expects"));
    call.inv.say(&[&head, &expects]);
}

fn send(call: &mut Call<'_>, sub: &str) -> Outcome {
    let (board, thread) = lead(call)?;
    let mut body = match sub {
        "estimate" => estimate_body(call)?,
        "write" => write_body(call, &board)?,
        _ => json!({ "action": "status" }),
    };
    body["board"] = json!(board);
    body["thread"] = json!(thread);
    let answer = connect(call, false)?.post(PATH, &body)?;
    report(call, &answer);
    Ok(())
}

/// `coord handover <estimate|write|status>`.
///
/// # Errors
/// A refusal, or the dashboard's own.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
    match sub.as_deref() {
        None | Some("status") => send(call, "status"),
        Some(verb @ ("estimate" | "write")) => send(call, verb),
        Some(other) => Err(CoordError::refuse(format!(
            "unknown handover command \"{other}\" (estimate, write, status)."
        ))),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::agent_of;

    #[test]
    fn an_agent_reads_id_branch_and_stop_point() {
        let agent = agent_of("A-12 | wip/sync | tests pass, docs left").unwrap();
        assert_eq!(
            agent,
            json!({"id": "A-12", "branch": "wip/sync", "stop": "tests pass, docs left"})
        );
        assert_eq!(
            agent_of("A-3").unwrap(),
            json!({"id": "A-3", "branch": null, "stop": null})
        );
        assert!(agent_of(" | x").is_err());
    }
}
