//! The runs of a test plan: `plan-run-start`, `plan-run-record` and `plan-run-end`. A run is kept
//! on the plan item in `fields.runs`:
//! `{ n, bundle, by, startedAt, endedAt, results: [{ test, status, note, at, proofs }], summary }`.
//! Recording a result in a run also records it on the test through the same result the test verb
//! uses, so the test's own `result`, `runs` and the `test-fail` wake stay true; a skip stays in
//! the run only. A plan has one open run at a time. Starting moves the plan to `running`, ending
//! moves it to `reported`.

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use super::announce::announce;
use super::authorize::{Check, authorize};
use super::result::OpResult;
use super::target::target_of;
use super::who::Who;
use crate::Board;
use crate::content::{Context, check as content_check};
use crate::error::{BoardError, refuse};
use crate::json::{array_mut, display, doc, doc_mut, ids, map_mut, set, truthy};
use crate::limits::{optional_text, require_text};
use crate::registry::registry;
use crate::results::{Outcome, record_result};
use crate::store::stamp;

const PLAN: &str = "test-plan";
const STATUSES: [&str; 5] = ["pass", "fail", "partial", "blocked", "skip"];
const NEEDS_NOTE: [&str; 3] = ["fail", "partial", "blocked"];

/// Whether an op kind is one of the plan run ops.
pub(crate) fn is_kind(kind: &str) -> bool {
    matches!(kind, "plan-run-start" | "plan-run-record" | "plan-run-end")
}

/// The test plan an op points at: its id and the doc.
fn plan_of(board: &Board, op: &Value) -> Result<(String, Value), BoardError> {
    let (coll, id) = target_of(board, op, false)?;
    let plan = doc(board, &coll, &id).cloned().unwrap_or(Value::Null);
    if coll != "items" || plan["type"] != PLAN {
        return Err(refuse(format!("{id} is not a test plan.")));
    }
    Ok((id, plan))
}

fn runs_of(plan: &Value) -> Vec<Value> {
    plan["fields"]["runs"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn cases_of(plan: &Value) -> Vec<String> {
    let cases = plan["fields"]["cases"].as_array();
    let listed = cases.into_iter().flatten();
    listed
        .filter_map(|c| c.as_str().map(str::to_owned))
        .collect()
}

fn open_index(runs: &[Value]) -> Option<usize> {
    runs.iter().position(|r| r["endedAt"].is_null())
}

fn open_run(runs: &[Value], id: &str) -> Result<usize, BoardError> {
    open_index(runs).ok_or_else(|| {
        refuse(format!(
            "{id} has no open run; start one with coord test plan start {id}."
        ))
    })
}

/// The latest status a run holds for a case, `None` while the case has no result.
fn last_status<'a>(run: &'a Value, case: &str) -> Option<&'a str> {
    let results = run["results"].as_array().map_or(&[][..], Vec::as_slice);
    let latest = results
        .iter()
        .rev()
        .find(|r| r["test"].as_str() == Some(case));
    latest.and_then(|r| r["status"].as_str())
}

/// The counts of a run by status, over the cases of the plan; a case without a result is `notRun`.
fn summary_of(run: &Value, cases: &[String]) -> Value {
    let count = |status: Option<&str>| {
        cases
            .iter()
            .filter(|c| last_status(run, c) == status)
            .count()
    };
    json!({
        "pass": count(Some("pass")), "fail": count(Some("fail")),
        "partial": count(Some("partial")), "blocked": count(Some("blocked")),
        "skip": count(Some("skip")), "notRun": count(None), "total": cases.len(),
    })
}

/// Write the runs of the plan, and move it into a stage when one is given. The stage's lane
/// becomes the item's status.
fn save(
    board: &mut Board,
    id: &str,
    runs: Vec<Value>,
    stage: Option<&str>,
) -> Result<(), BoardError> {
    let Some(plan) = doc_mut(board, "items", id) else {
        return Err(refuse("gone"));
    };
    map_mut(plan, "fields").insert("runs".to_owned(), Value::Array(runs));
    let lane = stage.and_then(|s| {
        let ty = registry().item_type(PLAN)?;
        ty.stage(s).map(|st| st.lane.clone())
    });
    if let (Some(stage), Some(lane)) = (stage, lane) {
        set(plan, "stage", stage);
        set(plan, "status", lane);
    }
    Ok(())
}

fn bundle_of(board: &Board, op: &Value, plan: &Value) -> Result<Value, BoardError> {
    let Some(named) = op.get("bundle").filter(|v| !v.is_null()) else {
        return Ok(plan["fields"]["bundle"].clone());
    };
    let known = ids(board, "builds");
    let cx = Context {
        ids: Some(known.as_slice()),
        ..Context::default()
    };
    content_check("bundle", named, &cx)
}

