//! S2, K2: the stdio bridge against a local HTTP stub that mounts the real handler: a call is
//! forwarded with the default project and the client name, the token goes only when the config
//! requires it, and a server that goes away is started again.
// test-tag: standard

use crate::bridge::{Env, Wire, error_of, init, opts};
use crate::support::stub::{Stub, TOKEN};
use crate::support::{Rig, call};
use argus_mcp::Endpoint;
use serde_json::{Value, json};

#[tokio::test]
async fn tools_call_is_forwarded_with_the_default_project_and_the_client_name_and_no_token() {
    let rig = Rig::new();
    let stub = Stub::start(rig.host.clone(), false).await;
    let mut wire = Wire::start(Env::port(stub.port), opts(Some("beta"), false, 5));
    wire.send(init()).await;
    let r = wire.send(call(2, "board_summary", json!({}))).await;
    assert_eq!(r["result"]["isError"], false);
    assert_eq!(r["result"]["structuredContent"]["project"], "beta");
    let w = wire
        .send(call(3, "decision", json!({ "text": "through the bridge" })))
        .await;
    assert_eq!(w["result"]["structuredContent"]["status"], "applied");
    let posted = rig.backend.calls().post.last().expect("a post").clone();
    assert_eq!(posted["via"], "mcp:bridge-test");
    assert_eq!(posted["scope"], "beta");
    let res = wire.send(json!({ "jsonrpc": "2.0", "id": 4, "method": "resources/read", "params": { "uri": "board://alpha/summary" } })).await;
    let text = res["result"]["contents"][0]["text"].as_str().expect("text");
    assert_eq!(
        serde_json::from_str::<Value>(text).expect("JSON")["project"],
        "alpha"
    );
    assert_eq!(
        wire.send(call(5, "nope", json!({}))).await["error"]["code"],
        -32602
    );
    wire.end().await;
    stub.close().await;
}

#[tokio::test]
async fn concurrent_calls_each_get_their_own_answer() {
    let stub = Stub::start(Rig::new().host.clone(), false).await;
    let mut wire = Wire::start(Env::port(stub.port), opts(Some("alpha"), false, 5));
    wire.send(init()).await;
    for id in [10, 11, 12, 13] {
        wire.raw(&call(id, "list_items", json!({})).to_string())
            .await;
    }
    let mut seen = Vec::new();
    for _ in 0..4 {
        let m = wire.line().await.expect("a reply");
        assert_eq!(
            m["result"]["structuredContent"]["items"]
                .as_array()
                .expect("items")
                .len(),
            4
        );
        seen.push(m["id"].as_i64().expect("an id"));
    }
    seen.sort_unstable();
    assert_eq!(seen, [10, 11, 12, 13]);
    wire.end().await;
    stub.close().await;
}

#[tokio::test]
async fn a_stale_token_in_the_config_does_not_matter_in_the_default_mode() {
    let stub = Stub::start(Rig::new().host.clone(), false).await;
    let env = Env::at(Some(Endpoint::loopback(
        stub.port,
        Some(&"wrong".repeat(13)),
        false,
    )));
    let mut wire = Wire::start(env, opts(None, false, 5));
    wire.send(init()).await;
    assert_eq!(
        wire.send(call(2, "health", json!({}))).await["result"]["isError"],
        false
    );
    wire.end().await;
    stub.close().await;
}

#[tokio::test]
async fn with_require_token_the_bridge_sends_the_token_and_a_wrong_one_reads_as_not_running() {
    let stub = Stub::start(Rig::new().host.clone(), true).await;
    let mut good = Wire::start(
        Env::at(Some(Endpoint::loopback(stub.port, Some(TOKEN), true))),
        opts(None, false, 5),
    );
    good.send(init()).await;
    assert_eq!(
        good.send(call(2, "health", json!({}))).await["result"]["isError"],
        false
    );
    good.end().await;
    let wrong = Env::at(Some(Endpoint::loopback(
        stub.port,
        Some(&"wrong".repeat(13)),
        true,
    )));
    let mut bad = Wire::start(wrong.clone(), opts(None, false, 5));
    bad.send(init()).await;
    let r = bad.send(call(2, "health", json!({}))).await;
    assert_eq!(r["result"]["isError"], true);
    assert!(
        error_of(&r)["message"]
            .as_str()
            .is_some_and(|m| m.contains("dashboard not running"))
    );
    assert_eq!(
        wrong.launches(),
        1,
        "a server that does not answer is started once"
    );
    bad.end().await;
    stub.close().await;
}

#[tokio::test]
async fn without_the_flag_a_crashed_dashboard_is_started_again_once_per_crash() {
    let stub = Stub::start(Rig::new().host.clone(), false).await;
    let env = Env::port(stub.port);
    let mut wire = Wire::start(env.clone(), opts(None, true, 5));
    wire.send(init()).await;
    assert_eq!(env.launches(), 1, "launched at start");
    let ok = wire
        .send(call(2, "board_summary", json!({ "project": "alpha" })))
        .await;
    assert_eq!(ok["result"]["isError"], false);
    stub.close().await;
    let down = wire.send(call(3, "health", json!({}))).await;
    assert_eq!(down["result"]["isError"], true);
    assert_eq!(
        env.launches(),
        2,
        "the crash was answered with a second start"
    );
    wire.send(call(4, "health", json!({}))).await;
    assert_eq!(
        env.launches(),
        2,
        "not again while that start is still pending"
    );
    wire.end().await;
}

#[tokio::test]
async fn an_invalid_default_project_is_ignored_with_a_log_line() {
    let stub = Stub::start(Rig::new().host.clone(), false).await;
    let env = Env::port(stub.port);
    let mut wire = Wire::start(env.clone(), opts(Some("Bad Key"), false, 5));
    wire.send(init()).await;
    let r = wire.send(call(2, "list_items", json!({}))).await;
    assert_eq!(
        error_of(&r)["code"],
        "bad_request",
        "no default project was sent"
    );
    assert!(env.logs().iter().any(|l| l == "[coord-dashboard mcp] ignoring --project \"Bad Key\": not a project key"));
    wire.end().await;
    stub.close().await;
}

#[tokio::test]
async fn the_protocol_version_the_client_asked_for_is_forwarded() {
    let stub = Stub::start(Rig::new().host.clone(), false).await;
    let mut wire = Wire::start(Env::port(stub.port), opts(None, false, 5));
    let mut older = init();
    older["params"]["protocolVersion"] = json!("2025-03-26");
    wire.send(older).await;
    let r = wire.send(call(2, "health", json!({}))).await;
    assert_eq!(
        r["result"]["isError"], false,
        "an older supported version passes the server check"
    );
    wire.end().await;
    stub.close().await;
}
