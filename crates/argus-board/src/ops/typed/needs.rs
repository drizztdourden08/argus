//! The conditions a stage gate or a done rule names, each answered with what is still missing.

use serde_json::Value;

use super::tree::children_of;
use crate::Board;
use crate::json::{display, doc, section, truthy};
use crate::launchers::bundle_status;
use crate::ops::Who;

/// What a condition is asked about: the board, the item as it will be, and who acts.
pub(crate) struct Subject<'a> {
    pub board: &'a Board,
    pub item: &'a Value,
    pub who: &'a Who,
}

/// What is missing for `need` to hold, one sentence each; nothing when it holds. An unknown name
/// holds.
pub(crate) fn missing(need: &str, s: &Subject) -> Vec<String> {
    match need {
        "ac" => open_criteria(s.item),
        "children" => open_children(s),
        "tests" => unpassed_tests(s),
        "verified" => unverified(s),
        "plan-passed" => plan_not_passed(s.item),
        "cases" => no_cases(s.item),
        "bundle-ready" => bundle_not_ready(s),
        _ => Vec::new(),
    }
}

fn listed(what: &str, ids: &[String]) -> Vec<String> {
    if ids.is_empty() {
        Vec::new()
    } else {
        vec![format!("{what}: {}", ids.join(", "))]
    }
}

fn id_of(item: &Value) -> &str {
    item["id"].as_str().unwrap_or_default()
}

fn open_criteria(item: &Value) -> Vec<String> {
    let open: Vec<String> = item["ac"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["done"] != true)
        .map(|c| format!("AC{}", display(&c["n"])))
        .collect();
    listed("criteria still open", &open)
}

fn is_settled(item: &Value) -> bool {
    matches!(item["status"].as_str(), Some("done" | "dropped"))
}

fn open_children(s: &Subject) -> Vec<String> {
    let open: Vec<String> = children_of(s.board, id_of(s.item))
        .into_iter()
        .filter(|c| !doc(s.board, "items", c).is_some_and(is_settled))
        .collect();
    listed("children not done or dropped", &open)
}

/// The live tests that verify an item, with their status.
fn linked_tests(board: &Board, id: &str) -> Vec<(String, String)> {
    section(board, "tests")
        .iter()
        .filter(|(_, t)| t["item"].as_str() == Some(id))
        .filter(|(_, t)| !truthy(t, "deletedAt") && !truthy(t, "archivedAt"))
        .map(|(tid, t)| {
            let status = t["status"].as_str().unwrap_or("pending");
            (tid.clone(), status.to_owned())
        })
        .collect()
}

fn counts_as_passed(status: &str) -> bool {
    matches!(status, "pass" | "moved")
}

fn not_passed(tests: &[(String, String)]) -> Vec<String> {
    tests
        .iter()
        .filter(|(_, status)| !counts_as_passed(status))
        .map(|(id, _)| id.clone())
        .collect()
}

fn unpassed_tests(s: &Subject) -> Vec<String> {
    let tests = linked_tests(s.board, id_of(s.item));
    listed("tests not passed", &not_passed(&tests))
}

fn reporter_is_owner(item: &Value) -> bool {
    match item["fields"]["reporter"].as_str() {
        Some(reporter) => reporter == "owner",
        None => item["createdBy"] == "human",
    }
}

fn unverified(s: &Subject) -> Vec<String> {
    if s.who.is_human() && reporter_is_owner(s.item) {
        return Vec::new();
    }
    let tests = linked_tests(s.board, id_of(s.item));
    if tests.is_empty() {
        return vec!["no test case is linked to it yet".to_owned()];
    }
    let failing = not_passed(&tests);
    if !failing.is_empty() {
        return listed("tests not passed", &failing);
    }
    if tests.iter().any(|(_, status)| status == "pass") {
        Vec::new()
    } else {
        vec!["no linked test has passed".to_owned()]
    }
}

fn cases_of(item: &Value) -> Vec<String> {
    item["fields"]["cases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c.as_str().map(str::to_owned))
        .collect()
}

fn case_passed(run: &Value, case: &str) -> bool {
    let results = run["results"].as_array().cloned().unwrap_or_default();
    results
        .iter()
        .rev()
        .find(|r| r["test"].as_str() == Some(case))
        .is_some_and(|r| r["status"] == "pass")
}

fn plan_not_passed(item: &Value) -> Vec<String> {
    let runs = item["fields"]["runs"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let Some(run) = runs.last() else {
        return vec!["the plan has no run yet".to_owned()];
    };
    let n = display(&run["n"]);
    if run["endedAt"].is_null() {
        return vec![format!("run {n} is still open")];
    }
    let failed: Vec<String> = cases_of(item)
        .into_iter()
        .filter(|c| !case_passed(run, c))
        .collect();
    listed(&format!("cases not passed in run {n}"), &failed)
}

fn no_cases(item: &Value) -> Vec<String> {
    if cases_of(item).is_empty() {
        vec!["it has no test case yet (add one with coord test plan cases)".to_owned()]
    } else {
        Vec::new()
    }
}

fn bundle_not_ready(s: &Subject) -> Vec<String> {
    let Some(tag) = s.item["fields"]["bundle"].as_str() else {
        return vec!["no bundle is set".to_owned()];
    };
    match doc(s.board, "builds", tag) {
        Some(rec) if bundle_status(s.board, tag, rec) == "ready" => Vec::new(),
        _ => vec![format!("bundle {tag} is not ready")],
    }
}
