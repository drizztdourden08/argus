// test-tag: critical
//! The Overlord settings and usage routes: the owner's page only, a bad value refused, a good one
//! saved and echoed, and an empty usage with no session.

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::{Value, json};

fn show() -> Request {
    Request::new("GET", "/api/overlord/settings")
}

fn change(body: &Value) -> Request {
    Request::new("POST", "/api/overlord/settings").json(body)
}

#[tokio::test(flavor = "multi_thread")]
async fn the_settings_need_the_page() {
    let t = TestServer::start(TestOptions::default()).await;
    let bare = t.send(show()).await;
    assert_eq!(bare.status, 403);
    let bearer = t.send(t.bearer(show())).await;
    assert_eq!(bearer.status, 403);
    let write = t.send(t.bearer(change(&json!({ "effort": "low" })))).await;
    assert_eq!(write.status, 403);
    let usage = t
        .send(t.bearer(Request::new("GET", "/api/overlord/usage")))
        .await;
    assert_eq!(usage.status, 403);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_defaults_and_the_choices_are_shown() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(show())).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["model"], json!("sonnet"));
    assert_eq!(body["effort"], json!("high"));
    assert_eq!(body["permissionMode"], json!("bypassPermissions"));
    let models = body["choices"]["models"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert_eq!(models.len(), 5, "the current model is offered too");
    assert_eq!(models[0], json!({ "id": "claude-opus-5-5", "label": "Opus 5.5" }));
    let efforts = json!(["low", "medium", "high", "xhigh", "max"]);
    assert_eq!(body["choices"]["efforts"], efforts);
    assert_eq!(body["choices"]["permissionModes"][2]["label"], json!("Ask"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_effort_is_400_and_nothing_is_saved() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t.send(t.page(change(&json!({ "effort": "huge" })))).await;
    assert_eq!(reply.status, 400, "{}", reply.text());
    let after = t.send(t.page(show())).await.json();
    assert_eq!(after["effort"], json!("high"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_good_change_is_saved_and_echoed() {
    let t = TestServer::start(TestOptions::default()).await;
    let wanted = json!({ "model": "claude-opus-5-5", "effort": "max", "permissionMode": "plan" });
    let reply = t.send(t.page(change(&wanted))).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["model"], json!("claude-opus-5-5"));
    assert_eq!(body["effort"], json!("max"));
    assert_eq!(body["permissionMode"], json!("plan"));
    let after = t.send(t.page(show())).await.json();
    assert_eq!(after["effort"], json!("max"));
    assert_eq!(after["permissionMode"], json!("plan"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_usage_is_zero_with_no_session() {
    let t = TestServer::start(TestOptions::default()).await;
    let reply = t
        .send(t.page(Request::new("GET", "/api/overlord/usage")))
        .await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["session"]["inputTokens"], json!(0));
    assert_eq!(body["session"]["costUsd"].as_f64(), Some(0.0));
    assert_eq!(body["lastTurn"], json!(null));
    assert_eq!(body["context"]["usedTokens"], json!(0));
    assert_eq!(body["context"]["windowTokens"], json!(200_000));
    assert_eq!(body["model"], json!("sonnet"));
    assert_eq!(body["month"]["budgetUsd"], json!(null));
    assert!(body["updatedAt"].is_string());
    t.stop(false).await;
}
