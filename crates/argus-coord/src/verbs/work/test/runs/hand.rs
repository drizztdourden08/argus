//! `coord test run hand <run>` and `unhand`, `ready` and `draft`: handing a run is making it
//! ready (its build set and live) and assigning it to the owner, who tests it. A project with no
//! master launcher cannot hand anything.

use argus_board::Board;
use argus_board::json::display;
use argus_launchers::launcher::require_master;
use serde_json::{Value, json};

use super::change::change;
use crate::cli::Outcome;
use crate::cli::exit::CoordError;
use crate::verbs::agent::rules::typed_board;
use crate::verbs::work::run::{Res, Run, bad};

/// Refuses a project with no master launcher.
fn master_ready(board: &Board) -> Res<()> {
    let typed = typed_board(board).map_err(|e| match e {
        CoordError::Refused(message) => bad(message),
        CoordError::Exit(code) => bad(format!("exit {code}")),
    })?;
    require_master(&typed).map_err(|e| bad(e.0))
}

/// Refuses a run that is past ready: only a draft or ready run is handed.
fn handable(run: &Value) -> Res<()> {
    match run["stage"].as_str() {
        Some("draft" | "ready") => Ok(()),
        other => Err(bad(format!(
            "{} is {}: only a draft or ready run is handed to the owner.",
            display(&run["id"]),
            other.unwrap_or("not started")
        ))),
    }
}

/// `coord test run hand`.
pub(super) fn hand(r: &mut Run<'_>) -> Outcome {
    let fields = json!({ "stage": "ready", "assignee": "owner" });
    let check = |board: &Board, run: &Value| {
        handable(run)?;
        master_ready(board)
    };
    let say = |run: &Value, id: &str| {
        format!(
            "{id}: ready, handed to the owner (build {}).",
            display(&run["fields"]["build"])
        )
    };
    change(r, fields, check, say)
}

/// `coord test run unhand`.
pub(super) fn unhand(r: &mut Run<'_>) -> Outcome {
    let say = |_: &Value, id: &str| format!("{id}: no longer handed to the owner.");
    change(r, json!({ "assignee": null }), |_, _| Ok(()), say)
}

/// `coord test run ready` and `draft`: the stage a draft or ready run moves to.
pub(super) fn stage(r: &mut Run<'_>, stage: &str) -> Outcome {
    let say = |_: &Value, id: &str| format!("{id}: {stage}.");
    change(r, json!({ "stage": stage }), |_, run| handable(run), say)
}
