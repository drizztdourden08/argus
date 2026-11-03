//! Board presence on the mux: a `presence` op lists the thread in the open boards, `unpresence`
//! and a closed stream take it off, a tab id replaces its own claim, and the tab past the cap is
//! an error of its own op.
// test-tag: critical

use argus_testkit::TestServer;
use serde_json::{Value, json};

use crate::mux_rig::Mux;
use crate::wait::until;
use crate::wake_rig::{Rig, rig};

fn presence(thread: &str, tab: &str) -> Value {
    json!({ "op": "presence", "scope": "alpha", "thread": thread, "tab": tab })
}

fn unpresence(tab: &str) -> Value {
    json!({ "op": "unpresence", "scope": "alpha", "tab": tab })
}

async fn open_boards(t: &TestServer) -> Vec<Value> {
    let body = t.get("/api/presence").await.json();
    let rows = body["open"].as_array().cloned().unwrap_or_default();
    rows.into_iter()
        .filter(|r| r["project"] == "alpha")
        .collect()
}

async fn no_board_open(t: &TestServer) {
    until("the open boards to be empty", || async {
        open_boards(t).await.is_empty().then_some(())
    })
    .await;
}

async fn opened() -> (Rig, Mux) {
    let r = rig().await;
    let m = Mux::open(&r.t).await;
    (r, m)
}

fn all_ok(results: &[Value]) -> bool {
    results.iter().all(|r| *r == json!({ "ok": true }))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_presence_op_lists_the_thread_and_unpresence_takes_it_off() {
    let (r, m) = opened().await;
    let t = &r.t;
    let got = m.ops(t, json!([presence("ui", "tab1")])).await;
    assert!(all_ok(&got), "{got:?}");
    let listed = open_boards(t).await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["thread"], "ui");
    assert_eq!(listed[0]["tabs"], 1);
    let gone = m
        .ops(t, json!([unpresence("tab1"), unpresence("tab1")]))
        .await;
    assert!(
        all_ok(&gone),
        "a tab with no claim is not an error: {gone:?}"
    );
    no_board_open(t).await;
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_closed_stream_takes_every_claim_of_the_connection_off() {
    let (r, m) = opened().await;
    let t = &r.t;
    let other = Mux::open(t).await;
    m.ops(t, json!([presence("ui", "a"), presence("api", "b")]))
        .await;
    other.ops(t, json!([presence("ui", "c")])).await;
    assert_eq!(open_boards(t).await.len(), 2);
    m.s.close();
    until("the claims of the closed stream to go", || async {
        let rows = open_boards(t).await;
        (rows.len() == 1 && rows[0]["tabs"] == 1).then_some(())
    })
    .await;
    other.s.close();
    no_board_open(t).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_same_tab_twice_replaces_itself_even_on_another_thread() {
    let (r, m) = opened().await;
    let t = &r.t;
    m.ops(t, json!([presence("ui", "tab1"), presence("ui", "tab1")]))
        .await;
    assert_eq!(open_boards(t).await[0]["tabs"], 1);
    m.ops(t, json!([presence("ui", "tab2")])).await;
    assert_eq!(open_boards(t).await[0]["tabs"], 2);
    m.ops(t, json!([presence("api", "tab1")])).await;
    let rows = open_boards(t).await;
    assert_eq!(rows.len(), 2, "{rows:?}");
    for row in &rows {
        assert_eq!(row["tabs"], 1, "{row}");
    }
    m.ops(t, json!([unpresence("tab1"), unpresence("tab2")]))
        .await;
    no_board_open(t).await;
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn the_ninth_tab_is_an_error_of_its_own_op() {
    let (r, m) = opened().await;
    let t = &r.t;
    let ops: Vec<Value> = (0..9).map(|n| presence("ui", &format!("tab{n}"))).collect();
    let results = m.ops(t, json!(ops)).await;
    assert!(all_ok(&results[..8]), "{results:?}");
    assert_eq!(results[8]["error"]["code"], "unavailable");
    assert_eq!(open_boards(t).await[0]["tabs"], 8);
    let renewal = m.ops(t, json!([presence("ui", "tab0")])).await;
    assert!(
        all_ok(&renewal),
        "a tab that holds a claim renews it: {renewal:?}"
    );
    m.ops(t, json!([unpresence("tab0"), presence("ui", "tab9")]))
        .await;
    assert_eq!(open_boards(t).await[0]["tabs"], 8, "a freed place is taken");
    m.s.close();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_presence_op_answers_for_itself() {
    let (r, m) = opened().await;
    let t = &r.t;
    let ops = json!([
        presence("*", "x"),
        presence("", "x"),
        { "op": "presence", "scope": "Bad_Scope", "thread": "ui", "tab": "x" },
        { "op": "presence", "scope": "alpha", "tab": "x" },
        { "op": "unpresence", "scope": "_portfolio", "tab": "x" },
        presence("ui", "x"),
    ]);
    let results = m.ops(t, ops).await;
    let codes: Vec<&str> = results
        .iter()
        .map(|r| r["error"]["code"].as_str().unwrap_or("ok"))
        .collect();
    assert_eq!(
        codes,
        [
            "bad_request",
            "bad_request",
            "bad_request",
            "bad_request",
            "bad_request",
            "ok"
        ]
    );
    assert_eq!(open_boards(t).await.len(), 1, "the last op ran");
    m.s.close();
}
