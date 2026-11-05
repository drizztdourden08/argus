//! One result as an op gives it, checked, and its record on the test.

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::runs::cases_of;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, truthy};
use crate::limits::{optional_text, require_text};
use crate::ops::authorize::{Check, authorize};
use crate::ops::who::Who;
use crate::results::{Outcome, record_result};

const STATUSES: [&str; 5] = ["pass", "fail", "partial", "blocked", "skip"];
const NEEDS_NOTE: [&str; 3] = ["fail", "partial", "blocked"];

/// The test a result is about: a live case of the plan the caller may record on.
fn case_of(board: &Board, who: &Who, op: &Value, plan: &Value) -> Result<Value, BoardError> {
    let test = display(&op["test"]);
    if !cases_of(plan).contains(&test) {
        return Err(refuse(format!(
            "{test} is not a case of {}; its cases are {}.",
            display(&plan["id"]),
            cases_of(plan).join(", ")
        )));
    }
    let found = doc(board, "tests", &test).filter(|t| !truthy(t, "deletedAt"));
    let found = found.ok_or_else(|| refuse(format!("no test \"{test}\".")))?;
    authorize(who, &Check::new("result", "tests", Some(found)))?;
    Ok(found.clone())
}

fn status_of(op: &Value) -> Result<&'static str, BoardError> {
    let asked = op["status"].as_str();
    let status = STATUSES.iter().find(|s| Some(**s) == asked);
    status
        .copied()
        .ok_or_else(|| refuse(format!("status must be one of {}.", STATUSES.join(", "))))
}

/// The note of a result: required for a fail, partial or blocked, free for the rest.
fn note_of(status: &str, op: &Value) -> Result<String, BoardError> {
    if NEEDS_NOTE.contains(&status) {
        return require_text("result", &op["note"]);
    }
    Ok(optional_text("result", &op["note"])?.unwrap_or_default())
}

/// The proof numbers a result points at: each must be a proof of the test.
fn proofs_of(op: &Value, test: &Value) -> Result<Value, BoardError> {
    let given = op["proofs"].as_array().cloned().unwrap_or_default();
    let have: Vec<i64> = test["proofs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|p| p["n"].as_i64())
        .collect();
    for n in &given {
        if !n.as_i64().is_some_and(|n| have.contains(&n)) {
            return Err(refuse(format!(
                "{} has no proof {}.",
                display(&test["id"]),
                display(n)
            )));
        }
    }
    Ok(Value::Array(given))
}

/// One result as the op gives it, checked.
pub(super) struct Given {
    pub test: String,
    pub status: &'static str,
    note: String,
    proofs: Value,
}

impl Given {
    pub(super) fn of(
        board: &Board,
        who: &Who,
        op: &Value,
        plan: &Value,
    ) -> Result<Self, BoardError> {
        let test = case_of(board, who, op, plan)?;
        let status = status_of(op)?;
        Ok(Self {
            note: note_of(status, op)?,
            proofs: proofs_of(op, &test)?,
            test: display(&test["id"]),
            status,
        })
    }

    /// The record kept in the run.
    pub(super) fn entry(&self, by: &str) -> Value {
        json!({
            "test": self.test, "status": self.status, "note": self.note, "at": now_iso(),
            "by": by, "proofs": self.proofs,
        })
    }

    /// Record the result on the test itself; a skip stays in the run.
    pub(super) fn on_test(
        &self,
        board: &mut Board,
        by: &str,
        (id, n): (&str, &Value),
    ) -> Result<(), BoardError> {
        if self.status == "skip" {
            return Ok(());
        }
        let text = if self.note.is_empty() {
            format!("pass in run {} of {id}", display(n))
        } else {
            self.note.clone()
        };
        let status = json!(self.status);
        record_result(
            board,
            &self.test,
            &Outcome {
                status: &status,
                text: &text,
                by,
            },
        )
    }
}
