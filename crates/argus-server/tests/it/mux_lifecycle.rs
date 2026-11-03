//! The mux connection lifecycle: the POST answers (unknown conn, bad bodies, per-op errors), a
//! closed stream forgotten by health, the registry and the fan-out, and shutdown. The 1 MB live
//! drop and the 64-connection cap are in `mux_limits.rs`.
// test-tag: critical

use argus_server::sse::MAX_CLIENTS_PER_SCOPE;
use argus_testkit::{Reply, Request, TestServer};
use serde_json::{Value, json};

use crate::mux_rig::{Mux, all_forgotten, health_streams, server, sub};
use crate::wait::until;

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
        { "op": "tail", "sub": "s", "key": "alpha", "rid": "R-1", "after": 0 },
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
        "unavailable",
        "ok",
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
