//! The limits of the mux stream: a connection a megabyte behind on live frames is dropped and
//! forgotten, and the server holds at most 64 connections.
// test-tag: critical

use std::time::Duration;

use argus_core::{Coll, DocChange};
use argus_server::sse::MAX_CONNS;
use argus_testkit::SseReader;
use http_body_util::BodyExt;
use serde_json::{Value, json};

use crate::mux_rig::{WAIT, all_forgotten, apply, server, sub};
use crate::wait::until;

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
        .open(
            &svc.fanout,
            Duration::from_secs(3600),
            &svc.streams,
            svc.wake.clone(),
        )
        .expect("opens");
    let mut body = res.into_body();
    let first = body.frame().await.expect("a frame").expect("body");
    let conn = svc
        .mux
        .get(&conn_of(first.data_ref().expect("data")))
        .expect("conn");
    let done = conn
        .apply(svc, &[sub("alpha", &["items"])])
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
