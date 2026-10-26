//! The page stream: event formats, collection filter, Last-Event-ID and ?since= resume,
//! snapshot fallback, reset, heartbeat, client limits and shutdown (port of `core.sse.test.mjs`).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use argus_core::{Coll, DocChange, ResetEvent};
use argus_server::sse::{MAX_CLIENTS_PER_SCOPE, SSE_BUFFER_LIMIT, frame, open_stream};
use argus_store::PruneOptions;
use argus_testkit::{Event, Request, SseReader, TestOptions, TestServer};
use http_body_util::BodyExt;
use serde_json::{Value, json};

const WAIT: u64 = 2000;

async fn server_with(heartbeat: Duration) -> TestServer {
    let t = TestServer::start(TestOptions {
        sync: false,
        heartbeat,
        ..TestOptions::default()
    })
    .await;
    apply(
        &t,
        "alpha",
        &[
            DocChange::set(Coll::Items, "I-1", json!({ "id": "I-1", "title": "one" })),
            DocChange::set(Coll::Items, "I-2", json!({ "id": "I-2", "title": "two" })),
            DocChange::set(Coll::Meta, "board", json!({ "project": "alpha" })),
        ],
    );
    t
}

async fn server() -> TestServer {
    server_with(Duration::from_secs(15)).await
}

fn apply(t: &TestServer, scope: &str, changes: &[DocChange]) -> i64 {
    t.svc.db.docs().apply(scope, changes).expect("apply").seq
}

fn seq(t: &TestServer) -> i64 {
    t.svc.db.docs().seq().expect("seq")
}

fn set_item(id: &str, extra: Value) -> DocChange {
    let mut data = json!({ "id": id });
    if let (Some(d), Value::Object(x)) = (data.as_object_mut(), extra) {
        d.extend(x);
    }
    DocChange::set(Coll::Items, id, data)
}

async fn until_ready(s: &SseReader) -> Vec<Event> {
    let mut got = Vec::new();
    loop {
        let e = s.next(None, WAIT).await.expect("a ready event arrives");
        let done = e.event == "ready";
        got.push(e);
        if done {
            return got;
        }
    }
}

fn header<'a>(s: &'a SseReader, name: &str) -> &'a str {
    let found = s.headers.iter().find(|(k, _)| k == name);
    found.map(|(_, v)| v.as_str()).expect("the header is sent")
}

fn names(got: &[Event]) -> Vec<&str> {
    got.iter().map(|e| e.event.as_str()).collect()
}

async fn resume(t: &TestServer, path: &str, since: i64) -> SseReader {
    t.sse_with(Request::new("GET", path).header("Last-Event-ID", &since.to_string()))
        .await
}

async fn ready_names(s: SseReader) -> Vec<String> {
    let got = until_ready(&s).await;
    s.close();
    got.into_iter().map(|e| e.event).collect()
}

async fn clients(t: &TestServer) -> i64 {
    t.get("/api/health").await.json()["sse"]["clients"]
        .as_i64()
        .expect("a client count")
}

