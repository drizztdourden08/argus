// test-tag: critical
//! The Overlord status and control routes: the owner's page only, a stopped answer with no
//! supervisor, and a plain 409 when the mode is off.

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

fn status() -> Request {
    Request::new("GET", "/api/overlord/status")
}

fn control(action: &str) -> Request {
    Request::new("POST", "/api/overlord/control").json(&json!({ "action": action }))
}

#[tokio::test(flavor = "multi_thread")]
async fn status_is_stopped_with_no_supervisor() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(status())).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["mode"], json!("live"), "a missing mode is live");
    assert_eq!(body["problem"], json!("no API key is saved"));
    assert_eq!(body["status"], json!("stopped"));
    assert_eq!(body["pid"], json!(null));
    assert_eq!(body["startedAt"], json!(null));
    assert_eq!(body["sessionId"], json!(null));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bearer_is_refused() {
    let t = TestServer::start(TestOptions::default()).await;
    let read = t.send(t.bearer(status())).await;
    assert_eq!(read.status, 403);
    let bearer = t.send(t.bearer(control("start"))).await;
    assert_eq!(bearer.status, 403, "the dashboard token is not the page");
    let bare = t.send(control("start")).await;
    assert_eq!(bare.status, 403);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn start_is_409_when_the_mode_is_off() {
    let options = TestOptions {
        config: json!({ "overlord": { "mode": "off" } }),
        ..TestOptions::default()
    };
    let t = TestServer::start(options).await;
    let reply = t.send(t.page(control("start"))).await;
    assert_eq!(reply.status, 409, "{}", reply.text());
    assert!(reply.text().contains("set overlord.mode to live"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_is_409_when_nothing_runs() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(control("stop"))).await;
    assert_eq!(reply.status, 409, "{}", reply.text());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_action_is_400() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(control("pause"))).await;
    assert_eq!(reply.status, 400, "{}", reply.text());
    t.stop(false).await;
}
