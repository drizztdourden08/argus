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
    assert_eq!(
        models[0],
        json!({ "id": "claude-opus-5-5", "label": "Opus 5.5" })
    );
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
async fn the_folder_and_limits_say_where_they_come_from() {
    let t = TestServer::start(TestOptions::default()).await;
    let body = t.send(t.page(show())).await.json();
    for name in [
        "cwd",
        "claudeExe",
        "budgetUsd",
        "restartMax",
        "approvalTimeoutMin",
    ] {
        assert_ne!(body[name]["source"], json!("set"), "{name}");
    }
    assert_eq!(body["budgetUsd"]["value"], json!(null));
    assert_eq!(body["restartMax"]["value"], json!(3));
    assert_eq!(body["approvalTimeoutMin"]["value"], json!(30));
    assert_eq!(body["restartMax"]["source"], json!("default"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_folder_and_limits_are_saved_and_reset() {
    let t = TestServer::start(TestOptions::default()).await;
    let dir = tempfile::tempdir().expect("a temp folder");
    let exe = dir.path().join("claude.exe");
    std::fs::write(&exe, "").expect("a file");
    let folder = dir.path().to_string_lossy().into_owned();
    let program = exe.to_string_lossy().into_owned();
    let wanted = json!({
        "cwd": folder, "claudeExe": program, "budgetUsd": 25.5,
        "restartMax": 5, "approvalTimeoutMin": 10,
    });
    let reply = t.send(t.page(change(&wanted))).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = t.send(t.page(show())).await.json();
    assert_eq!(body["cwd"], json!({ "value": folder, "source": "set" }));
    assert_eq!(body["claudeExe"]["value"], json!(program));
    assert_eq!(body["budgetUsd"], json!({ "value": 25.5, "source": "set" }));
    assert_eq!(body["restartMax"], json!({ "value": 5, "source": "set" }));
    assert_eq!(body["approvalTimeoutMin"]["value"], json!(10));
    let reset = json!({ "cwd": "", "claudeExe": "", "budgetUsd": "", "restartMax": "" });
    let reply = t.send(t.page(change(&reset))).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let after = t.send(t.page(show())).await.json();
    assert_ne!(after["cwd"]["source"], json!("set"));
    assert_eq!(after["claudeExe"]["source"], json!("default"));
    assert_eq!(
        after["budgetUsd"],
        json!({ "value": null, "source": "default" })
    );
    assert_eq!(
        after["restartMax"],
        json!({ "value": 3, "source": "default" })
    );
    assert_eq!(after["approvalTimeoutMin"]["source"], json!("set"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_folder_or_limit_is_400_and_nothing_is_saved() {
    let t = TestServer::start(TestOptions::default()).await;
    let missing = std::env::temp_dir().join("argus-no-such-folder-i207");
    let missing = missing.to_string_lossy().into_owned();
    let bad = [
        json!({ "cwd": missing }),
        json!({ "claudeExe": missing }),
        json!({ "budgetUsd": 0 }),
        json!({ "budgetUsd": 1_000_000 }),
        json!({ "restartMax": 99 }),
        json!({ "approvalTimeoutMin": 0 }),
        json!({ "approvalTimeoutMin": 5000 }),
        json!({ "restartMax": "many" }),
        json!({ "cwd": 7 }),
    ];
    for request in bad {
        let reply = t.send(t.page(change(&request))).await;
        assert_eq!(reply.status, 400, "{request}: {}", reply.text());
    }
    let mixed = json!({ "effort": "low", "restartMax": 99 });
    assert_eq!(t.send(t.page(change(&mixed))).await.status, 400);
    let after = t.send(t.page(show())).await.json();
    assert_eq!(after["effort"], json!("high"));
    assert_eq!(after["restartMax"]["source"], json!("default"));
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
