//! The `bulk` op: up to 50 ops applied one after the other, each judged on its own.

use std::path::Path;

use serde_json::{Map, Value};

use super::apply_op;
use super::result::{OpResult, Status};
use super::who::Who;
use crate::Board;
use crate::error::{BoardError, refuse};

const BULK_MAX: usize = 50;

/// `{ force: op.force, ...sub }`: the bulk's force applies unless the sub-op sets its own.
fn merged(op: &Value, sub: &Value) -> Value {
    let mut out = Map::new();
    if let Some(force) = op.get("force") {
        out.insert("force".into(), force.clone());
    }
    if let Some(fields) = sub.as_object() {
        out.extend(fields.clone());
    }
    Value::Object(out)
}

fn run_sub(
    board: &mut Board,
    (op, sub): (&Value, &Value),
    who: &Who,
    home: Option<&Path>,
) -> OpResult {
    match sub["kind"].as_str() {
        Some(kind @ ("bulk" | "purge" | "build" | "avatar")) => {
            OpResult::refused(format!("a {kind} cannot be inside a bulk."))
        }
        _ => apply_op(board, &merged(op, sub), who, home),
    }
}

/// Apply a `bulk` op.
pub(crate) fn handle(
    board: &mut Board,
    op: &Value,
    who: &Who,
    home: Option<&Path>,
) -> Result<OpResult, BoardError> {
    let ops = match op["ops"].as_array() {
        Some(ops) if !ops.is_empty() => ops,
        _ => return Err(refuse("a bulk needs ops.")),
    };
    if ops.len() > BULK_MAX {
        return Err(refuse(format!(
            "a bulk holds at most {BULK_MAX} ops, this has {}.",
            ops.len()
        )));
    }
    let results: Vec<OpResult> = ops
        .iter()
        .map(|sub| run_sub(board, (op, sub), who, home))
        .collect();
    let count = |s: Status| results.iter().filter(|r| r.status == s).count();
    let (ok, clash, nope) = (
        count(Status::Applied),
        count(Status::Conflict),
        count(Status::Refused),
    );
    let status = if ok > 0 {
        Status::Applied
    } else if clash > 0 {
        Status::Conflict
    } else {
        Status::Refused
    };
    let part = |n: usize, word: &str| {
        if n > 0 {
            format!(", {n} {word}")
        } else {
            String::new()
        }
    };
    let outcome = format!(
        "{ok} of {} applied{}{}",
        ops.len(),
        part(clash, "conflict"),
        part(nope, "refused")
    );
    let mut result = OpResult::refused(outcome);
    result.status = status;
    result.notify = results.iter().flat_map(|r| r.notify.clone()).collect();
    result.results = Some(results.iter().map(OpResult::summary).collect());
    Ok(result)
}
