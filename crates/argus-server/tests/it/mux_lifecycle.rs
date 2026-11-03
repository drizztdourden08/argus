//! The mux connection lifecycle: the POST answers (unknown conn, bad bodies, per-op errors), a
//! closed stream forgotten by health, the registry and the fan-out, shutdown, the 1 MB live
//! drop and the 64-connection cap.
// test-tag: critical

use std::time::Duration;

use argus_core::{Coll, DocChange};
use argus_server::sse::{MAX_CLIENTS_PER_SCOPE, MAX_CONNS};
use argus_testkit::{Reply, Request, SseReader, TestServer};
use http_body_util::BodyExt;
use serde_json::{Value, json};

use crate::mux_rig::{Mux, WAIT, apply, server, sub};
use crate::wait::until;

async fn health_streams(t: &TestServer) -> i64 {
    t.get("/api/health").await.json()["sse"]["clients"]
        .as_i64()
        .expect("a client count")
}

async fn all_forgotten(t: &TestServer) {
    until("every mux stream to be forgotten", || async {
        let gone = t.svc.mux.is_empty() && health_streams(t).await == 0;
        gone.then_some(())
    })
    .await;
}

fn codes(results: &[Value]) -> Vec<String> {
    let code = |r: &Value| r["error"]["code"].as_str().unwrap_or("ok").to_owned();
    results.iter().map(code).collect()
}

