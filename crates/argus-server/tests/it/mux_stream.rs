//! The mux stream (`GET /api/mux`, `POST /api/mux/:conn`): `hello`, subscriptions per scope and
//! collection, resume, unsubscribe and the tagged frames. The connection lifecycle (unknown
//! conn, bad bodies, teardown, shutdown, limits) is in `mux_lifecycle.rs`.
// test-tag: critical

use argus_core::{Coll, DocChange, ResetEvent};
use argus_store::PruneOptions;
use serde_json::json;

use crate::mux_rig::{Mux, WAIT, apply, names, seq, server, set_item, sub, sub_since};

#[tokio::test(flavor = "multi_thread")]
async fn hello_is_the_first_frame_and_names_the_conn() {
    let t = server().await;
    let s = t.sse("/api/mux").await;
    assert_eq!(s.status, 200);
    let hello = s.next(None, WAIT).await.expect("hello");
    assert_eq!(hello.event, "hello");
    assert_eq!(hello.id, None);
    let conn = hello.data["conn"].as_str().expect("a conn id");
    assert_eq!(conn.len(), 32);
    assert!(conn.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(hello.data["v"], 1);
    assert!(s.raw().starts_with("event: hello\ndata: {"));
    assert!(!s.raw().contains("id:"), "the mux stream has no id lines");
    s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn sub_sends_a_snapshot_per_collection_then_a_tagged_ready() {
    let t = server().await;
    let m = Mux::open(&t).await;
    let got = m.sub(&t, sub("alpha", &["items", "meta"])).await;
    assert_eq!(names(&got), ["snapshot", "snapshot", "ready"]);
    let (items, meta) = (&got[0], &got[1]);
    assert_eq!(items.data["scope"], "alpha");
    assert_eq!(items.data["coll"], "items");
    assert_eq!(items.data["seq"], seq(&t));
    assert_eq!(items.data["docs"].as_array().map(Vec::len), Some(2));
    assert_eq!(meta.data["coll"], "meta");
    assert_eq!(items.id, None);
    let ready = json!({ "scope": "alpha", "seq": seq(&t), "colls": ["items", "meta"] });
    assert_eq!(got[2].data, ready);
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_sub_sends_only_the_snapshot_of_its_new_collection() {
    let t = server().await;
    let m = Mux::open(&t).await;
    m.sub(&t, sub("alpha", &["items"])).await;
    let got = m.sub(&t, sub("alpha", &["meta"])).await;
    assert_eq!(names(&got), ["snapshot", "ready"]);
    assert_eq!(got[0].data["coll"], "meta");
    assert_eq!(got[1].data["colls"], json!(["meta"]));
    let again = m.sub(&t, sub_since("alpha", &["items", "meta"], seq(&t))).await;
    assert_eq!(names(&again), ["ready"], "nothing new, no snapshot");
    assert_eq!(again[0].data["colls"], json!([]));
    apply(&t, "alpha", &[set_item("I-3")]);
    let ev = m.s.next(Some("changes"), WAIT).await.expect("changes");
    assert_eq!(ev.data["changes"][0]["id"], "I-3");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn two_scopes_on_one_conn_are_tagged_and_filtered() {
    let t = server().await;
    let m = Mux::open(&t).await;
    m.sub(&t, sub("alpha", &["items"])).await;
    m.sub(&t, sub("beta", &["meta"])).await;
    let meta = DocChange::set(Coll::Meta, "board", json!({ "project": "beta", "n": 2 }));
    apply(&t, "beta", &[set_item("B-2")]);
    apply(&t, "alpha", &[set_item("I-3"), meta.clone()]);
    apply(&t, "beta", &[meta]);
    let first = m.s.next(Some("changes"), WAIT).await.expect("changes");
    assert_eq!(first.data["scope"], "alpha");
    let ids: Vec<&str> = first.data["changes"]
        .as_array()
        .expect("changes")
        .iter()
        .map(|c| c["id"].as_str().expect("an id"))
        .collect();
    assert_eq!(ids, ["I-3"], "only the followed collection of alpha");
    let second = m.s.next(Some("changes"), WAIT).await.expect("changes");
    assert_eq!(second.data["scope"], "beta");
    assert_eq!(second.data["changes"][0]["coll"], "meta");
    assert_eq!(m.s.queued(), 0, "nothing else was delivered");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_servable_since_sends_only_what_was_missed() {
    let t = server().await;
    let since = seq(&t);
    apply(&t, "alpha", &[set_item("I-10")]);
    let m = Mux::open(&t).await;
    let got = m.sub(&t, sub_since("alpha", &["items"], since)).await;
    assert_eq!(names(&got), ["changes", "ready"], "no snapshot");
    assert_eq!(got[0].data["scope"], "alpha");
    assert_eq!(got[0].data["changes"][0]["id"], "I-10");
    assert_eq!(got[1].data["colls"], json!(["items"]));
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_since_the_log_cannot_serve_falls_back_to_snapshots() {
    let t = server().await;
    let m = Mux::open(&t).await;
    let future = sub_since("alpha", &["meta"], seq(&t) + 1000);
    assert_eq!(names(&m.sub(&t, future).await), ["snapshot", "ready"]);
    let old = seq(&t);
    apply(&t, "alpha", &[set_item("I-30"), set_item("I-31")]);
    let prune = PruneOptions {
        max_rows: 0,
        ..PruneOptions::default()
    };
    t.svc.db.docs().prune(prune).expect("prune");
    let pruned = sub_since("beta", &["meta"], old);
    assert_eq!(names(&m.sub(&t, pruned).await), ["snapshot", "ready"]);
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn unsub_stops_a_collection_and_then_the_scope() {
    let t = server().await;
    let m = Mux::open(&t).await;
    m.sub(&t, sub("alpha", &["items", "meta"])).await;
    let ops = json!([{ "op": "unsub", "scope": "alpha", "colls": ["items"] }]);
    assert_eq!(m.ops(&t, ops).await, [json!({ "ok": true })]);
    let meta = DocChange::set(Coll::Meta, "board", json!({ "project": "alpha", "n": 2 }));
    apply(&t, "alpha", &[set_item("I-3"), meta]);
    let ev = m.s.next(Some("changes"), WAIT).await.expect("changes");
    assert_eq!(ev.data["changes"].as_array().map(Vec::len), Some(1));
    assert_eq!(ev.data["changes"][0]["coll"], "meta");
    let all = json!([{ "op": "unsub", "scope": "alpha" }, { "op": "unsub", "scope": "gamma" }]);
    assert_eq!(
        m.ops(&t, all).await,
        [json!({ "ok": true }), json!({ "ok": true })]
    );
    apply(&t, "alpha", &[set_item("I-4")]);
    assert!(m.s.next(Some("changes"), 300).await.is_none());
    let back = m.sub(&t, sub("alpha", &["items"])).await;
    assert_eq!(names(&back), ["snapshot", "ready"], "a fresh snapshot");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reset_is_tagged_with_its_scope_and_reaches_only_followers() {
    let t = server().await;
    let m = Mux::open(&t).await;
    m.sub(&t, sub("alpha", &["meta"])).await;
    let reset = |scope: &str, reason: &str| ResetEvent {
        scope: scope.into(),
        reason: reason.into(),
    };
    t.svc.bus.emit_reset(&reset("beta", "unfollowed"));
    t.svc.bus.emit_reset(&reset("alpha", "resync"));
    let ev = m.s.next(Some("reset"), WAIT).await.expect("reset");
    assert_eq!(ev.data, json!({ "scope": "alpha", "reason": "resync" }));
    assert_eq!(ev.id, None);
    assert!(m.s.next(Some("reset"), 300).await.is_none());
    m.s.close();
}
