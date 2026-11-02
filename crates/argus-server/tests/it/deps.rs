//! Cross-board dependencies over HTTP on the fixture boards: an ask and a close through the
//! routes, the refusals with their status and text, and the off answer while `deps.enabled` is
//! false. The rules themselves are tested in `argus-board`.
// test-tag: critical

use argus_testkit::{TestOptions, TestServer};
use serde_json::{Value, json};

const OFF: &str = "dependencies are off (deps.enabled is false in config.json)";

async fn server(config: Value) -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        config,
        ..TestOptions::default()
    })
    .await
}

fn ask_body(target: &str) -> Value {
    json!({
        "project": "alpha", "item": "I-1", "actor": "alpha/ui", "target": target,
        "title": "Expose the save hook",
    })
}

async fn asked(t: &TestServer) -> Value {
    let reply = t.write("POST", "/api/deps", &ask_body("beta")).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()
}

#[tokio::test(flavor = "multi_thread")]
async fn an_ask_opens_a_dependency_and_a_close_resolves_it() {
    let t = server(json!({})).await;
    let opened = asked(&t).await;
    assert_eq!(opened["dep"]["id"], "D-1");
    assert_eq!(opened["dep"]["state"], "open");
    assert_eq!(opened["dep"]["from"]["project"], "alpha");
    assert_eq!(opened["dep"]["to"]["project"], "beta");
    assert!(opened["text"].as_str().is_some_and(|l| l.contains("D-1")));
    assert_eq!(t.board("alpha")["items"]["I-1"]["waitsOn"]["dep"], "D-1");

    let closed = t
        .write(
            "POST",
            "/api/deps/D-1/close",
            &json!({ "as": "done", "actor": "beta/api" }),
        )
        .await;
    assert_eq!(closed.status, 200, "{}", closed.text());
    assert_eq!(closed.json()["dep"]["state"], "done");

    let waiting = t.get("/api/deps?project=alpha&view=waiting").await.json();
    assert_eq!(waiting["deps"][0]["id"], "D-1");
    assert_eq!(waiting["deps"][0]["state"], "done");
    let found = t.get("/api/deps/by-item/alpha/I-1").await.json();
    assert_eq!(found["dep"]["id"], "D-1");
    let shown = t.get("/api/deps/D-1?project=beta").await.json();
    assert_eq!(shown["dep"]["id"], "D-1");
    assert_eq!(shown["requester"]["id"], "I-1");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refused_call_answers_the_error_body_with_the_text_and_status_of_the_rule() {
    let t = server(json!({})).await;
    let same = t.write("POST", "/api/deps", &ask_body("alpha")).await;
    assert_eq!(same.status, 409);
    assert_eq!(same.json()["error"]["code"], "conflict");
    assert_eq!(
        same.json()["error"]["message"],
        "a dependency stays inside one project: use coord item add --for"
    );

    let nowhere = t.write("POST", "/api/deps", &ask_body("nowhere")).await;
    assert_eq!(nowhere.status, 404);
    let message = nowhere.json()["error"]["message"].clone();
    assert!(
        message
            .as_str()
            .is_some_and(|m| m.starts_with("nowhere is not a board here: known"))
    );

    asked(&t).await;
    let blocked = t
        .write(
            "POST",
            "/api/deps/D-1/close",
            &json!({ "as": "blocked", "actor": "beta/api" }),
        )
        .await;
    assert_eq!(blocked.status, 400);
    assert_eq!(
        blocked.json()["error"]["message"],
        "--as blocked needs --reason"
    );
    let unknown = t.get("/api/deps/D-9").await;
    assert_eq!(unknown.status, 404);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn every_route_answers_the_off_refusal_while_dependencies_are_off() {
    let t = server(json!({ "deps": { "enabled": false } })).await;
    let ask = t.write("POST", "/api/deps", &ask_body("beta")).await;
    assert_eq!(ask.status, 404);
    assert_eq!(ask.json()["error"]["message"], OFF);
    for path in ["/api/deps", "/api/deps/D-1", "/api/deps/by-item/alpha/I-1"] {
        let reply = t.get(path).await;
        assert_eq!(reply.status, 404, "{path}");
        assert_eq!(reply.json()["error"]["message"], OFF, "{path}");
    }
    assert!(t.board("alpha")["items"]["I-1"].get("waitsOn").is_none());
    t.stop(false).await;
}
