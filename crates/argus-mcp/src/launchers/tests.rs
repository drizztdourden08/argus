// test-tag: critical

use std::collections::HashMap;

use serde_json::{Value, json};

use super::runs_of;

fn run(id: &str, build: Option<&str>, results: &Value) -> Value {
    json!({ "id": id, "type": "test-run", "title": "Run", "stage": "running",
        "fields": { "plan": "I-1", "build": build, "results": results } })
}

fn plan() -> Value {
    json!({ "id": "I-1", "type": "test-plan", "fields": { "cases": ["T-1", "T-2", "T-3"] } })
}

#[test]
fn a_launcher_lists_the_runs_naming_its_build_with_their_counts() {
    let plan = plan();
    let plans = HashMap::from([("I-1", &plan)]);
    let results = json!([
        { "test": "T-1", "status": "fail" },
        { "test": "T-1", "status": "pass" },
        { "test": "T-2", "status": "fail" },
    ]);
    let runs = [
        run("I-2", Some("build-1"), &results),
        run("I-3", Some("other"), &json!([])),
    ];
    let got = runs_of(&runs, &plans, &json!({ "name": "build-1" }));
    assert_eq!(got.len(), 1);
    assert_eq!(
        got[0],
        json!({ "id": "I-2", "title": "Run", "plan": "I-1", "build": "build-1",
            "stage": "running", "pass": 1, "fail": 1, "toRun": 1 })
    );
}

#[test]
fn a_launcher_for_a_run_lists_that_run() {
    let plan = plan();
    let plans = HashMap::from([("I-1", &plan)]);
    let runs = [run("I-4", None, &json!([]))];
    let got = runs_of(&runs, &plans, &json!({ "name": "build-2", "for": "I-4" }));
    assert_eq!(got.len(), 1);
    assert_eq!(got[0]["toRun"], 3);
    assert!(runs_of(&runs, &plans, &json!({ "name": "master" })).is_empty());
}
