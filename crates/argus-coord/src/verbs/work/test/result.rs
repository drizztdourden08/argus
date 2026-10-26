//! `coord test result <T> <pass|fail|partial|blocked|moved> "<note>" [--proof <image> --caption
//! "..."]` and `coord test move <T> --item <I2> [--ac N]...`.

use argus_board::json::{display, doc_mut};
use argus_board::limits::require_text;
use argus_board::ops::{Check, authorize};
use argus_board::results::{Outcome as Recorded, record_result};
use argus_board::store::stamp;
use serde_json::{Value, json};

use super::proof::keep_proof;
use super::spec::ac_numbers;
use crate::cli::{Outcome, Val};
use crate::verbs::work::crud::{apply, checked, find_doc, find_live, move_op, who_of};
use crate::verbs::work::run::{Run, bad};

const RESULTS: [&str; 5] = ["pass", "fail", "partial", "blocked", "moved"];

fn attach_proofs(r: &Run<'_>, board: &mut Value, id: &str) -> crate::verbs::work::run::Res<()> {
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
        keep_proof(r, test, (&file.to_json(), caption))?;
    }
    Ok(())
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
    if !status.as_str().is_some_and(|s| RESULTS.contains(&s)) {
        return Err(bad(format!("result must be one of {}.", RESULTS.join(", "))).into());
    }
    let note = require_text("result", &r.arg_json(2))?;
    r.write(|board, r| {
        let who = r.begin(board)?;
        let test = checked(board, &who, ("tests", "result"), r.arg(0))?;
        let id = display(&test["id"]);
        attach_proofs(r, board, &id)?;
        let outcome = Recorded {
            status: &status,
            text: &note,
            by: &who.actor,
        };
        record_result(board, &id, &outcome)?;
        let after = find_doc(board, "tests", Some(&id))?;
        let said = if status == "pass" {
            ticked(&after)
        } else {
            String::new()
        };
        r.say(&[&format!("{id}: {}{said}", display(&status))]);
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
