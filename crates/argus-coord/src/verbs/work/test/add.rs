//! `coord test add <item> "<title>" [--mode auto|manual] [--build <tag>] [--file spec.json] [flags]`
//! and `coord test set <T> [--file spec.json] [--covers --run --context --changed --note --ac
//! --mode --title] [--build <tag> | --build "" | --clear-build]`.

use argus_board::json::{display, doc, map_mut};
use argus_board::ops::{Check, authorize};
use argus_board::store::create_doc;
use serde_json::{Value, json};

use super::edit::edit;
use super::spec::{apply_spec, spec_from_options};
use crate::cli::Outcome;
use crate::verbs::work::crud::{find_live, who_of};
use crate::verbs::work::run::{Run, bad};

fn blank_test(item: &Value) -> serde_json::Map<String, Value> {
    let fields = json!({
        "item": item["id"], "thread": item["thread"], "subject": item["subject"],
        "title": null, "covers": null, "build": null, "mode": "auto", "run": null,
        "context": null, "changed": [], "setup": [], "steps": [], "pass": [], "fail": [],
        "notes": [], "ac": [], "status": "pending", "result": null, "runs": [], "proofs": [],
    });
    fields.as_object().cloned().unwrap_or_default()
}

fn summary(test: &Value, item_id: &str) -> String {
    let build = test["build"].as_str().filter(|b| !b.is_empty());
    let acs: Vec<String> = test["ac"]
        .as_array()
        .map_or_else(Vec::new, |a| a.iter().map(display).collect());
    format!(
        "{}: {} ({}, on {item_id}{}{})",
        display(&test["id"]),
        display(&test["title"]),
        display(&test["mode"]),
        build.map_or_else(String::new, |b| format!(", build {b}")),
        if acs.is_empty() {
            String::new()
        } else {
            format!(", proves AC {}", acs.join(","))
        },
    )
}

/// `coord test add`.
pub(super) fn add(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let item = find_live(board, "items", r.arg(0))?;
        let check = Check::new("add", "tests", None).to(item["thread"].as_str());
        authorize(&who_of(&who), &check)?;
        let id = create_doc(board, "tests", "T", blank_test(&item), &who.actor)?;
        let mut spec = serde_json::Map::new();
        if let Some(title) = r.arg(1) {
            spec.insert("title".into(), json!(title));
        }
        spec.extend(spec_from_options(r)?);
        let mut test = doc(board, "tests", &id).cloned().unwrap_or(Value::Null);
        apply_spec(&mut test, &item, &spec)?;
        if !argus_core::js::truthy(&test["title"]) {
            return Err(bad("title text is required."));
        }
        r.say(&[&summary(&test, &display(&item["id"]))]);
        map_mut(board, "tests").insert(id, test);
        Ok(())
    })
}

/// `coord test set`.
pub(super) fn set_fields(r: &mut Run<'_>) -> Outcome {
    edit(r, |r, test, item| {
        let spec = spec_from_options(r)?;
        if spec.is_empty() {
            return Err(bad(
                "nothing to change: pass --file or flags such as --title --covers --run --context --mode --changed --note --ac --build.",
            ));
        }
        apply_spec(test, item, &spec)?;
        r.say(&[&format!("{}: updated.", display(&test["id"]))]);
        Ok(())
    })
}
