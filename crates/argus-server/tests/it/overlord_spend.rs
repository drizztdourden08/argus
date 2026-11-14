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
async fn an_empty_ledger_shows_the_defaults_and_the_whole_budget_left() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(show())).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["spentUsd"], json!(0.0));
    assert_eq!(body["budgetUsd"], json!(100.0));
    assert_eq!(body["warnAt"], json!(0.8));
    assert_eq!(body["leftUsd"], json!(100.0));
    assert_eq!(body["note"], json!("estimate; the Console is the bill"));
    assert_eq!(body["month"].as_str().map(str::len), Some(7));
    t.stop(false).await;
}
