// test-tag: critical
//! The owner chat over HTTP: only the owner's page may post (K4), and with no running child the
//! answer is 409 with a plain sentence.

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

fn post(text: &str) -> Request {
    Request::new("POST", "/api/overlord/message").json(&json!({ "text": text }))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_post_without_the_page_origin_is_refused() {
    let t = TestServer::start(TestOptions::default()).await;
    let bare = t.send(post("hello")).await;
    assert_eq!(bare.status, 403, "{}", bare.text());
    let bearer = t.send(t.bearer(post("hello"))).await;
    assert_eq!(bearer.status, 403, "the dashboard token is not the page");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_post_without_a_running_child_is_409() {
    let t = TestServer::start(TestOptions::default()).await;
    let sent = t.send(t.page(post("hello"))).await;
    assert_eq!(sent.status, 409, "{}", sent.text());
    assert!(sent.text().contains("the hosted Overlord is not running"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn empty_and_long_texts_are_400() {
    let t = TestServer::start(TestOptions::default()).await;
    let empty = t.send(t.page(post("   "))).await;
    assert_eq!(empty.status, 400, "{}", empty.text());
    let long = t.send(t.page(post(&"a".repeat(20_001)))).await;
    assert_eq!(long.status, 400, "{}", long.text());
    t.stop(false).await;
}
