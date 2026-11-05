//! `coord test result <T> <pass|fail|partial|blocked|moved> "<note>" [--proof <image> --caption
//! "..."] [--run <plan|run>]` and `coord test move <T> --item <I2> [--ac N]...`. With `--run` the
//! result goes into the open run of that test plan, or the run itself (which also records it on
//! the test), and
//! `skip` is a status too: it stays in the run and writes nothing to the test.

use argus_board::json::{display, doc_mut};
use argus_board::limits::{optional_text, require_text};
use argus_board::ops::{Check, authorize};
use argus_board::results::{Outcome as Recorded, record_result};
use argus_board::store::stamp;
use serde_json::{Value, json};

use super::plan::send;
use super::proof::keep_proof;
use super::spec::ac_numbers;
use crate::cli::{Outcome, Val};
use crate::verbs::work::crud::{apply, checked, find_doc, find_live, move_op, op, who_of};
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

const RESULTS: [&str; 5] = ["pass", "fail", "partial", "blocked", "moved"];

/// Keeps the `--proof` images on the test and gives the numbers they got.
fn attach_proofs(r: &Run<'_>, board: &mut Value, id: &str) -> Res<Vec<i64>> {
    let mut kept = Vec::new();
    let captions = r.options.many("caption");
    for (i, file) in r.options.many("proof").iter().enumerate() {
        let made = format!("{id} proof");
        let caption = match captions.get(i) {
            Some(Val::Text(text)) => Some(text.as_str()),
            Some(Val::Flag) => None,
            None => Some(made.as_str()),
        };
        let Some(test) = doc_mut(board, "tests", id) else {
            continue;
        };
        kept.push(keep_proof(r, test, (&file.to_json(), caption))?);
    }
    Ok(kept)
}

/// The status is one of the five, or `skip` when the result goes into a run.
fn valid(status: &Value, in_run: bool) -> bool {
    status
        .as_str()
        .is_some_and(|s| RESULTS.contains(&s) || (in_run && s == "skip"))
}

/// The test plan or test run an argument names.
fn run_target(board: &Value, id: &str) -> Res<String> {
    let found = find_live(board, "items", Some(id))?;
    if found["type"] != "test-plan" && found["type"] != "test-run" {
        return Err(bad(format!("{id} is not a test plan or a test run.")));
    }
    Ok(display(&found["id"]))
}

/// The result written into the open run of a plan or into a run; the run records it on the test
/// as well.
fn through_run(
    r: &Run<'_>,
    (board, who): (&mut Value, &Caller),
    (plan, test, note, proofs): (&str, &str, &str, &[i64]),
) -> Res<String> {
    let target = run_target(board, plan)?;
    let rest = serde_json::Map::from_iter([
        ("test".to_owned(), json!(test)),
        ("result".to_owned(), r.arg_json(1)),
        ("note".to_owned(), json!(note)),
        ("proofs".to_owned(), json!(proofs)),
    ]);
    let operation = op(("plan-run-record", "items", &target), rest, r.options)?;
    Ok(send(board, who, &operation)?.outcome)
}

fn ticked(test: &Value) -> String {
    let covered: Vec<String> = test["ac"]
        .as_array()
        .map_or_else(Vec::new, |a| a.iter().map(display).collect());
    if covered.is_empty() {
        return String::new();
    }
    format!(
        " (AC {} on {} ticked)",
        covered.join(","),
        display(&test["item"])
    )
}

/// `coord test result`.
pub(super) fn result(r: &mut Run<'_>) -> Outcome {
    let status = r.arg_json(1);
    let plan = r.text_opt("run").map(str::to_owned);
    if !valid(&status, plan.is_some()) {
        return Err(bad(format!("result must be one of {}.", RESULTS.join(", "))).into());
    }
    let note = if status == "skip" {
        optional_text("result", &r.arg_json(2))?.unwrap_or_default()
    } else {
        require_text("result", &r.arg_json(2))?
    };
    r.write(|board, r| {
        let who = r.begin(board)?;
        let test = checked(board, &who, ("tests", "result"), r.arg(0))?;
        let id = display(&test["id"]);
        let proofs = attach_proofs(r, board, &id)?;
        let line = if let Some(plan) = &plan {
            through_run(
                r,
                (&mut *board, &who),
                (plan.as_str(), &id, &note, proofs.as_slice()),
            )?
        } else {
            let outcome = Recorded {
                status: &status,
                text: &note,
                by: &who.actor,
            };
            record_result(board, &id, &outcome)?;
            format!("{id}: {}", display(&status))
        };
        let after = find_doc(board, "tests", Some(&id))?;
        let said = if status == "pass" {
            ticked(&after)
        } else {
            String::new()
        };
        r.say(&[&format!("{line}{said}")]);
        Ok(())
    })
}

/// `coord test move`: the test follows another item; its criteria are cleared unless `--ac`
/// names the new item's.
pub(super) fn move_to(r: &mut Run<'_>) -> Outcome {
    let Some(target) = r.text_opt("item").map(str::to_owned) else {
        return Err(bad("say the item to move it to: coord test move <T> --item <I>.").into());
    };
    r.write(|board, r| {
        let who = r.begin(board)?;
        let test = find_live(board, "tests", r.arg(0))?;
        let item = find_live(board, "items", Some(&target))?;
        let given = Value::Array(r.options.many("ac").iter().map(Val::to_json).collect());
        let acs = ac_numbers(&item, &given)?;
        let check = Check::new("move", "tests", Some(&test)).to(item["thread"].as_str());
        authorize(&who_of(&who), &check)?;
        let (id, item_id) = (display(&test["id"]), display(&item["id"]));
        let to = serde_json::Map::from_iter([("item".to_owned(), json!(item_id))]);
        apply(board, &who, &move_op("tests", &id, to, r.options)?)?;
        if let Some(moved) = doc_mut(board, "tests", &id) {
            argus_board::json::set(moved, "ac", json!(acs));
        }
        stamp(board, "tests", &id, &who.actor)?;
        let proves = if acs.is_empty() {
            String::new()
        } else {
            format!(
                ", proves AC {}",
                acs.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
            )
        };
        r.say(&[&format!(
            "{id}: moved to {item_id} ({}{proves}).",
            display(&item["thread"])
        )]);
        Ok(())
    })
}
