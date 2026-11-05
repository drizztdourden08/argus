//! Starting a run: the plan's ready run when it has one, else a new run.

use argus_core::clock::now_iso;
use serde_json::Value;

use super::build::named;
use super::done::finish;
use super::fresh::create;
use super::place::{plan_to, run_to};
use super::runs::{cases_of, is_open, runs_of};
use super::target::Found;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, doc_mut, map_mut, set};
use crate::ops::result::OpResult;
use crate::ops::who::Who;

fn next_n(runs: &[Value]) -> i64 {
    runs.iter()
        .filter_map(|r| r["n"].as_i64())
        .fold(0, i64::max)
        + 1
}

/// The run that starts: the one the op points at, else the plan's ready one.
fn ready_of(found: &Found, runs: &[Value]) -> Result<Option<Value>, BoardError> {
    let Some(run) = &found.run else {
        return Ok(runs.iter().find(|r| r["stage"] == "ready").cloned());
    };
    if run["stage"] != "ready" {
        return Err(refuse(format!(
            "{} is {}; only a ready run starts.",
            display(&run["id"]),
            display(&run["stage"])
        )));
    }
    Ok(Some(run.clone()))
}

/// Start a run that was made ready: the build the op names replaces its own.
fn begin(board: &mut Board, op: &Value, run: &Value) -> Result<String, BoardError> {
    let id = display(&run["id"]);
    let build = named(board, op)?;
    run_to(board, &id, "running")?;
    if let Some(item) = doc_mut(board, "items", &id) {
        if let Some(build) = build {
            map_mut(item, "fields").insert("build".to_owned(), build);
        }
        set(item, "startedAt", now_iso());
    }
    Ok(id)
}

pub(super) fn start(
    board: &mut Board,
    (op, who): (&Value, &Who),
    found: &Found,
) -> Result<OpResult, BoardError> {
    let id = found.plan_id.as_str();
    let runs = runs_of(board, id);
    if let Some(open) = runs.iter().find(|r| is_open(r)) {
        return Err(refuse(format!(
            "{id} already has run {} open; end it first.",
            display(&open["n"])
        )));
    }
    if cases_of(&found.plan).is_empty() {
        return Err(refuse(format!(
            "{id} has no test case yet (add one with coord test plan cases)."
        )));
    }
    let run_id = if let Some(ready) = ready_of(found, &runs)? {
        begin(board, op, &ready)?
    } else {
        let build = named(board, op)?;
        create(board, (op, who), (found, next_n(&runs), build))?
    };
    plan_to(board, id, "running")?;
    let n = doc(board, "items", &run_id).map_or_else(String::new, |r| display(&r["n"]));
    finish(board, who, (id, &run_id), format!("{id} run {n} started"))
}