async fn clients_settle_at(t: &TestServer, want: i64) -> i64 {
    for _ in 0..40 {
        if clients(t).await == want {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    clients(t).await
}

#[tokio::test(flavor = "multi_thread")]
async fn connect_sends_one_snapshot_per_collection_then_ready() {
    let t = server().await;
    let s = t.sse("/api/p/alpha/stream?c=items,tests,meta").await;
    assert_eq!(s.status, 200);
    let got = until_ready(&s).await;
    assert_eq!(names(&got), ["snapshot", "snapshot", "snapshot", "ready"]);
    let colls: Vec<&str> = got[..3]
        .iter()
        .map(|e| e.data["coll"].as_str().unwrap())
        .collect();
    assert_eq!(colls, ["items", "tests", "meta"]);
    let (items, tests, meta) = (&got[0], &got[1], &got[2]);
    let now = seq(&t);
    assert_eq!(items.data["scope"], "alpha");
    assert_eq!(items.data["seq"], now);
    let ids: Vec<&str> = items.data["docs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["I-1", "I-2"]);
    assert_eq!(items.data["docs"][0]["title"], "one");
    assert_eq!(
        items.data["docs"][0]["data"],
        json!({ "id": "I-1", "title": "one" })
    );
    assert_eq!(tests.data["docs"], json!([]));
    assert_eq!(meta.data["docs"][0]["id"], "board");
    assert_eq!(items.id.as_deref(), Some(now.to_string().as_str()));
    assert_eq!(got[3].id, None);
    assert_eq!(got[3].data, json!({ "seq": now }));
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn the_raw_bytes_follow_the_documented_format() {
    let t = server().await;
    let s = t.sse("/api/p/alpha/stream?c=meta").await;
    assert!(header(&s, "content-type").starts_with("text/event-stream"));
    assert!(header(&s, "cache-control").contains("no-cache"));
    until_ready(&s).await;
    let n = seq(&t);
    let doc = r#"{"project":"alpha","id":"board","data":{"project":"alpha"}}"#;
    let snap = format!(r#"{{"scope":"alpha","seq":{n},"coll":"meta","docs":[{doc}]}}"#);
    let want = format!(
        "id: {n}\nevent: snapshot\ndata: {snap}\n\nevent: ready\ndata: {{\"seq\":{n}}}\n\n"
    );
    assert_eq!(s.raw(), want);
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn changes_are_one_event_per_apply_filtered_to_the_collections() {
    let t = server().await;
    let s = t.sse("/api/p/alpha/stream?c=items,tests").await;
    until_ready(&s).await;
    let at = apply(
        &t,
        "alpha",
        &[
            set_item("I-3", json!({})),
            DocChange::set(Coll::Tests, "T-1", json!({ "id": "T-1" })),
            DocChange::set(Coll::Agents, "A-1", json!({ "id": "A-1" })),
            DocChange::delete(Coll::Items, "I-1"),
        ],
    );
    let ev = s.next(Some("changes"), WAIT).await.expect("changes");
    assert_eq!(ev.id, Some(at.to_string()));
    assert_eq!(ev.data["scope"], "alpha");
    assert_eq!(ev.data["seq"], at);
    assert_eq!(
        ev.data["changes"],
        json!([
            { "coll": "items", "id": "I-3", "op": "set", "data": { "id": "I-3" } },
            { "coll": "tests", "id": "T-1", "op": "set", "data": { "id": "T-1" } },
            { "coll": "items", "id": "I-1", "op": "delete" },
        ])
    );
    let agent = DocChange::set(Coll::Agents, "A-2", json!({ "id": "A-2" }));
    apply(&t, "alpha", &[agent]);
    apply(&t, "beta", &[set_item("B-1", json!({}))]);
    apply(&t, "alpha", &[set_item("I-1", json!({ "title": "back" }))]);
    let next = s.next(Some("changes"), WAIT).await.expect("changes");
    let ids: Vec<&str> = next.data["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        ["I-1"],
        "other collections and scopes were not delivered"
    );
    assert_eq!(s.queued(), 0);
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn an_apply_that_changes_nothing_sends_nothing() {
    let t = server().await;
    let s = t.sse("/api/p/alpha/stream?c=items").await;
    until_ready(&s).await;
    apply(&t, "alpha", &[set_item("I-2", json!({ "title": "two" }))]);
    assert!(s.next(Some("changes"), 250).await.is_none());
    s.close();
}

fn change_summary(c: &Value) -> String {
    let v = c["data"]["v"]
        .as_i64()
        .map_or_else(String::new, |v| format!(":{v}"));
    format!(
        "{}:{}{v}",
        c["op"].as_str().unwrap(),
        c["id"].as_str().unwrap()
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn resume_with_last_event_id_sends_only_what_was_missed_collapsed() {
    let t = server().await;
    let first = t.sse("/api/p/alpha/stream?c=items").await;
    until_ready(&first).await;
    let since = seq(&t);
    first.close();
    apply(&t, "alpha", &[set_item("I-10", json!({ "v": 1 }))]);
    apply(&t, "alpha", &[set_item("I-10", json!({ "v": 2 }))]);
    apply(&t, "alpha", &[DocChange::delete(Coll::Items, "I-2")]);
    let test = DocChange::set(Coll::Tests, "T-9", json!({ "id": "T-9" }));
    apply(&t, "alpha", &[test]);
    let s = resume(&t, "/api/p/alpha/stream?c=items", since).await;
    let got = until_ready(&s).await;
    assert_eq!(names(&got), ["changes", "ready"], "no snapshot");
    let list: Vec<String> = got[0].data["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(change_summary)
        .collect();
    assert_eq!(list, ["set:I-10:2", "delete:I-2"]);
    let newest = seq(&t) - 1;
    assert_eq!(got[0].data["seq"], newest, "newest missed change here");
    assert_eq!(got[0].id, Some(newest.to_string()));
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn resume_works_with_since_and_the_header_wins_over_the_query() {
    let t = server().await;
    let since = seq(&t);
    apply(&t, "alpha", &[set_item("I-20", json!({}))]);
    let q = t
        .sse(&format!("/api/p/alpha/stream?c=items&since={since}"))
        .await;
    assert_eq!(ready_names(q).await, ["changes", "ready"]);
    let both = resume(&t, "/api/p/alpha/stream?c=items&since=0", seq(&t)).await;
    assert_eq!(
        ready_names(both).await,
        ["ready"],
        "nothing missed since the header seq"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_resume_point_the_log_cannot_serve_falls_back_to_snapshots() {
    let t = server().await;
    let path = "/api/p/alpha/stream?c=meta";
    let future = resume(&t, path, seq(&t) + 1000).await;
    assert_eq!(ready_names(future).await, ["snapshot", "ready"]);
    let old = seq(&t);
    apply(
        &t,
        "alpha",
        &[set_item("I-30", json!({})), set_item("I-31", json!({}))],
    );
    let prune = PruneOptions {
        max_rows: 0,
        ..PruneOptions::default()
    };
    t.svc.db.docs().prune(prune).expect("prune");
    let pruned = resume(&t, path, old).await;
    let got = until_ready(&pruned).await;
    assert_eq!(names(&got), ["snapshot", "ready"]);
    assert_eq!(got[1].data["seq"], seq(&t));
    pruned.close();
    let junk = t.sse("/api/p/alpha/stream?c=meta&since=abc").await;
    assert_eq!(ready_names(junk).await, ["snapshot", "ready"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn reset_reaches_that_scope_only_in_the_documented_format() {
    let t = server().await;
    let a = t.sse("/api/p/alpha/stream?c=meta").await;
    let b = t.sse("/api/p/beta/stream?c=meta").await;
    until_ready(&a).await;
    until_ready(&b).await;
    t.svc.bus.emit_reset(&ResetEvent {
        scope: "alpha".into(),
        reason: "resync".into(),
    });
    let ev = a.next(Some("reset"), WAIT).await.expect("reset");
    assert_eq!(ev.data, json!({ "reason": "resync" }));
    assert_eq!(ev.id, None);
    assert!(b.next(Some("reset"), 200).await.is_none());
    a.close();
    b.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn heartbeat_comments_keep_the_stream_alive() {
    let t = server_with(Duration::from_millis(120)).await;
    let s = t.sse("/api/p/alpha/stream?c=meta").await;
    until_ready(&s).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    assert!(
        s.comments().contains(&"ping".to_owned()),
        "{:?}",
        s.comments()
    );
    assert!(s.raw().contains(": ping\n\n"));
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_requests_answer_json_errors_not_streams() {
    let t = server().await;
    assert_eq!(t.sse("/api/p/alpha/stream?c=nope").await.status, 400);
    assert_eq!(t.sse("/api/p/Bad_Scope/stream").await.status, 400);
    assert_eq!(t.sse("/api/p/alpha/stream?c=,").await.status, 400);
    let bearer = Request::new("GET", "/api/p/alpha/stream").header("Authorization", "Bearer nope");
    let wrong = t.sse_with(bearer).await;
    assert_eq!(wrong.status, 401, "a wrong bearer is refused");
    assert_eq!(wrong.body["error"]["code"], "unauthorized");
    let open = t.sse("/api/p/alpha/stream?c=meta").await;
    assert_eq!(open.status, 200, "no sign-in is needed");
    open.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn with_no_c_every_collection_is_streamed() {
    let t = server().await;
    let s = t.sse("/api/p/alpha/stream").await;
    let got = until_ready(&s).await;
    let snapshots = got.iter().filter(|e| e.event == "snapshot").count();
    assert_eq!(snapshots, Coll::ALL.len());
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn health_counts_open_streams_and_a_closed_stream_is_forgotten() {
    let t = server().await;
    let before = clients_settle_at(&t, 0).await;
    assert_eq!(before, 0, "no stream is open yet");
    let s = t.sse("/api/p/alpha/stream?c=meta").await;
    until_ready(&s).await;
    assert_eq!(clients(&t).await, before + 1);
    s.close();
    assert_eq!(clients_settle_at(&t, before).await, before);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_scope_holds_at_most_32_streams() {
    let t = server().await;
    assert_eq!(MAX_CLIENTS_PER_SCOPE, 32);
    let mut open = Vec::new();
    for i in 0..MAX_CLIENTS_PER_SCOPE {
        let s = t.sse("/api/p/gamma/stream?c=meta").await;
        assert_eq!(s.status, 200, "stream {i}");
        open.push(s);
    }
    let over = t.sse("/api/p/gamma/stream?c=meta").await;
    assert_eq!(over.status, 503);
    assert_eq!(over.body["error"]["code"], "unavailable");
    let other = t.sse("/api/p/delta/stream?c=meta").await;
    assert_eq!(other.status, 200, "other scopes are not affected");
    other.close();
    open.into_iter().for_each(SseReader::close);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_that_falls_a_megabyte_behind_is_dropped() {
    let open = Arc::new(AtomicUsize::new(0));
    let (sender, res) = open_stream(Duration::from_secs(3600), &open, None);
    assert_eq!(open.load(Ordering::SeqCst), 1);
    assert!(sender.event("a", &json!({ "n": 1 }), Some(5)));
    let first = frame("a", &json!({ "n": 1 }), Some(5));
    assert_eq!(first, "id: 5\nevent: a\ndata: {\"n\":1}\n\n");
    let chunk = json!({ "pad": "x".repeat(64 * 1024) });
    let mut sent = 0;
    while sender.event("a", &chunk, None) {
        sent += 1;
        assert!(
            sent < 32,
            "the limit of {SSE_BUFFER_LIMIT} bytes must stop it"
        );
    }
    assert!((10..=16).contains(&sent), "{sent} chunks fit");
    assert!(
        !sender.event("a", &json!({ "n": 3 }), None),
        "nothing after close"
    );
    let end = tokio::time::timeout(Duration::from_secs(2), res.into_body().collect());
    let bytes = end.await.expect("the body ends").expect("body").to_bytes();
    assert!(bytes.starts_with(first.as_bytes()));
    assert_eq!(open.load(Ordering::SeqCst), 0, "the stream is forgotten");
}

#[tokio::test(flavor = "multi_thread")]
async fn stopping_the_server_ends_open_streams() {
    let t = server().await;
    let s = t.sse("/api/p/alpha/stream?c=meta").await;
    until_ready(&s).await;
    t.stop(false).await;
    for _ in 0..40 {
        if s.ended() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(s.ended(), "the client saw the stream end");
}
