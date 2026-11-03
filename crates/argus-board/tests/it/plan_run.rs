//! The runs of a test plan: a run is kept on the plan item, each result is also written to its
//! test, a skip stays in the run, one run is open at a time and ending it counts the results.
//! I-70 P3b.

// test-tag: critical

use crate::common::{alpha, applied, doc, lead, refused};
use argus_board::ops::{OpResult, Role, Who, apply_op};
use serde_json::{Value, json};

fn stranger() -> Who {
    Who {
        thread: Some("other".to_owned()),
        role: Role::Thread,
        actor: None,
    }
}

fn run_op(board: &mut Value, who: &Who, kind: &str, plan: &str, rest: Value) -> OpResult {
    let mut op = json!({"kind": kind, "target": {"c": "items", "id": plan}});
    if let Some(extra) = rest.as_object() {
        for (key, value) in extra {
            op[key.as_str()] = value.clone();
        }
    }
    apply_op(board, &op, who, None)
}

fn record(board: &mut Value, plan: &str, test: &str, status: &str, note: &str) -> OpResult {
    let rest = json!({"test": test, "status": status, "note": note});
    run_op(board, &lead(), "plan-run-record", plan, rest)
}

/// The alpha board with a third test and a plan of the three tests, owned by `ui`.
fn plan_board() -> (Value, String) {
    let mut b = alpha();
    b["builds"]["b1"] = json!({"tag": "b1"});
    b["tests"]["T-3"] = doc(
        "T-3",
        "ui",
        json!({
            "item": "I-1", "thread": "ui", "title": "Third", "mode": "manual",
            "status": "pending", "ac": [], "proofs": [], "runs": [],
        }),
        3072,
    );
    let set = json!({
        "title": "Smoke", "thread": "ui", "type": "test-plan",
        "fields": {"cases": ["T-1", "T-2", "T-3"]},
    });
    let op = json!({"kind": "create", "target": {"c": "items"}, "set": set});
    let res = apply_op(&mut b, &op, &lead(), None);
    applied(&res);
    let id = res.extra["created"]["id"].as_str().unwrap_or_default();
    (b, id.to_owned())
}

fn run_of(b: &Value, plan: &str, at: usize) -> Value {
    b["items"][plan]["fields"]["runs"][at].clone()
}

#[test]
fn a_run_starts_once_and_moves_the_plan_to_running() {
    let (mut b, plan) = plan_board();
    let start = run_op(
        &mut b,
        &lead(),
        "plan-run-start",
        &plan,
        json!({"bundle": "b1"}),
    );
    applied(&start);
    assert_eq!(start.outcome, format!("{plan} run 1 started"));
    let run = run_of(&b, &plan, 0);
    assert_eq!(run["n"], 1);
    assert_eq!(run["bundle"], "b1");
    assert_eq!(run["by"], "coordinator");
    assert_eq!(run["endedAt"], Value::Null);
    assert_eq!(b["items"][plan.as_str()]["stage"], "running");
    assert_eq!(b["items"][plan.as_str()]["status"], "doing");
    let again = run_op(&mut b, &lead(), "plan-run-start", &plan, json!({}));
    assert!(refused(&again).contains("already has run 1 open"));
    applied(&run_op(&mut b, &lead(), "plan-run-end", &plan, json!({})));
    let bad = run_op(
        &mut b,
        &lead(),
        "plan-run-start",
        &plan,
        json!({"bundle": "nope"}),
    );
    assert!(refused(&bad).contains("not on this board"));
}

#[test]
fn a_plan_without_cases_or_a_stranger_cannot_start_a_run() {
    let (mut b, plan) = plan_board();
    let denied = run_op(&mut b, &stranger(), "plan-run-start", &plan, json!({}));
    assert!(refused(&denied).contains("belongs to ui"));
    let op = json!({"kind": "create", "target": {"c": "items"}, "set": {"title": "Empty", "thread": "ui", "type": "test-plan"}});
    let empty = apply_op(&mut b, &op, &lead(), None);
    applied(&empty);
    let id = empty.extra["created"]["id"].as_str().unwrap_or_default();
    let none = run_op(&mut b, &lead(), "plan-run-start", id, json!({}));
    assert!(refused(&none).contains("has no test case yet"));
    let task = run_op(&mut b, &lead(), "plan-run-start", "I-1", json!({}));
    assert_eq!(refused(&task), "I-1 is not a test plan.");
}

