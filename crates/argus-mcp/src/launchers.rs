//! `list_launchers`: the launchers as the Builds tab shows them. A row is the launcher record, so
//! it carries `for` (the item the build serves), `master`, `runs` (its test runs),
//! `retiredByItem` (the item whose closing retired it) and `relink` (a build waiting for `coord launcher link`) when they are set.

use std::cmp::Ordering;
use std::collections::HashMap;

use serde_json::{Value, json};

use crate::args::flag;
use crate::backend::McpBackend;
use crate::json::{display, items, nn, text, truthy};
use argus_core::collate;

/// The master build: the launcher carrying the `master` flag, or the one named `master` while no
/// launcher carries it.
fn is_master(l: &Value, flagged: bool) -> bool {
    if flagged {
        truthy(l, "master")
    } else {
        text(l, "name") == Some("master")
    }
}

fn pin(x: &Value, y: &Value, flagged: bool) -> Ordering {
    is_master(y, flagged)
        .cmp(&is_master(x, flagged))
        .then_with(|| truthy(y, "current").cmp(&truthy(x, "current")))
        .then_with(|| collate::compare(&display(x, "name"), &display(y, "name"), true))
}

fn plays(backend: &dyn McpBackend, key: &str) -> HashMap<String, Value> {
    backend
        .docs(key, "launches")
        .iter()
        .filter(|l| truthy(l, "launcher"))
        .filter_map(|l| Some((text(l, "launcher")?.to_owned(), nn(l, "name")?.clone())))
        .collect()
}

fn is_run(item: &Value) -> bool {
    text(item, "type") == Some("test-run") && !truthy(item, "deletedAt")
}

fn case_ids(plan: Option<&Value>) -> Vec<String> {
    plan.map(|p| items(&p["fields"], "cases"))
        .unwrap_or_default()
        .iter()
        .filter_map(|c| c.as_str().map(str::to_owned))
        .collect()
}

/// The status each case last got in the run, keyed by test id.
fn latest(run: &Value) -> HashMap<&str, &str> {
    items(&run["fields"], "results")
        .iter()
        .filter_map(|r| Some((text(r, "test")?, text(r, "status")?)))
        .collect()
}

fn tally(run: &Value, cases: &[String]) -> (usize, usize, usize) {
    let got = latest(run);
    let count = |status: &str| got.values().filter(|s| **s == status).count();
    let to_run = cases
        .iter()
        .filter(|c| matches!(got.get(c.as_str()), None | Some(&"pending")))
        .count();
    (count("pass"), count("fail"), to_run)
}

fn run_row(run: &Value, plans: &HashMap<&str, &Value>) -> Value {
    let plan = text(&run["fields"], "plan");
    let cases = case_ids(plan.and_then(|p| plans.get(p).copied()));
    let (pass, fail, to_run) = tally(run, &cases);
    json!({
        "id": run["id"], "title": run["title"], "plan": plan, "build": nn(&run["fields"], "build"),
        "stage": run["stage"], "pass": pass, "fail": fail, "toRun": to_run,
    })
}

/// The test runs of a launcher: the runs whose build names it, or the run the launcher is for.
fn runs_of(runs: &[Value], plans: &HashMap<&str, &Value>, launcher: &Value) -> Vec<Value> {
    let name = text(launcher, "name");
    let serves = text(launcher, "for");
    runs.iter()
        .filter(|r| {
            (name.is_some() && text(&r["fields"], "build") == name) || text(r, "id") == serves
        })
        .map(|r| run_row(r, plans))
        .collect()
}

fn launcher_row(l: &Value, plays: &HashMap<String, Value>, runs: Vec<Value>) -> Value {
    let play = if truthy(l, "retiredAt") {
        Value::Null
    } else {
        text(l, "name")
            .and_then(|n| plays.get(n))
            .cloned()
            .unwrap_or(Value::Null)
    };
    let mut row = l.clone();
    if let Some(map) = row.as_object_mut() {
        map.insert("play".into(), play);
        map.insert("runs".into(), Value::Array(runs));
    }
    row
}

/// The launchers, master first, then the current ones, with their Play run name and test runs.
pub fn list_launchers(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let retired = flag(args, "includeRetired");
    let play_runs = plays(backend, key);
    let docs = backend.docs(key, "items");
    let plans: HashMap<&str, &Value> = docs
        .iter()
        .filter(|i| text(i, "type") == Some("test-plan"))
        .filter_map(|i| Some((text(i, "id")?, i)))
        .collect();
    let runs: Vec<Value> = docs.iter().filter(|i| is_run(i)).cloned().collect();
    let all = backend.docs(key, "launchers");
    let flagged = all.iter().any(|l| truthy(l, "master"));
    let mut rows: Vec<Value> = all
        .into_iter()
        .filter(|l| retired || !truthy(l, "retiredAt"))
        .collect();
    rows.sort_by(|x, y| pin(x, y, flagged));
    let master = rows
        .iter()
        .any(|l| is_master(l, flagged) && !truthy(l, "retiredAt"));
    let launchers: Vec<Value> = rows
        .iter()
        .map(|l| launcher_row(l, &play_runs, runs_of(&runs, &plans, l)))
        .collect();
    json!({ "launchers": launchers, "master": master })
}

#[cfg(test)]
mod tests;
