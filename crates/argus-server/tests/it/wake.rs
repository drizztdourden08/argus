//! Wakes through the real server, ported from `wake.http.test.mjs` and the HTTP side of
//! `inbox.wake.test.mjs`: the wake stream (one coalesced event, star listeners, queued replay
//! for a late listener, listeners 0 when nobody was watching), the `wakes` docs, `POST wakes`,
//! the v2 replacement and legacy streams, the listeners route and board presence.
//!
//! The builds, launchers and rerun cases of `wake.http.test.mjs` belong to the sync and runs
//! slices, and `coord inbox send` itself to the `coord` port of slice 8.
//! S2, K2: the wake stream, the wake docs and the listeners against an in-process server.
// test-tag: standard

use crate::posting::{doc, post};
use crate::wait::until;
use crate::wake_rig::{lines, listeners, post_wake, result, rig, wake_stream, wake_where, wakes};
use argus_testkit::TestServer;
use serde_json::{Value, json};

fn is_wake_id(id: &Value) -> bool {
    let text = id.as_str().unwrap_or_default();
    text.strip_prefix("W-")
        .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

#[tokio::test(flavor = "multi_thread")]
async fn wakes_inside_the_window_arrive_as_one_event_and_the_other_threads_hear_nothing() {
    let r = rig().await;
    let t = &r.t;
    let ui = wake_stream(t, "ui", "alpha").await;
    let api = wake_stream(t, "api", "alpha").await;
    result(t, "T-2", "fail", "The error text is missing from the form").await;
    result(t, "T-3", "partial", "the icon does not change").await;
    let e = ui.next(Some("wake"), 5000).await.expect("a wake event");
    assert_eq!(e.data["scope"], "alpha");
    assert_eq!(e.data["thread"], "ui");
    assert_eq!(e.data["count"], 2);
    assert_eq!(
        lines(&e.data),
        [
            "T-2 failed: The error text is missing from the form",
            "T-3 partial: the icon does not change"
        ]
    );
    assert!(is_wake_id(&e.data["id"]));
    assert_eq!(e.id.as_deref(), e.data["id"].as_str());
    assert!(e.data["at"].as_str().is_some_and(|at| at.ends_with('Z')));
    assert_eq!(api.queued(), 0, "another thread hears nothing");
}

/// The page doc of a wake row.
async fn wake_doc(t: &TestServer, row: &Value) -> Value {
    doc(t, "alpha", "wakes", row["id"].as_str().unwrap_or_default()).await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_event_nobody_heard_is_replayed_once_and_a_pass_that_completes_the_bundle_wakes_once() {
    let r = rig().await;
    let t = &r.t;
    let first = wake_stream(t, "ui", "alpha").await;
    result(t, "T-2", "fail", "The error text is missing from the form").await;
    result(t, "T-3", "partial", "the icon does not change").await;
    first
        .next(Some("wake"), 5000)
        .await
        .expect("the first event");
    first.close();
    result(t, "T-4", "blocked", "device missing").await;
    let queued = wake_where(t, "alpha", |w| {
        w["text"]
            .as_str()
            .is_some_and(|x| x.starts_with("T-4 blocked"))
            && w["status"] == "queued"
    })
    .await;
    assert_eq!(queued["status"], "queued");
    assert_eq!(queued["listeners"], 0);
    assert_eq!(queued["thread"], "ui");
    assert_eq!(queued["reason"], "test-blocked");
    assert_eq!(wake_doc(t, &queued).await["listeners"], 0);

    let late = wake_stream(t, "ui", "alpha").await;
    let replay = late.next(Some("wake"), 5000).await.expect("the replay");
    assert_eq!(lines(&replay.data), ["T-4 blocked: device missing"]);
    assert_eq!(replay.data["id"], queued["id"]);
    let delivered = until("the replayed wake to be delivered", || async {
        let doc = wake_doc(t, &queued).await;
        (doc["status"] == "delivered").then_some(doc)
    })
    .await;
    assert_eq!(delivered["listeners"], 1);

    result(t, "T-5", "pass", "ok").await;
    let done = late.next(Some("wake"), 5000).await.expect("the bundle");
    assert_eq!(
        lines(&done.data),
        ["b1: all manual tests run, 1 pass 3 fail"]
    );
    let row = wake_where(t, "alpha", |w| w["reason"] == "bundle-complete").await;
    assert_eq!(row["subject"], "b1");
    assert_eq!(row["thread"], "ui");
    assert_eq!(row["listeners"], 1);
    assert_eq!(row["status"], "delivered");

    let second = wake_stream(t, "ui", "alpha").await;
    let after = json!({ "thread": "ui", "reason": "thread-message", "text": "lead: after" });
    post_wake(t, "alpha", &after).await;
    let next = second
        .next(Some("wake"), 5000)
        .await
        .expect("the next wake");
    assert_eq!(lines(&next.data), ["lead: after"], "replayed once");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_answer_a_resume_and_a_reassigned_item_wake_the_right_thread_to_a_star_listener() {
    let r = rig().await;
    let t = &r.t;
    let star = wake_stream(t, "*", "alpha").await;
    post(
        t,
        &json!({ "kind": "followup", "replyTo": "Q-1", "text": "Yes, remember it." }),
    )
    .await;
    let answer = star.next(Some("wake"), 5000).await.expect("the answer");
    assert_eq!(answer.data["thread"], "ui");
    assert_eq!(
        lines(&answer.data),
        ["owner answered Q-1: Yes, remember it."]
    );

    post(t, &json!({ "kind": "pause", "replyTo": "ui" })).await;
    post(t, &json!({ "kind": "resume", "replyTo": "ui" })).await;
    let resumed = star.next(Some("wake"), 5000).await.expect("the resume");
    assert_eq!(
        lines(&resumed.data),
        ["ui was resumed by the owner"],
        "pause wakes nobody"
    );

    let moved = post(
        t,
        &json!({ "kind": "move", "target": { "c": "items", "id": "I-2" }, "to": { "thread": "coordinator" } }),
    )
    .await;
    assert_eq!(moved["status"], "applied");
    let assigned = star.next(Some("wake"), 5000).await.expect("the assignment");
    assert_eq!(assigned.data["thread"], "coordinator");
    assert!(
        lines(&assigned.data)[0]
            .starts_with("I-2 (Settings page) was assigned to you by the owner")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn edits_status_moves_and_archives_wake_nobody() {
    let r = rig().await;
    let t = &r.t;
    let star = wake_stream(t, "*", "alpha").await;
    let target = json!({ "c": "items", "id": "I-2" });
    post(
        t,
        &json!({ "kind": "edit", "target": target, "set": { "title": "Settings screen" } }),
    )
    .await;
    post(
        t,
        &json!({ "kind": "move", "target": target, "to": { "status": "doing" } }),
    )
    .await;
    post(t, &json!({ "kind": "archive", "target": target })).await;
    let text = json!({ "kind": "followup", "replyTo": "Q-1", "text": "Yes." });
    post(t, &text).await;
    let first = star.next(Some("wake"), 5000).await.expect("the answer");
    assert_eq!(lines(&first.data), ["owner answered Q-1: Yes."]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_portfolio_send_wakes_the_coordinator_of_the_project_it_was_delivered_to() {
    let r = rig().await;
    let t = &r.t;
    let lead = wake_stream(t, "*", "beta").await;
    let body =
        json!({ "project": "beta", "text": "Freeze the API until Friday.", "decision": true });
    let sent = t.write("POST", "/api/portfolio/send", &body).await.json();
    assert_eq!(sent["status"], "delivered");
    let e = lead.next(Some("wake"), 5000).await.expect("the decision");
    assert_eq!(e.data["scope"], "beta");
    assert_eq!(e.data["thread"], "coordinator");
    assert_eq!(
        lines(&e.data),
        ["owner decision from the Overlord: Freeze the API until Friday."]
    );
    let chips: Vec<Value> = wakes(t, "beta")
        .await
        .into_iter()
        .filter(|w| w["reason"] == "portfolio-decision")
        .collect();
    assert_eq!(chips.len(), 1);
    assert_eq!(chips[0]["listeners"], 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_stream_validates_its_thread_and_listens_to_every_thread_with_none_given() {
    let r = rig().await;
    let bad = r.t.sse("/api/p/alpha/wakes/stream?thread=a%20b").await;
    assert_eq!(bad.status, 400);
    let open = r.t.sse("/api/p/alpha/wakes/stream").await;
    let ready = open.next(Some("ready"), 5000).await.expect("ready");
    assert_eq!(ready.data["thread"], "*");
    assert_eq!(ready.data["windowMs"], 150);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_wake_frames_are_written_byte_for_byte_as_node_writes_them() {
    let r = rig().await;
    let t = &r.t;
    let s = wake_stream(t, "ui", "alpha").await;
    result(t, "T-2", "fail", "bytes").await;
    let e = s.next(Some("wake"), 5000).await.expect("a wake");
    let raw = s.raw();
    let ready = r#"event: ready
data: {"scope":"alpha","thread":"ui","windowMs":150}

"#;
    assert!(raw.starts_with(ready), "ready first: {raw}");
    let id = e.data["id"].as_str().unwrap_or_default();
    let at = e.data["at"].as_str().unwrap_or_default();
    let frame = format!(
        "id: {id}\nevent: wake\ndata: {{\"id\":\"{id}\",\"scope\":\"alpha\",\"thread\":\"ui\",\"at\":\"{at}\",\"lines\":[\"T-2 failed: bytes\"],\"count\":1}}\n\n"
    );
    assert!(raw.contains(&frame), "wake frame: {raw}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_lead_message_goes_through_the_engine_window_star_listener_replay_and_page_doc() {
    let r = rig().await;
    let t = &r.t;
    let star = wake_stream(t, "*", "beta").await;
    let text = format!("beta lead: {}", "a very long message ".repeat(4));
    let sent = post_wake(
        t,
        "beta",
        &json!({ "thread": "coordinator", "reason": "lead-message", "text": text }),
    )
    .await;
    let sent = sent.json();
    assert_eq!(sent["queued"], true);
    assert_eq!(sent["scope"], "beta");
    assert_eq!(sent["thread"], "coordinator");
    assert_eq!(sent["windowMs"], 150);
    let second = json!({ "thread": "coordinator", "reason": "lead-message", "text": "pixelarttool lead: second" });
    post_wake(t, "beta", &second).await;
    let e = star.next(Some("wake"), 5000).await.expect("the event");
    assert_eq!(e.data["count"], 2, "coalesced in one window");
    assert_eq!(lines(&e.data), [text.trim(), "pixelarttool lead: second"]);
    let row = wake_where(t, "beta", |w| {
        w["reason"] == "lead-message" && w["status"] == "delivered"
    })
    .await;
    assert_eq!(row["listeners"], 1);
    assert_eq!(row["count"], 2);
    drop(star);

    let ping =
        json!({ "thread": "coordinator", "reason": "lead-message", "text": "tessera lead: ping" });
    post_wake(t, "beta", &ping).await;
    let queued = wake_where(t, "beta", |w| {
        w["text"] == "tessera lead: ping" && w["status"] == "queued"
    })
    .await;
    assert_eq!(queued["listeners"], 0);
    let late = wake_stream(t, "coordinator", "beta").await;
    let replay = late.next(Some("wake"), 5000).await.expect("the replay");
    assert_eq!(lines(&replay.data), ["tessera lead: ping"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_an_outside_reason_an_existing_thread_and_a_text_are_accepted() {
    let r = rig().await;
    let t = &r.t;
    let before = wakes(t, "beta").await.len();
    let bad = |body: Value, key: &'static str| async move { post_wake(t, key, &body).await.status };
    let ok = json!({ "thread": "coordinator", "reason": "lead-message", "text": "x" });
    assert_eq!(
        bad(
            json!({ "thread": "coordinator", "reason": "test-fail", "text": "x" }),
            "beta"
        )
        .await,
        400,
        "a policy reason"
    );
    assert_eq!(
        bad(json!({ "thread": "coordinator", "text": "x" }), "beta").await,
        400,
        "no reason"
    );
    assert_eq!(
        bad(
            json!({ "thread": "*", "reason": "lead-message", "text": "x" }),
            "beta"
        )
        .await,
        400,
        "a star"
    );
    assert_eq!(
        bad(json!({ "reason": "lead-message", "text": "x" }), "beta").await,
        400,
        "no thread"
    );
    assert_eq!(
        bad(
            json!({ "thread": "coordinator", "reason": "lead-message", "text": "  " }),
            "beta"
        )
        .await,
        400,
        "no text"
    );
    assert_eq!(
        bad(
            json!({ "thread": "nobody", "reason": "lead-message", "text": "x" }),
            "beta"
        )
        .await,
        404,
        "unknown thread"
    );
    assert_eq!(bad(ok, "nowhere").await, 404, "unknown board");
    assert_eq!(wakes(t, "beta").await.len(), before, "nothing was recorded");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_newer_v2_stream_for_the_same_thread_replaces_the_older_one() {
    let r = rig().await;
    let t = &r.t;
    let old = wake_stream(t, "coordinator", "beta").await;
    let star = wake_stream(t, "*", "beta").await;
    let mut threads: Vec<String> = listeners(t, "beta")
        .await
        .iter()
        .map(|l| l["thread"].as_str().unwrap_or_default().to_owned())
        .collect();
    threads.sort();
    assert_eq!(threads, ["*", "coordinator"]);
    let fresh = t
        .sse("/api/p/beta/wakes/stream?thread=coordinator&v=2&id=RotP%20lead")
        .await;
    fresh.next(Some("ready"), 5000).await.expect("ready");
    let bye = old.next(Some("replaced"), 5000).await.expect("replaced");
    assert_eq!(bye.data, json!({ "reason": "newer listener" }));
    until("the old connection to close", || async {
        old.ended().then_some(())
    })
    .await;
    let open = listeners(t, "beta").await;
    assert_eq!(
        open.len(),
        2,
        "the replaced one is gone, the star one stays"
    );
    let mine = open
        .iter()
        .find(|l| l["thread"] == "coordinator")
        .expect("mine");
    assert_eq!(mine["label"], "RotP lead");
    assert_eq!(mine["v"], 2);
    assert!(mine["since"].as_str().is_some());

    let hello = json!({ "thread": "coordinator", "reason": "thread-message", "text": "a: hello" });
    post_wake(t, "beta", &hello).await;
    let e = fresh.next(Some("wake"), 5000).await.expect("the event");
    assert_eq!(lines(&e.data), ["a: hello"]);
    assert!(star.next(Some("wake"), 5000).await.is_some());
    assert_eq!(old.queued(), 0, "nothing reached the replaced stream");
    let row = wake_where(t, "beta", |w| {
        w["text"] == "a: hello" && w["status"] == "delivered"
    })
    .await;
    assert_eq!(
        row["listeners"], 2,
        "the coordinator stream and the star stream"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_legacy_stream_is_served_but_not_counted_replayed_to_or_replacing() {
    let r = rig().await;
    let t = &r.t;
    let text = |s: &str| json!({ "thread": "coordinator", "reason": "thread-message", "text": s });
    post_wake(t, "beta", &text("queued before any listener")).await;
    wake_where(t, "beta", |w| {
        w["text"] == "queued before any listener" && w["status"] == "queued"
    })
    .await;
    let legacy = t.sse("/api/p/beta/wakes/stream?thread=coordinator").await;
    let ready = legacy.next(Some("ready"), 5000).await.expect("ready");
    assert_eq!(ready.data["thread"], "coordinator");
    let legacy2 = t.sse("/api/p/beta/wakes/stream?thread=coordinator").await;
    legacy2.next(Some("ready"), 5000).await.expect("ready");
    assert!(
        !legacy.ended(),
        "a second legacy stream does not replace the first"
    );
    let versions: Vec<Value> = listeners(t, "beta")
        .await
        .iter()
        .map(|l| l["v"].clone())
        .collect();
    assert_eq!(versions, [json!(1), json!(1)]);

    post_wake(t, "beta", &text("live while only legacy listens")).await;
    let heard = legacy.next(Some("wake"), 5000).await;
    let heard = heard.expect("a legacy stream hears live wakes");
    assert!(
        lines(&heard.data).contains(&"live while only legacy listens".to_string()),
        "no replay to a legacy listener: its first wake is the live one, which lists what is pending"
    );
    assert!(legacy2.next(Some("wake"), 5000).await.is_some());
    let row = wake_where(t, "beta", |w| {
        w["text"] == "live while only legacy listens" && w["status"] == "queued"
    })
    .await;
    assert_eq!(row["listeners"], 0, "dead orphans cannot swallow a wake");

    let modern = wake_stream(t, "coordinator", "beta").await;
    assert!(!legacy.ended(), "a v2 stream does not replace a legacy one");
    let mut texts = Vec::new();
    while texts.len() < 2 {
        let event = modern.next(Some("wake"), 5000).await.expect("replay");
        texts.extend(lines(&event.data));
    }
    texts.sort();
    assert_eq!(
        texts,
        [
            "live while only legacy listens",
            "queued before any listener"
        ]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_listeners_route_lists_open_streams_and_validates_the_project_key() {
    let r = rig().await;
    let t = &r.t;
    assert_eq!(listeners(t, "alpha").await.len(), 0);
    let s = t
        .sse("/api/p/alpha/wakes/stream?thread=ui&v=2&id=%20%20ui%0Asession%20")
        .await;
    s.next(Some("ready"), 5000).await.expect("ready");
    let open = listeners(t, "alpha").await;
    assert_eq!(open.len(), 1);
    assert_eq!(open[0]["thread"], "ui");
    assert_eq!(open[0]["label"], "ui session");
    assert_eq!(open[0]["v"], 2);
    let mut keys: Vec<&String> = open[0].as_object().expect("an object").keys().collect();
    keys.sort();
    assert_eq!(keys, ["label", "since", "thread", "v"]);
    s.close();
    assert_eq!(t.get("/api/p/Bad_Key/wakes/listeners").await.status, 400);
}

/// The alpha threads with an open board.
async fn open_boards(t: &TestServer) -> Vec<Value> {
    let body = t.get("/api/presence").await.json();
    let rows = body["open"].as_array().cloned().unwrap_or_default();
    rows.into_iter()
        .filter(|r| r["project"] == "alpha")
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_board_page_holds_presence_open_per_thread_and_closing_drops_it() {
    let r = rig().await;
    let t = &r.t;
    let bad = t.sse("/api/p/alpha/presence?thread=*").await;
    assert_eq!(bad.status, 400);
    let s = t.sse("/api/p/alpha/presence?thread=ui&tab=tab1").await;
    assert_eq!(s.status, 200);
    let ready = s.next(Some("ready"), 5000).await.expect("ready");
    assert_eq!(
        ready.data,
        json!({ "scope": "alpha", "thread": "ui", "tab": "tab1" })
    );
    let listed = open_boards(t).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["thread"], "ui");
    assert_eq!(listed[0]["tabs"], 1);
    s.close();
    until("the presence to drop", || async {
        open_boards(t).await.is_empty().then_some(())
    })
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_wake_service_off_its_routes_answer_503() {
    let t = TestServer::start(argus_testkit::TestOptions {
        seed: true,
        tweak: Some(std::sync::Arc::new(|o| o.wake.enabled = false)),
        ..argus_testkit::TestOptions::default()
    })
    .await;
    assert_eq!(t.get("/api/p/alpha/wakes").await.status, 503);
    assert_eq!(t.get("/api/wake-faults").await.code(), "unavailable");
    assert_eq!(t.sse("/api/p/alpha/wakes/stream").await.status, 503);
}
