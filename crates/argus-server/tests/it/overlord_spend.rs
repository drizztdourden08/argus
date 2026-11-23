// test-tag: critical
//! The Overlord spend route: the owner's page only, the plan's defaults, and the shape of the reply.

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

fn show() -> Request {
    Request::new("GET", "/api/overlord/spend")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bearer_is_refused() {
    let t = TestServer::start(TestOptions::default()).await;
    let bearer = t.send(t.bearer(show())).await;
    assert_eq!(bearer.status, 403);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_empty_ledger_shows_zero_spent_and_no_cap() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(show())).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["spentUsd"].as_f64(), Some(0.0));
    assert!(body["budgetUsd"].is_null());
    assert_eq!(body["warnAt"].as_f64(), Some(0.8));
    assert!(body["leftUsd"].is_null());
    assert_eq!(body["note"], json!("estimate; the Console is the bill"));
    assert_eq!(body["month"].as_str().map(str::len), Some(7));
    t.stop(false).await;
}
