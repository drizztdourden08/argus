// test-tag: critical
//! The Overlord key route: the owner's page only, and the key never comes back.
//! K4: a foreign or missing origin is refused; a saved key shows its last 4 characters and
//! its full text is in no reply and not in the activity log.

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

const KEY: &str = "sk-ant-api03-SECRETSECRETSECRET-abcd";

async fn start() -> TestServer {
    TestServer::start(TestOptions::default()).await
}

fn put(key: &str) -> Request {
    Request::new("PUT", "/api/overlord/key").json(&json!({ "key": key }))
}

fn show() -> Request {
    Request::new("GET", "/api/overlord/key")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_put_with_a_foreign_or_missing_origin_is_refused() {
    let t = start().await;
    let foreign = t.send(put(KEY).header("Origin", "http://evil.test")).await;
    assert_eq!(foreign.status, 403);
    let missing = t.send(put(KEY)).await;
    assert_eq!(missing.status, 403);
    let bearer = t.send(t.bearer(put(KEY))).await;
    assert_eq!(bearer.status, 403);
    let after = t.send(t.page(show())).await;
    assert_eq!(after.json()["set"], json!(false));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_saved_key_shows_its_last_four_only_and_leaks_nowhere() {
    let t = start().await;
    let saved = t.send(t.page(put(KEY))).await;
    assert_eq!(saved.status, 200, "{}", saved.text());
    assert!(!saved.text().contains(KEY));
    let shown = t.send(t.page(show())).await;
    assert_eq!(shown.json()["set"], json!(true));
    assert_eq!(shown.json()["last4"], json!("abcd"));
    assert!(shown.json()["savedAt"].is_string());
    assert!(!shown.text().contains(KEY));
    let activity = t.get("/api/activity?limit=500").await;
    assert!(!activity.text().contains("SECRETSECRET"));
    let blank = t.send(t.page(put("   "))).await;
    assert_eq!(blank.status, 400);
    assert!(!blank.text().contains("SECRETSECRET"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn removing_the_key_clears_it() {
    let t = start().await;
    t.send(t.page(put(KEY))).await;
    let delete = Request::new("DELETE", "/api/overlord/key").json(&json!({}));
    let gone = t.send(t.page(delete)).await;
    assert_eq!(gone.status, 200, "{}", gone.text());
    assert_eq!(gone.json()["set"], json!(false));
    assert_eq!(gone.json()["last4"], json!(null));
    t.stop(false).await;
}