fn start(
    board: &mut Board,
    (op, who): (&Value, &Who),
    (id, plan): (&str, &Value),
) -> Result<OpResult, BoardError> {
    let mut runs = runs_of(plan);
    if let Some(open) = open_index(&runs) {
        return Err(refuse(format!(
            "{id} already has run {} open; end it first.",
            display(&runs[open]["n"])
        )));
    }
    if cases_of(plan).is_empty() {
        return Err(refuse(format!(
            "{id} has no test case yet (add one with coord test plan cases)."
        )));
    }
    let n = runs
        .iter()
        .filter_map(|r| r["n"].as_i64())
        .fold(0, i64::max)
        + 1;
    runs.push(json!({
        "n": n, "bundle": bundle_of(board, op, plan)?, "by": who.actor_of(),
        "startedAt": now_iso(), "endedAt": null, "results": [],
    }));
    save(board, id, runs, Some("running"))?;
    finish(board, who, id, format!("{id} run {n} started"), n)
}

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
struct Given {
    test: String,
    status: &'static str,
    note: String,
    proofs: Value,
}

impl Given {
    fn of(board: &Board, who: &Who, op: &Value, plan: &Value) -> Result<Self, BoardError> {
        let test = case_of(board, who, op, plan)?;
        let status = status_of(op)?;
        Ok(Self {
            note: note_of(status, op)?,
            proofs: proofs_of(op, &test)?,
            test: display(&test["id"]),
            status,
        })
    }

    fn entry(&self) -> Value {
        json!({
            "test": self.test, "status": self.status, "note": self.note, "at": now_iso(),
            "proofs": self.proofs,
        })
    }

    /// Record the result on the test itself; a skip stays in the run.
    fn on_test(
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

fn record(
    board: &mut Board,
    (op, who): (&Value, &Who),
    (id, plan): (&str, &Value),
) -> Result<OpResult, BoardError> {
    let mut runs = runs_of(plan);
    let at = open_run(&runs, id)?;
    let given = Given::of(board, who, op, plan)?;
    let n = runs[at]["n"].clone();
    given.on_test(board, &who.actor_of(), (id, &n))?;
    array_mut(&mut runs[at], "results").push(given.entry());
    let (done, total) = progress(&runs[at], plan);
    save(board, id, runs, None)?;
    stamp(board, "items", id, &who.actor_of())?;
    let said = format!(
        "{}: {} in run {} of {id} ({done} of {total} cases)",
        given.test,
        given.status,
        display(&n)
    );
    Ok(OpResult::applied(said).with_extra("run", n))
}

fn progress(run: &Value, plan: &Value) -> (usize, usize) {
    let cases = cases_of(plan);
    let done = cases
        .iter()
        .filter(|c| last_status(run, c).is_some())
        .count();
    (done, cases.len())
}

fn end(board: &mut Board, who: &Who, (id, plan): (&str, &Value)) -> Result<OpResult, BoardError> {
    let mut runs = runs_of(plan);
    let at = open_run(&runs, id)?;
    let summary = summary_of(&runs[at], &cases_of(plan));
    set(&mut runs[at], "endedAt", now_iso());
    set(&mut runs[at], "summary", summary.clone());
    let n = runs[at]["n"].as_i64().unwrap_or_default();
    save(board, id, runs, Some("reported"))?;
    let said = format!(
        "{id} run {n} ended: {} pass, {} fail, {} partial, {} blocked, {} skip, {} not run",
        display(&summary["pass"]),
        display(&summary["fail"]),
        display(&summary["partial"]),
        display(&summary["blocked"]),
        display(&summary["skip"]),
        display(&summary["notRun"]),
    );
    finish(board, who, id, said, n).map(|r| r.with_extra("summary", summary))
}

/// Stamp the plan, tell its thread when someone else changed it, and say how it went.
fn finish(
    board: &mut Board,
    who: &Who,
    id: &str,
    said: String,
    n: i64,
) -> Result<OpResult, BoardError> {
    stamp(board, "items", id, &who.actor_of())?;
    let owner = doc(board, "items", id).and_then(|p| p["thread"].as_str().map(str::to_owned));
    let notify = announce(board, who, &[owner], &said, Some(id));
    Ok(OpResult::applied(said)
        .with_extra("run", json!(n))
        .with_notify(notify))
}

/// Apply a plan run op as `who`.
pub(crate) fn handle(board: &mut Board, op: &Value, who: &Who) -> Result<OpResult, BoardError> {
    let (id, plan) = plan_of(board, op)?;
    authorize(who, &Check::new("edit", "items", Some(&plan)))?;
    let target = (id.as_str(), &plan);
    match op["kind"].as_str() {
        Some("plan-run-start") => start(board, (op, who), target),
        Some("plan-run-record") => record(board, (op, who), target),
        _ => end(board, who, target),
    }
}
