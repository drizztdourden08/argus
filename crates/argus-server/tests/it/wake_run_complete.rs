//! The wake of a test run that ends: the owner passes the last case of the handed run, the run is
//! ended through the `plan-run-end` op, and the thread that owns the run is woken once.
//! I-99: the run replaces the bundle-complete wake.
// test-tag: standard

use crate::posting::post;
use crate::wake_rig::{lines, result, rig, wake_stream, wake_where};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn ending_a_run_after_an_owner_pass_wakes_the_thread_with_run_complete() {
    let r = rig().await;
    let t = &r.t;
    let plan = json!({ "c": "items", "id": "I-20" });
    let ui = wake_stream(t, "ui", "alpha").await;

    let started = post(t, &json!({ "kind": "plan-run-start", "target": plan })).await;
    assert_eq!(started["status"], "applied", "{started}");
    let record = json!({ "kind": "plan-run-record", "target": plan, "test": "T-2", "result": "pass", "note": "ok" });
    let recorded = post(t, &record).await;
    assert_eq!(recorded["status"], "applied", "{recorded}");
    result(t, "T-2", "pass", "ok").await;
    let ended = post(t, &json!({ "kind": "plan-run-end", "target": plan })).await;
    assert_eq!(ended["status"], "applied");

    let row = wake_where(t, "alpha", |w| w["reason"] == "run-complete").await;
    assert_eq!(row["thread"], "ui");
    assert_eq!(row["subject"], "I-21");
    let wake = ui.next(Some("wake"), 5000).await.expect("the run wake");
    assert_eq!(lines(&wake.data), ["run 1: all tests run, 1 pass 0 fail"]);
}
