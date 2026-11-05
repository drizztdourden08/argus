//! The wake of a test run that ends: the owner passes the case of the handed run, ends it through
//! the `plan-run-end` op, and the thread of the run hears `plan-run-done` with the counts, once.
//! D-63: `plan-run-done` is the only wake of a finished run.
// test-tag: standard

use crate::posting::post;
use crate::wait::wakes_of;
use crate::wake_rig::{lines, rig, wake_stream, wake_where};
use serde_json::json;

const LINE: &str = "I-21 (Login smoke run 1): run 1: 1 pass";

#[tokio::test(flavor = "multi_thread")]
async fn ending_a_run_wakes_the_thread_with_plan_run_done_and_the_counts_once() {
    let r = rig().await;
    let t = &r.t;
    let plan = json!({ "c": "items", "id": "I-20" });
    let ui = wake_stream(t, "ui", "alpha").await;

    let started = post(t, &json!({ "kind": "plan-run-start", "target": plan })).await;
    assert_eq!(started["status"], "applied", "{started}");
    let record = json!({ "kind": "plan-run-record", "target": plan, "test": "T-2", "result": "pass", "note": "ok" });
    let recorded = post(t, &record).await;
    assert_eq!(recorded["status"], "applied", "{recorded}");
    let ended = post(t, &json!({ "kind": "plan-run-end", "target": plan })).await;
    assert_eq!(ended["status"], "applied", "{ended}");

    let row = wake_where(t, "alpha", |w| w["reason"] == "plan-run-done").await;
    assert_eq!(row["thread"], "ui");
    let heard = ui
        .next_where(
            |e| e.event == "wake" && lines(&e.data).iter().any(|l| l == LINE),
            5000,
        )
        .await;
    assert!(heard.is_some(), "the run wake says {LINE}");
    t.write("POST", "/api/admin/resync", &json!({ "project": "alpha" }))
        .await;
    assert_eq!(wakes_of(t, "alpha", "plan-run-done").await.len(), 1);
}