#[test]
fn a_result_in_a_run_is_also_written_to_its_test() {
    let (mut b, plan) = plan_board();
    applied(&run_op(&mut b, &lead(), "plan-run-start", &plan, json!({})));
    let fail = record(&mut b, &plan, "T-3", "fail", "the join hangs");
    applied(&fail);
    assert_eq!(
        fail.outcome,
        format!("T-3: fail in run 1 of {plan} (1 of 3 cases)")
    );
    assert_eq!(b["tests"]["T-3"]["status"], "fail");
    assert_eq!(b["tests"]["T-3"]["result"]["text"], "the join hangs");
    assert_eq!(b["tests"]["T-3"]["result"]["by"], "coordinator");
    assert_eq!(b["tests"]["T-3"]["runs"].as_array().map_or(0, Vec::len), 1);
    let again = record(&mut b, &plan, "T-3", "pass", "");
    applied(&again);
    assert_eq!(b["tests"]["T-3"]["status"], "pass");
    assert_eq!(
        b["tests"]["T-3"]["result"]["text"],
        format!("pass in run 1 of {plan}")
    );
    let results = run_of(&b, &plan, 0)["results"].clone();
    assert_eq!(results.as_array().map_or(0, Vec::len), 2);
    assert_eq!(results[0]["status"], "fail");
    assert_eq!(results[0]["note"], "the join hangs");
    assert_eq!(results[1]["status"], "pass");
}

#[test]
fn a_skip_stays_in_the_run_and_writes_nothing_to_the_test() {
    let (mut b, plan) = plan_board();
    applied(&run_op(&mut b, &lead(), "plan-run-start", &plan, json!({})));
    let before = b["tests"]["T-2"].clone();
    applied(&record(&mut b, &plan, "T-2", "skip", ""));
    assert_eq!(b["tests"]["T-2"], before);
    assert_eq!(run_of(&b, &plan, 0)["results"][0]["status"], "skip");
}

#[test]
fn a_result_needs_an_open_run_a_case_a_status_and_a_note_when_it_is_not_a_pass() {
    let (mut b, plan) = plan_board();
    let early = record(&mut b, &plan, "T-1", "pass", "");
    assert!(refused(&early).contains("has no open run"));
    applied(&run_op(&mut b, &lead(), "plan-run-start", &plan, json!({})));
    b["tests"]["T-4"] = doc(
        "T-4",
        "ui",
        json!({"item": "I-1", "thread": "ui", "title": "Other", "status": "pending"}),
        4096,
    );
    let foreign = record(&mut b, &plan, "T-4", "pass", "");
    assert!(refused(&foreign).contains("T-4 is not a case of"));
    let word = record(&mut b, &plan, "T-1", "moved", "x");
    assert!(refused(&word).contains("status must be one of"));
    for status in ["fail", "partial", "blocked"] {
        let bare = record(&mut b, &plan, "T-1", status, "");
        assert!(
            refused(&bare).contains("result text is required"),
            "{status}"
        );
    }
    let rest = json!({"test": "T-1", "status": "pass", "proofs": [7]});
    let proof = run_op(&mut b, &lead(), "plan-run-record", &plan, rest);
    assert!(refused(&proof).contains("T-1 has no proof 7"));
    let denied = run_op(
        &mut b,
        &stranger(),
        "plan-run-record",
        &plan,
        json!({"test": "T-1", "status": "pass"}),
    );
    assert!(refused(&denied).contains("belongs to ui"));
    assert_eq!(
        run_of(&b, &plan, 0)["results"]
            .as_array()
            .map_or(0, Vec::len),
        0
    );
}

#[test]
fn ending_a_run_counts_the_last_result_of_each_case_and_reports_the_plan() {
    let (mut b, plan) = plan_board();
    applied(&run_op(&mut b, &lead(), "plan-run-start", &plan, json!({})));
    applied(&record(&mut b, &plan, "T-1", "fail", "first try"));
    applied(&record(&mut b, &plan, "T-1", "pass", "second try"));
    applied(&record(&mut b, &plan, "T-2", "blocked", "no device"));
    let end = run_op(&mut b, &lead(), "plan-run-end", &plan, json!({}));
    applied(&end);
    let summary = run_of(&b, &plan, 0)["summary"].clone();
    assert_eq!(
        summary,
        json!({"pass": 1, "fail": 0, "partial": 0, "blocked": 1, "skip": 0, "notRun": 1, "total": 3})
    );
    assert_eq!(end.extra["summary"], summary);
    assert_ne!(run_of(&b, &plan, 0)["endedAt"], Value::Null);
    assert_eq!(b["items"][plan.as_str()]["stage"], "reported");
    assert_eq!(b["items"][plan.as_str()]["status"], "review");
    let late = record(&mut b, &plan, "T-3", "pass", "");
    assert!(refused(&late).contains("has no open run"));
    let second = run_op(&mut b, &lead(), "plan-run-start", &plan, json!({}));
    applied(&second);
    assert_eq!(second.outcome, format!("{plan} run 2 started"));
}

#[test]
fn a_run_that_passed_every_case_lets_the_owner_pass_the_plan() {
    let (mut b, plan) = plan_board();
    applied(&run_op(&mut b, &lead(), "plan-run-start", &plan, json!({})));
    for test in ["T-1", "T-2", "T-3"] {
        applied(&record(&mut b, &plan, test, "pass", ""));
    }
    applied(&run_op(&mut b, &lead(), "plan-run-end", &plan, json!({})));
    let edit =
        json!({"kind": "edit", "target": {"c": "items", "id": plan}, "set": {"stage": "passed"}});
    applied(&apply_op(&mut b, &edit, &Who::human(), None));
    assert_eq!(b["items"][plan.as_str()]["status"], "done");
}
