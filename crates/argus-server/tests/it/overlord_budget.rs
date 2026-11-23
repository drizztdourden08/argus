// test-tag: critical
//! The Overlord budget route: the owner's page only, a saved figure the spend route shows, and
//! a 400 for a figure out of range.

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

fn put(usd: f64) -> Request {
    Request::new("PUT", "/api/overlord/budget").json(&json!({ "budgetUsd": usd }))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_put_without_the_page_origin_is_refused() {
    let t = TestServer::start(TestOptions::default()).await;
    let bearer = t.send(t.bearer(put(50.0))).await;
    assert_eq!(bearer.status, 403, "{}", bearer.text());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_saved_budget_shows_in_the_spend_route() {
    let t = TestServer::start(TestOptions::default()).await;
    let saved = t.send(t.page(put(42.5))).await;
    assert_eq!(saved.status, 200, "{}", saved.text());
    assert_eq!(saved.json()["budgetUsd"].as_f64(), Some(42.5));
    let spend = t
        .send(t.page(Request::new("GET", "/api/overlord/spend")))
        .await;
    let body = spend.json();
    assert_eq!(body["budgetUsd"].as_f64(), Some(42.5));
    assert_eq!(body["leftUsd"].as_f64(), Some(42.5));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_null_budget_clears_it_back_to_no_cap() {
    let t = TestServer::start(TestOptions::default()).await;
    t.send(t.page(put(42.5))).await;
    let clear = Request::new("PUT", "/api/overlord/budget").json(&json!({ "budgetUsd": null }));
    let cleared = t.send(t.page(clear)).await;
    assert_eq!(cleared.status, 200, "{}", cleared.text());
    assert!(cleared.json()["budgetUsd"].is_null());
    assert!(cleared.json()["leftUsd"].is_null());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_budget_out_of_range_is_400() {
    let t = TestServer::start(TestOptions::default()).await;
    let negative = t.send(t.page(put(-1.0))).await;
    assert_eq!(negative.status, 400, "{}", negative.text());
    let zero = t.send(t.page(put(0.0))).await;
    assert_eq!(zero.status, 400, "{}", zero.text());
    let huge = t.send(t.page(put(10_000.5))).await;
    assert_eq!(huge.status, 400, "{}", huge.text());
    t.stop(false).await;
}