async fn post(t: &TestServer, path: &str, body: Value) -> Reply {
    t.write("POST", path, &body).await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_conn_is_a_404() {
    let t = server().await;
    let r = t
        .write(
            "POST",
            "/api/mux/nope",
            &json!({ "ops": [sub("alpha", &["meta"])] }),
        )
        .await;
    assert_eq!(r.status, 404);
    assert_eq!(r.code(), "not_found");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_body_is_a_400() {
    let t = server().await;
    let m = Mux::open(&t).await;
    let path = m.path();
    assert_eq!(post(&t, &path, json!({})).await.status, 400, "no ops");
    assert_eq!(
        post(&t, &path, json!({ "ops": [] })).await.status,
        400,
        "empty ops"
    );
    assert_eq!(
        post(&t, &path, json!({ "ops": "sub" })).await.status,
        400,
        "ops not a list"
    );
    let leave = json!({ "op": "unsub", "scope": "alpha" });
    let over = post(&t, &path, json!({ "ops": vec![leave.clone(); 51] })).await;
    assert_eq!((over.status, over.code().as_str()), (400, "bad_request"));
    let at_most = post(&t, &path, json!({ "ops": vec![leave; 50] })).await;
    assert_eq!(at_most.status, 200, "50 ops are allowed");
    let junk = Request::new("POST", &m.path())
        .header("Content-Type", "application/json")
        .body("{");
    assert_eq!(t.send(junk).await.status, 400, "not JSON");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn each_op_answers_for_itself_and_the_others_still_run() {
    let t = server().await;
    let m = Mux::open(&t).await;
    let ops = json!([
        sub("Bad_Scope", &["items"]),
        sub("alpha", &[]),
        sub("alpha", &["nope"]),
        { "op": "fly" },
        5,
        { "op": "presence", "scope": "alpha", "thread": "t", "tab": "x" },
        { "op": "untail", "sub": "s" },
        sub("alpha", &["meta"]),
    ]);
    let results = m.ops(&t, ops).await;
    let want = [
        "bad_request",
        "bad_request",
        "bad_request",
        "bad_request",
        "bad_request",
        "not_supported",
        "not_supported",
        "ok",
    ];
    assert_eq!(codes(&results), want);
    assert_eq!(results[7], json!({ "ok": true }));
    let ready = m.until_ready().await;
    assert_eq!(ready[0].data["coll"], "meta", "the last op ran");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_closed_stream_is_forgotten_by_health_the_registry_and_the_fanout() {
    let t = server().await;
    all_forgotten(&t).await;
    let mut open = Vec::new();
    for _ in 0..MAX_CLIENTS_PER_SCOPE {
        let m = Mux::open(&t).await;
        m.sub(&t, sub("alpha", &["meta"])).await;
        open.push(m);
    }
    assert_eq!(health_streams(&t).await, 32);
    assert_eq!(t.svc.mux.len(), 32);
    let extra = Mux::open(&t).await;
    let full = extra.ops(&t, json!([sub("alpha", &["meta"])])).await;
    assert_eq!(codes(&full), ["unavailable"], "the scope is full");
    let path = extra.path();
    drop(open);
    drop(extra);
    all_forgotten(&t).await;
    let gone = t
        .write("POST", &path, &json!({ "ops": [sub("beta", &["meta"])] }))
        .await;
    assert_eq!(gone.status, 404, "a forgotten conn is unknown");
    let mut again = Vec::new();
    for _ in 0..MAX_CLIENTS_PER_SCOPE {
        let m = Mux::open(&t).await;
        m.sub(&t, sub("alpha", &["meta"])).await;
        again.push(m);
    }
    assert_eq!(again.len(), 32, "the closed streams left the fan-out");
}

#[tokio::test(flavor = "multi_thread")]
async fn stopping_the_server_ends_every_mux_stream() {
    let t = server().await;
    let busy = Mux::open(&t).await;
    busy.sub(&t, sub("alpha", &["meta"])).await;
    let idle = Mux::open(&t).await;
    t.stop(false).await;
    for m in [&busy, &idle] {
        until("the stream to end", || async { m.s.ended().then_some(()) }).await;
    }
}

fn padded(id: &str) -> DocChange {
    let pad = "x".repeat(64 * 1024);
    DocChange::set(Coll::Items, id, json!({ "id": id, "pad": pad }))
}

fn conn_of(hello: &[u8]) -> String {
    let text = String::from_utf8_lossy(hello);
    let data = text
        .lines()
        .find_map(|l| l.strip_prefix("data: "))
        .expect("data");
    let v: Value = serde_json::from_str(data).expect("hello json");
    v["conn"].as_str().expect("a conn").to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_conn_a_megabyte_behind_on_live_frames_is_dropped_and_forgotten() {
    let t = server().await;
    let svc = &t.svc;
    let res = svc
        .mux
        .open(&svc.fanout, Duration::from_secs(3600), &svc.streams)
        .expect("opens");
    let mut body = res.into_body();
    let first = body.frame().await.expect("a frame").expect("body");
    let conn = svc
        .mux
        .get(&conn_of(first.data_ref().expect("data")))
        .expect("conn");
    let done = conn
        .apply(&svc.db, &svc.fanout, &[sub("alpha", &["items"])])
        .expect("applies");
    assert_eq!(done, [json!({ "ok": true })]);
    for i in 0..32 {
        apply(&t, "alpha", &[padded(&format!("P-{i}"))]);
    }
    let end = tokio::time::timeout(Duration::from_millis(WAIT), body.collect());
    let rest = end.await.expect("the body ends").expect("body").to_bytes();
    let text = String::from_utf8_lossy(&rest);
    let live = text.matches("event: changes").count();
    assert!((10..=17).contains(&live), "{live} live frames fit");
    all_forgotten(&t).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_server_holds_at_most_64_conns() {
    let t = server().await;
    assert_eq!(MAX_CONNS, 64);
    let mut open = Vec::new();
    for i in 0..MAX_CONNS {
        let s = t.sse("/api/mux").await;
        assert_eq!(s.status, 200, "conn {i}");
        open.push(s);
    }
    let over = t.sse("/api/mux").await;
    assert_eq!(over.status, 503);
    assert_eq!(over.body["error"]["code"], "unavailable");
    open.pop().map(SseReader::close);
    let freed = until("a slot to free", || async {
        let s = t.sse("/api/mux").await;
        (s.status == 200).then_some(s)
    })
    .await;
    freed.close();
    open.into_iter().for_each(SseReader::close);
}
