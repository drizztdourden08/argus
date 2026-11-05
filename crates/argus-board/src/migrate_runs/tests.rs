use serde_json::{Value, json};

use super::{is_migrated, migrate_runs};
use crate::Board;
use crate::store::empty_board;

fn board() -> Board {
    let mut b = empty_board("p");
    b["items"] = json!({
        "I-1": {"id": "I-1", "type": "test-plan", "title": "Login", "thread": "argus", "stage": "reported",
            "fields": {"bundle": "b1", "cases": ["T-1"], "runs": [
                {"n": 1, "bundle": "b1", "by": "argus", "startedAt": "2026-01-01T00:00:00Z",
                 "endedAt": "2026-01-02T00:00:00Z",
                 "results": [{"test": "T-1", "status": "pass", "note": "", "at": "x", "proofs": []}],
                 "summary": {"pass": 1, "fail": 0, "partial": 0, "blocked": 0, "skip": 0, "notRun": 0, "total": 1}}
            ]}},
    });
    b["tests"] = json!({
        "T-1": {"id": "T-1", "title": "a", "build": "b1", "result": {"status": "fail", "text": "bad", "by": "x", "at": "t"}},
        "T-2": {"id": "T-2", "title": "b", "build": "b2", "result": {"status": "pass", "text": "ok", "by": "x", "at": "t"}},
        "T-3": {"id": "T-3", "title": "c", "build": "b3"},
    });
    b["builds"] = json!({
        "b1": {"tag": "b1", "status": "ready", "launcher": "l1"},
        "b2": {"tag": "b2", "status": "ready", "closedAt": "2026-02-01T00:00:00Z"},
    });
    b["launchers"] = json!({"l1": {"name": "l1"}});
    b["handedBuild"] = json!("b3");
    b
}

fn runs(b: &Board) -> Vec<Value> {
    let all = b["items"].as_object().unwrap().values();
    all.filter(|i| i["type"] == "test-run").cloned().collect()
}

fn legacy_run(b: &Board, key: &str) -> Value {
    let found = runs(b).into_iter().find(|r| r["fields"]["legacyRun"] == key);
    found.unwrap()
}

fn bundle_run(b: &Board, tag: &str) -> Value {
    let found = runs(b)
        .into_iter()
        .find(|r| r["fields"]["legacyBundle"] == tag && r["fields"]["legacyRun"].is_null());
    found.unwrap()
}

#[test]
fn a_plan_run_keeps_its_numbers_and_results() {
    let mut b = board();
    migrate_runs(&mut b, "overlord").unwrap();
    let run = legacy_run(&b, "I-1#1");
    assert_eq!(run["n"], 1);
    assert_eq!(run["stage"], "passed");
    assert_eq!(run["status"], "done");
    assert_eq!(run["fields"]["plan"], "I-1");
    assert_eq!(run["fields"]["build"], "l1");
    assert_eq!(run["fields"]["legacyBundle"], "b1");
    assert_eq!(run["fields"]["results"][0]["test"], "T-1");
    assert_eq!(run["endedAt"], "2026-01-02T00:00:00Z");
}

#[test]
fn a_bundle_a_plan_run_covers_makes_no_second_run() {
    let mut b = board();
    migrate_runs(&mut b, "overlord").unwrap();
    let of_plan = runs(&b).into_iter().filter(|r| r["fields"]["plan"] == "I-1");
    assert_eq!(of_plan.count(), 1);
}

#[test]
fn a_closed_bundle_without_a_plan_gets_a_plan_and_a_run() {
    let mut b = board();
    migrate_runs(&mut b, "overlord").unwrap();
    let run = bundle_run(&b, "b2");
    assert_eq!(run["stage"], "passed");
    let plan = &b["items"][run["fields"]["plan"].as_str().unwrap()];
    assert_eq!(plan["type"], "test-plan");
    assert_eq!(plan["fields"]["cases"], json!(["T-2"]));
    assert_eq!(run["fields"]["results"][0]["status"], "pass");
}

#[test]
fn the_handed_bundle_is_ready_for_the_owner() {
    let mut b = board();
    migrate_runs(&mut b, "overlord").unwrap();
    let run = bundle_run(&b, "b3");
    assert_eq!(run["stage"], "ready");
    assert_eq!(run["assignee"], "owner");
    assert_eq!(run["status"], "todo");
}

#[test]
fn an_open_bundle_is_ready_only_with_a_live_build() {
    let mut b = board();
    b["handedBuild"] = Value::Null;
    b["builds"]["b3"] = json!({"tag": "b3", "status": "draft", "launcher": "l1"});
    migrate_runs(&mut b, "overlord").unwrap();
    assert_eq!(bundle_run(&b, "b3")["stage"], "ready");
    let mut c = board();
    c["handedBuild"] = Value::Null;
    migrate_runs(&mut c, "overlord").unwrap();
    assert_eq!(bundle_run(&c, "b3")["stage"], "draft");
}

#[test]
fn legacy_fields_stay_and_the_board_is_marked() {
    let mut b = board();
    let before = b.clone();
    let report = migrate_runs(&mut b, "overlord").unwrap();
    assert_eq!(b["items"]["I-1"]["fields"]["runs"], before["items"]["I-1"]["fields"]["runs"]);
    assert_eq!(b["items"]["I-1"]["fields"]["bundle"], "b1");
    assert_eq!(b["builds"], before["builds"]);
    assert_eq!(b["tests"], before["tests"]);
    assert_eq!(b["handedBuild"], "b3");
    assert!(is_migrated(&b));
    assert_eq!((report.from_plan_runs, report.from_bundles, report.plans), (1, 2, 2));
}

#[test]
fn a_second_run_changes_nothing() {
    let mut b = board();
    migrate_runs(&mut b, "overlord").unwrap();
    let once = b.clone();
    let report = migrate_runs(&mut b, "overlord").unwrap();
    assert!(report.already);
    assert_eq!(b, once);
}

#[test]
fn a_tag_with_no_test_and_no_plan_is_skipped() {
    let mut b = empty_board("p");
    b["builds"] = json!({"lonely": {"tag": "lonely", "status": "draft"}});
    let report = migrate_runs(&mut b, "overlord").unwrap();
    assert!(runs(&b).is_empty());
    assert_eq!(report.from_bundles, 0);
    assert!(is_migrated(&b));
}
