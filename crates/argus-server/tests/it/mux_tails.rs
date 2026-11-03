//! Run-log tails on the mux: replay, live lines, the final status and `tail-end`, `untail`, a
//! closed stream that leaves no listener, and a run that is gone.
// test-tag: critical

use argus_testkit::Event;
use serde_json::{Value, json};

use crate::mux_rig::{Mux, WAIT};
use crate::runs_rig::{Rig, until};

fn tail(sub: &str, rid: &str, after: u64) -> Value {
    json!({ "op": "tail", "sub": sub, "key": "alpha", "rid": rid, "after": after })
}

fn untail(sub: &str) -> Value {
    json!({ "op": "untail", "sub": sub })
}

fn pid_of(run: &Value) -> u32 {
    u32::try_from(run["pid"].as_u64().expect("a pid")).expect("a pid fits")
}

/// What the run tails hold on the server: the log feed and the bus.
fn listeners(r: &Rig) -> (usize, usize) {
    (
        r.t.svc.run_logs.listener_count(),
        r.t.svc.bus.listener_count(),
    )
}

async fn listeners_back_to(r: &Rig, base: (usize, usize)) {
    until("the tail to drop its listeners", || async {
        (listeners(r) == base).then_some(())
    })
    .await;
}

/// The events up to and including the next one with this name.
async fn until_event(m: &Mux, name: &str) -> Vec<Event> {
    let mut got = Vec::new();
    loop {
        let e = m.s.next(None, WAIT).await.expect("an event arrives");
        let done = e.event == name;
        got.push(e);
        if done {
            return got;
        }
    }
}

fn texts(got: &[Event]) -> Vec<&str> {
    got.iter()
        .filter(|e| e.event == "log")
        .filter_map(|e| e.data["text"].as_str())
        .collect()
}

/// A running run that has said `first`, and the mux that will tail it.
async fn running(first: &str) -> (Rig, Mux, u32) {
    let r = Rig::start().await;
    r.define(&[("slow", json!({}))]);
    let pid = pid_of(&r.begin("slow").await);
    r.host.say(pid, &format!("{first}\n")).await;
    r.until_line("R-1", first).await;
    let m = Mux::open(&r.t).await;
    (r, m, pid)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_tail_replays_then_follows_then_sends_the_final_status_and_tail_end() {
    let (r, m, pid) = running("one").await;
    let results = m.ops(&r.t, json!([tail("t1", "R-1", 0)])).await;
    assert_eq!(results, [json!({ "ok": true })]);
    let replayed = until_event(&m, "run").await;
    assert!(texts(&replayed).contains(&"one"), "{replayed:?}");
    assert_eq!(replayed.last().unwrap().data["run"]["status"], "running");
    r.host.say(pid, "two\n").await;
    let live = loop {
        let got = until_event(&m, "log").await;
        if texts(&got).contains(&"two") {
            break got;
        }
    };
    assert!(live.iter().all(|e| e.data["sub"] == "t1"));
    r.host.exit(pid, Some(0)).await;
    let end = until_event(&m, "tail-end").await;
    let [.., status, ended] = end.as_slice() else {
        panic!("the status comes before tail-end: {end:?}");
    };
    assert_eq!(
        (status.event.as_str(), &status.data["run"]["status"]),
        ("run", &json!("exited"))
    );
    assert_eq!(status.data["sub"], "t1");
    assert_eq!(ended.data, json!({ "sub": "t1" }));
    assert!(
        !m.s.raw().contains("id:"),
        "the mux frames have no id lines"
    );
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn after_skips_the_lines_the_tab_has() {
    let (r, m, pid) = running("one").await;
    m.ops(&r.t, json!([tail("t1", "R-1", 0)])).await;
    let first = until_event(&m, "run").await;
    let seen = first
        .iter()
        .filter(|e| e.event == "log")
        .filter_map(|e| e.data["seq"].as_u64())
        .max()
        .expect("replayed lines have a seq");
    m.ops(&r.t, json!([untail("t1")])).await;
    r.host.say(pid, "two\n").await;
    r.until_line("R-1", "two").await;
    m.ops(&r.t, json!([tail("t2", "R-1", seen)])).await;
    let got = until_event(&m, "run").await;
    assert_eq!(texts(&got), ["two"], "{got:?}");
    assert!(got.iter().all(|e| e.data["sub"] == "t2"));
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn untail_stops_the_tail_and_the_stream_stays_open() {
    let (r, m, pid) = running("one").await;
    let base = listeners(&r);
    m.ops(&r.t, json!([tail("t1", "R-1", 0)])).await;
    until_event(&m, "run").await;
    assert_ne!(listeners(&r), base, "the tail listens");
    let gone = m.ops(&r.t, json!([untail("t1"), untail("t1")])).await;
    assert_eq!(gone, [json!({ "ok": true }), json!({ "ok": true })]);
    listeners_back_to(&r, base).await;
    r.host.say(pid, "two\n").await;
    r.until_line("R-1", "two").await;
    assert!(m.s.next(None, 300).await.is_none(), "nothing more arrives");
    let again = m.ops(&r.t, json!([tail("t2", "R-1", 0)])).await;
    assert_eq!(again, [json!({ "ok": true })]);
    let replay = until_event(&m, "run").await;
    assert!(texts(&replay).contains(&"two"), "the stream still works");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_closed_stream_leaves_no_listener() {
    let (r, m, _) = running("one").await;
    let base = listeners(&r);
    m.ops(&r.t, json!([tail("t1", "R-1", 0), tail("t2", "R-1", 0)]))
        .await;
    until_event(&m, "run").await;
    assert_ne!(listeners(&r), base);
    m.s.close();
    listeners_back_to(&r, base).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_run_that_is_gone_answers_tail_end_only() {
    let r = Rig::start().await;
    let m = Mux::open(&r.t).await;
    let base = listeners(&r);
    let results = m.ops(&r.t, json!([tail("t1", "R-9", 0)])).await;
    assert_eq!(results, [json!({ "ok": true })]);
    let got = until_event(&m, "tail-end").await;
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].data, json!({ "sub": "t1" }));
    listeners_back_to(&r, base).await;
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_tail_op_answers_for_itself() {
    let r = Rig::start().await;
    let m = Mux::open(&r.t).await;
    let ops = json!([
        tail("", "R-1", 0),
        tail("t1", "nope", 0),
        { "op": "tail", "sub": "t1", "key": "Bad Key", "rid": "R-1" },
        { "op": "untail" },
        tail("t2", "R-9", 0),
    ]);
    let results = m.ops(&r.t, ops).await;
    let codes: Vec<&str> = results
        .iter()
        .map(|x| x["error"]["code"].as_str().unwrap_or("ok"))
        .collect();
    assert_eq!(
        codes,
        [
            "bad_request",
            "bad_request",
            "bad_request",
            "bad_request",
            "ok"
        ]
    );
    let got = until_event(&m, "tail-end").await;
    assert_eq!(got[0].data["sub"], "t2", "the last op ran");
    m.s.close();
}
