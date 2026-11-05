//! The migration from bundles to test runs (item I-99), pure and idempotent.
//!
//! | Legacy | Becomes |
//! |---|---|
//! | `plan.fields.runs[k]` | a test run of that plan: `n`, results, summary, times, `by`; the build is the `launcher` of its bundle record; `legacyBundle` keeps the tag |
//! | a bundle tag (`board.builds` and `build` on live tests) | one run of the plan whose `fields.bundle` is the tag, else of a new plan holding the tagged tests as cases |
//! | open or draft bundle | a draft run, or a ready one when its build is live |
//! | the handed bundle | a ready run assigned to the owner |
//! | closed bundle | a passed run when every tagged test passed, else a reported one |
//!
//! A bundle tag a migrated plan run already carries makes no second run. Every legacy field stays in
//! place, unread, for a later purge. `board.migrations.testRuns` marks the board as done.

mod bundles;
mod make;
mod plan_runs;
mod status;
#[cfg(test)]
mod tests;

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use crate::Board;
use crate::error::BoardError;
use crate::json::{doc, ids, map_mut, section, truthy};

const PLAN: &str = "test-plan";
const RUN: &str = "test-run";

/// What the migration did, or would do on a copy of the board.
#[derive(Debug, Default)]
pub struct Report {
    /// The board was marked done before: nothing was touched.
    pub already: bool,
    /// One line for each plan and run made, in order.
    pub lines: Vec<String>,
    /// Runs made from `fields.runs` of plans.
    pub from_plan_runs: usize,
    /// Runs made from bundle tags.
    pub from_bundles: usize,
    /// Plans made for bundle tags that had none.
    pub plans: usize,
}

/// Whether the board carries the mark.
#[must_use]
pub fn is_migrated(board: &Board) -> bool {
    truthy(&board["migrations"], "testRuns")
}

/// The live test plans, in id order.
fn plans_of(board: &Board) -> Vec<String> {
    ids(board, "items")
        .into_iter()
        .filter(|id| {
            doc(board, "items", id).is_some_and(|p| p["type"] == PLAN && !truthy(p, "deletedAt"))
        })
        .collect()
}

/// The runs a plan has on the board, trash included.
fn runs_of(board: &Board, plan: &str) -> Vec<Value> {
    let runs = section(board, "items").values();
    let own = runs.filter(|i| i["type"] == RUN && i["fields"]["plan"].as_str() == Some(plan));
    own.cloned().collect()
}

/// The number the next run of the plan gets.
fn next_n(board: &Board, plan: &str) -> i64 {
    let highest = runs_of(board, plan)
        .iter()
        .filter_map(|r| r["n"].as_i64())
        .max();
    highest.unwrap_or(0) + 1
}

/// Whether a run of the plan already carries the legacy key.
fn has_legacy(board: &Board, plan: &str, (key, value): (&str, &str)) -> bool {
    runs_of(board, plan)
        .iter()
        .any(|r| r["fields"][key].as_str() == Some(value))
}

/// Migrate the board once: plan runs first, then bundle tags, then the mark.
///
/// # Errors
/// A doc the board cannot create.
pub fn migrate_runs(board: &mut Board, actor: &str) -> Result<Report, BoardError> {
    let mut report = Report::default();
    if is_migrated(board) {
        report.already = true;
        return Ok(report);
    }
    plan_runs::migrate(board, actor, &mut report)?;
    bundles::migrate(board, actor, &mut report)?;
    let mark = json!({"at": now_iso()});
    map_mut(board, "migrations").insert("testRuns".to_owned(), mark);
    Ok(report)
}
