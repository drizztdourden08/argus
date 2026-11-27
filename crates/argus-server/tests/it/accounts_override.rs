//! The owner's overrides of the switch process: the skip routes answer the page only, refuse
//! a step that cannot be skipped or is not under way, and know no lead without a cycle.
// test-tag: critical

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use argus_server::ServeMode;
use argus_testkit::{Request, TestServer};
use serde_json::{Value, json};

use super::accounts_shadow::{post, start};

const SKIP_STEP: &str = "/api/accounts/switch/skip-step";
const SKIP_THREAD: &str = "/api/accounts/switch/skip-thread";

async fn live(dir: &std::path::Path) -> TestServer {
    start(false, ServeMode::Live, dir, &Arc::new(AtomicUsize::new(0))).await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_step_that_cannot_be_skipped_or_is_not_under_way_is_a_conflict() {
    let dir = tempfile::tempdir().expect("dir");
    let t = live(dir.path()).await;
    for stage in ["sync", "switch", "countdown", "handover", "verify"] {
        let status = post(&t, SKIP_STEP, &json!({ "stage": stage })).await.0;
        assert_eq!(status, 409, "{stage}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_or_missing_stage_is_a_bad_request() {
    let dir = tempfile::tempdir().expect("dir");
    let t = live(dir.path()).await;
    assert_eq!(post(&t, SKIP_STEP, &json!({ "stage": "later" })).await.0, 400);
    assert_eq!(post(&t, SKIP_STEP, &json!({})).await.0, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn skipping_a_lead_with_no_cycle_is_not_found() {
    let dir = tempfile::tempdir().expect("dir");
    let t = live(dir.path()).await;
    let body = json!({ "board": "rotp", "thread": "lead" });
    assert_eq!(post(&t, SKIP_THREAD, &body).await.0, 404);
    let half = post(&t, SKIP_THREAD, &json!({ "board": "rotp" })).await;
    assert_eq!(half.0, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_owners_page_may_override() {
    let dir = tempfile::tempdir().expect("dir");
    let t = live(dir.path()).await;
    let body: Value = json!({ "stage": "countdown" });
    let cli = t.send(t.bearer(Request::new("POST", SKIP_STEP).json(&body))).await;
    assert_eq!(cli.status, 403);
}
