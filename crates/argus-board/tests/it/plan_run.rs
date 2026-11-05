//! The runs of a test plan: a run is an item of its own that links its plan, each result is also
//! written to its test, a skip stays in the run, one run is open per plan and ending it counts
//! the results. I-70 P3b, I-99.

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
    if let Value::Object(extra) = rest {
        for (key, value) in extra {
            op[key.as_str()] = value;
        }
    }
    apply_op(board, &op, who, None)
}

fn record(board: &mut Value, plan: &str, test: &str, status: &str, note: &str) -> OpResult {
    let extra = json!({"test": test, "status": status, "note": note});
    run_op(board, &lead(), "plan-run-record", plan, extra)
}

/// The alpha board with a third test and a plan of the three tests, owned by `ui`.
fn plan_board() -> (Value, String) {
    let mut b = alpha();
    b["launchers"]["l1"] = json!({"name": "l1"});
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

/// The runs of a plan, oldest first.
fn runs_of(b: &Value, plan: &str) -> Vec<Value> {
    let mut runs: Vec<Value> = b["items"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(_, item)| item.clone())
        .filter(|i| i["type"] == "test-run" && i["fields"]["plan"] == plan)
        .collect();
    runs.sort_by_key(|r| r["n"].as_i64().unwrap_or_default());
    runs
}

fn run_of(b: &Value, plan: &str, at: usize) -> Value {
    runs_of(b, plan)[at].clone()
}

fn results_of(run: &Value) -> Vec<Value> {
    run["fields"]["results"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[test]
fn a_run_starts_once_and_moves_the_plan_to_running() {
    let (mut b, plan) = plan_board();
    let start = run_op(
        &mut b,
        &lead(),
        "plan-run-start",
        &plan,
        json!({"build": "l1"}),
    );
    applied(&start);
    assert_eq!(start.outcome, format!("{plan} run 1 started"));
    let run = run_of(&b, &plan, 0);
    assert_eq!(run["n"], 1);
    assert_eq!(run["fields"]["build"], "l1");
    assert_eq!(run["fields"]["plan"], plan.as_str());
    assert_eq!(run["createdBy"], "coordinator");
    assert_eq!(run["stage"], "running");
    assert_eq!(run["status"], "doing");
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
        json!({"build": "nope"}),
    );
    assert!(refused(&bad).contains("not on this board"));
}

#[test]
fn a_stranger_or_a_task_cannot_start_a_run() {
    let (mut b, plan) = plan_board();
    let denied = run_op(&mut b, &stranger(), "plan-run-start", &plan, json!({}));
    assert!(refused(&denied).contains("belongs to ui"));
    let task = run_op(&mut b, &lead(), "plan-run-start", "I-1", json!({}));
    assert_eq!(refused(&task), "I-1 is not a test plan.");
    assert_eq!(runs_of(&b, &plan).len(), 0);
}

#[test]
fn a_plan_has_one_open_run_and_a_ready_run_is_the_one_that_starts() {
    let (mut b, plan) = plan_board();
    let ready = doc(
        "I-90",
        "ui",
        json!({
            "type": "test-run", "stage": "ready", "status": "todo", "n": 1, "title": "Run 1",
            "endedAt": null, "fields": {"plan": plan, "build": "l1"},
        }),
        9000,
    );
    b["items"]["I-90"] = ready;
    let start = run_op(&mut b, &lead(), "plan-run-start", &plan, json!({}));
    applied(&start);
    assert_eq!(start.outcome, format!("{plan} run 1 started"));
    assert_eq!(runs_of(&b, &plan).len(), 1);
    assert_eq!(b["items"]["I-90"]["stage"], "running");
    assert_eq!(b["items"][plan.as_str()]["stage"], "running");
    let second = run_op(&mut b, &lead(), "plan-run-start", &plan, json!({}));
    assert!(refused(&second).contains("already has run 1 open"));
    assert_eq!(runs_of(&b, &plan).len(), 1);
    let on_run = run_op(
        &mut b,
        &lead(),
        "plan-run-record",
        "I-90",
        json!({"test": "T-1", "status": "pass"}),
    );
    applied(&on_run);
    assert_eq!(results_of(&b["items"]["I-90"]).len(), 1);
}

#[test]
fn a_plan_keeps_at_least_one_case() {
    let (mut b, plan) = plan_board();
    let set = json!({"title": "Empty", "thread": "ui", "type": "test-plan"});
    let op = json!({"kind": "create", "target": {"c": "items"}, "set": set});
    let empty = apply_op(&mut b, &op, &lead(), None);
    assert!(refused(&empty).contains("keeps at least one case"));
    let op =
        json!({"kind": "edit", "target": {"c": "items", "id": plan}, "set": {"fields.cases": []}});
    let emptied = apply_op(&mut b, &op, &lead(), None);
    assert!(refused(&emptied).contains("keeps at least one case"));
    let fields = &b["items"][plan.as_str()]["fields"];
    assert_eq!(fields["cases"], json!(["T-1", "T-2", "T-3"]));
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
    let results = Value::Array(results_of(&run_of(&b, &plan, 0)));
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
    assert_eq!(results_of(&run_of(&b, &plan, 0))[0]["status"], "skip");
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
    assert_eq!(results_of(&run_of(&b, &plan, 0)).len(), 0);
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
    let summary = json!({"pass": 1, "fail": 0, "partial": 0, "blocked": 1, "skip": 0, "notRun": 1, "total": 3});
    assert_eq!(end.extra["summary"], summary);
    let run = run_of(&b, &plan, 0);
    assert_eq!(
        run["fields"]["summary"],
        "1 pass, 0 fail, 0 partial, 1 blocked, 0 skip, 1 not run"
    );
    assert_ne!(run["endedAt"], Value::Null);
    assert_eq!(run["stage"], "reported");
    assert_eq!(run["status"], "review");
    assert_eq!(b["items"][plan.as_str()]["stage"], "reported");
    assert_eq!(b["items"][plan.as_str()]["status"], "review");
    let late = record(&mut b, &plan, "T-3", "pass", "");
    assert!(refused(&late).contains("has no open run"));
    let second = run_op(&mut b, &lead(), "plan-run-start", &plan, json!({}));
    applied(&second);
    assert_eq!(second.outcome, format!("{plan} run 2 started"));
    assert_eq!(runs_of(&b, &plan).len(), 2);
}

#[test]
fn a_run_that_passed_every_case_lets_the_owner_pass_the_plan() {
    let (mut b, plan) = plan_board();
    applied(&run_op(&mut b, &lead(), "plan-run-start", &plan, json!({})));
    for test in ["T-1", "T-2", "T-3"] {
        applied(&record(&mut b, &plan, test, "pass", ""));
    }
    applied(&run_op(&mut b, &lead(), "plan-run-end", &plan, json!({})));
    assert_eq!(run_of(&b, &plan, 0)["stage"], "passed");
    assert_eq!(run_of(&b, &plan, 0)["status"], "done");
    let edit =
        json!({"kind": "edit", "target": {"c": "items", "id": plan}, "set": {"stage": "passed"}});
    applied(&apply_op(&mut b, &edit, &Who::human(), None));
    assert_eq!(b["items"][plan.as_str()]["status"], "done");
}
