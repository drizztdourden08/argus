//! `notify_owner`: it speaks to the owner and does not post as him.
//! K2, K4: `notify_owner` speaks to the owner and refuses bad arguments.
// test-tag: critical

use argus_mcp::{CallContext, MemoryBackend};
use serde_json::json;

use crate::support::{Rig, ctx, fixture};

fn on() -> Rig {
    Rig::over(MemoryBackend::new(fixture()).with_notify())
}

#[tokio::test]
async fn it_is_unavailable_while_notifications_are_off() {
    let e = Rig::new()
        .err(
            "notify_owner",
            json!({ "text": "build ready" }),
            "unavailable",
        )
        .await;
    assert_eq!(e["message"], "notifications are off");
}

#[tokio::test]
async fn a_ping_is_recorded_with_the_caller_and_the_default_level() {
    let rig = on();
    let r = rig
        .ok(
            "notify_owner",
            json!({ "text": "build ready", "title": "Alpha" }),
        )
        .await;
    assert_eq!(
        r,
        json!({ "recorded": true, "id": "N-1", "level": "ping", "toast": true })
    );
    let sent = rig.backend.calls().notify[0].clone();
    assert_eq!(
        sent,
        json!({ "title": "Alpha", "text": "build ready", "level": "ping", "project": null, "thread": "mcp:tester" })
    );
    assert!(
        rig.backend.calls().post.is_empty(),
        "it speaks to the owner, it posts nothing"
    );
}

#[tokio::test]
async fn the_project_comes_from_the_argument_or_the_default() {
    let rig = on();
    let beta = CallContext {
        default_project: Some("beta".into()),
        ..ctx()
    };
    rig.ok_as(
        "notify_owner",
        json!({ "text": "a", "level": "info" }),
        &beta,
    )
    .await;
    rig.ok_as(
        "notify_owner",
        json!({ "text": "b", "project": "alpha" }),
        &beta,
    )
    .await;
    let calls = rig.backend.calls().notify;
    assert_eq!(calls[0]["project"], "beta");
    assert_eq!(calls[0]["level"], "info");
    assert_eq!(calls[1]["project"], "alpha");
}

#[tokio::test]
async fn the_arguments_are_checked() {
    let rig = on();
    rig.err("notify_owner", json!({}), "bad_request").await;
    rig.err("notify_owner", json!({ "text": "" }), "bad_request")
        .await;
    rig.err(
        "notify_owner",
        json!({ "text": "x", "level": "loud" }),
        "bad_request",
    )
    .await;
    rig.err(
        "notify_owner",
        json!({ "text": "x".repeat(241) }),
        "bad_request",
    )
    .await;
    assert_eq!(rig.backend.calls().notify.len(), 0);
}
