//! The MCP layer on the real server with the fixture boards: the auth, Host, Origin and content
//! type rules of `/mcp`, the tools against live boards, owner writes landing on `board.json`,
//! hidden runs started and read back, the policy of `config.json`, and the `notify_owner` tool.
//! Ported from `mcp.server.test.mjs`, the auth case of `mcp.http.test.mjs` and the MCP case of
//! `notify.http.test.mjs`. The protocol itself is tested in `argus-mcp`.
//! K2, K4: the MCP layer on a live server: auth and refusals, tool results, owner writes and the policy.
// test-tag: standard

use std::sync::{Arc, PoisonError};

use crate::toasts::Toasts;
use crate::wait::until;
use argus_core::config::McpConfig;
use argus_launchers::FakeHost;
use argus_native::Notifier;
use argus_server::ServeOptions;
use argus_testkit::{Reply, Request, TestOptions, TestServer, png};
use base64::Engine;
use serde_json::{Map, Value, json};

fn ping() -> Value {
    json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" })
}

async fn start_with(tweak: impl Fn(&mut ServeOptions) + Send + Sync + 'static) -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        tweak: Some(Arc::new(tweak)),
        ..TestOptions::default()
    })
    .await
}

async fn seeded() -> TestServer {
    start_with(|_| {}).await
}

async fn rpc_with(t: &TestServer, msg: &Value, headers: &[(&str, &str)]) -> Reply {
    let mut req = Request::new("POST", "/mcp").json(msg);
    for (name, value) in headers {
        req = req.header(name, value);
    }
    t.send(req).await
}

async fn rpc(t: &TestServer, msg: &Value) -> Reply {
    rpc_with(t, msg, &[]).await
}

/// One tool call: the structured result, with `$error` set to `isError`.
async fn tool_with(t: &TestServer, name: &str, args: Value, headers: &[(&str, &str)]) -> Value {
    let msg = json!({ "jsonrpc": "2.0", "id": 7, "method": "tools/call",
        "params": { "name": name, "arguments": args } });
    let body = rpc_with(t, &msg, headers).await.json();
    let result = body["result"].clone();
    assert!(result.is_object(), "{body}");
    let mut out = result["structuredContent"].clone();
    out["$error"] = result["isError"].clone();
    out
}

async fn tool(t: &TestServer, name: &str, args: Value) -> Value {
    tool_with(t, name, args, &[]).await
}

async fn read(t: &TestServer, uri: &str) -> Value {
    let msg =
        json!({ "jsonrpc": "2.0", "id": 3, "method": "resources/read", "params": { "uri": uri } });
    let body = rpc(t, &msg).await.json();
    serde_json::from_str(
        body["result"]["contents"][0]["text"]
            .as_str()
            .expect("text"),
    )
    .expect("json text")
}

fn ids(list: &Value, field: &str) -> Vec<String> {
    let rows = list[field].as_array().cloned().unwrap_or_default();
    rows.iter()
        .map(|r| r["id"].as_str().unwrap_or_default().to_owned())
        .collect()
}

fn set_policy(t: &TestServer, policy: McpConfig) {
    t.svc
        .config
        .write()
        .unwrap_or_else(PoisonError::into_inner)
        .mcp = policy;
}

#[tokio::test(flavor = "multi_thread")]
async fn no_credential_is_needed_a_wrong_bearer_is_401_and_a_form_post_is_415() {
    let t = seeded().await;
    assert_eq!(rpc(&t, &ping()).await.status, 200);
    let wrong = rpc_with(&t, &ping(), &[("Authorization", "Bearer nope")]).await;
    assert_eq!(wrong.status, 401);
    assert_eq!(wrong.code(), "unauthorized");
    let form = Request::new("POST", "/mcp")
        .header("Content-Type", "text/plain")
        .body(ping().to_string());
    assert_eq!(t.send(form).await.status, 415);
    let right = t.bearer(Request::new("POST", "/mcp").json(&ping()));
    assert_eq!(t.send(right).await.status, 200);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bad_host_is_421_a_foreign_origin_403_and_get_and_delete_are_405() {
    let t = seeded().await;
    let host = Request::new("POST", "/mcp")
        .header("Host", "evil.test")
        .json(&ping());
    let refused = t.send(host).await;
    assert_eq!(refused.status, 421, "{}", refused.text());
    let origin = Request::new("POST", "/mcp")
        .header("Origin", "http://evil.test")
        .json(&ping());
    let refused = t.send(origin).await;
    assert_eq!(refused.status, 403, "{}", refused.text());
    for method in ["GET", "DELETE"] {
        let req = Request::new(method, "/mcp").header("Content-Type", "application/json");
        let r = t.send(t.bearer(req)).await;
        assert_eq!(r.status, 405, "{method}");
        assert_eq!(r.header("allow"), Some("POST"), "{method}");
        assert_eq!(r.code(), "bad_request");
    }
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_notification_is_202_and_a_big_body_is_413() {
    let t = seeded().await;
    let note = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    let r = rpc(&t, &note).await;
    assert_eq!((r.status, r.body.len()), (202, 0));
    let pad = "x".repeat(2 * 1024 * 1024 + 10);
    let big = json!({ "jsonrpc": "2.0", "id": 3, "method": "ping", "params": { "pad": pad } });
    assert_eq!(rpc(&t, &big).await.status, 413);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn initialize_and_tools_list_answer_on_the_live_server() {
    let t = seeded().await;
    let init = json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {},
        "clientInfo": { "name": "it", "version": "0" } } });
    let result = rpc(&t, &init).await.json()["result"].clone();
    assert_eq!(result["serverInfo"]["name"], "coord-dashboard");
    let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });
    let tools = rpc(&t, &list).await.json()["result"]["tools"].clone();
    assert_eq!(tools.as_array().map(Vec::len), Some(25));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn reads_overview_summary_and_items() {
    let t = seeded().await;
    assert_eq!(tool(&t, "health", json!({})).await["ok"], true);
    let cards = tool(&t, "portfolio_overview", json!({})).await;
    let mut keys: Vec<String> = cards["cards"]
        .as_array()
        .expect("cards")
        .iter()
        .map(|c| {
            c["id"]
                .as_str()
                .or(c["key"].as_str())
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    keys.sort();
    assert_eq!(keys, ["alpha", "beta"]);
    let sum = tool(&t, "board_summary", json!({ "project": "alpha" })).await;
    assert_eq!(sum["project"], "alpha");
    assert_eq!(sum["handedRun"]["id"], "I-21");
    assert_eq!(sum["handedRun"]["build"], "build-1");
    assert!(sum.get("handedBuild").is_none());
    assert_eq!(sum["openQuestions"], json!(["Q-1"]));
    assert_eq!(sum["items"].as_array().map(Vec::len), Some(4));
    assert!(sum.get("artifact").is_none());
    let doing = tool(
        &t,
        "list_items",
        json!({ "project": "alpha", "status": "doing" }),
    )
    .await;
    assert_eq!(ids(&doing, "items"), ["I-1", "I-20", "I-21"]);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn reads_item_tests_agents_and_entries() {
    let t = seeded().await;
    let item = tool(&t, "get_item", json!({ "project": "alpha", "id": "I-1" })).await;
    assert_eq!(item["item"]["title"], "Login screen");
    assert_eq!(ids(&item, "tests"), ["T-1", "T-2"]);
    assert!(ids(&item, "entries").contains(&"Q-1".to_owned()));
    let manual = tool(
        &t,
        "list_tests",
        json!({ "project": "alpha", "mode": "manual" }),
    )
    .await;
    assert_eq!(ids(&manual, "tests"), ["T-2"]);
    let test = tool(&t, "get_test", json!({ "project": "alpha", "id": "T-2" })).await;
    assert_eq!(test["test"]["title"], "Bad password is refused");
    let live = tool(
        &t,
        "list_agents",
        json!({ "project": "alpha", "live": true }),
    )
    .await;
    assert_eq!(ids(&live, "agents"), ["A-1"]);
    let open = tool(
        &t,
        "list_entries",
        json!({ "project": "alpha", "open": true }),
    )
    .await;
    assert!(ids(&open, "entries").contains(&"Q-1".to_owned()));
    let none = tool(&t, "get_item", json!({ "project": "nope", "id": "I-1" })).await;
    assert_eq!(none["error"]["code"], "not_found");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn search_and_the_run_definitions_read_the_live_boards() {
    let t = seeded().await;
    let hits = tool(&t, "search", json!({ "query": "login" })).await;
    let found = hits["hits"].as_array().expect("hits");
    assert!(
        found
            .iter()
            .any(|h| h["project"] == "alpha" && h["id"] == "I-1")
    );
    let status = tool(&t, "run_status", json!({ "project": "alpha" })).await;
    let mut names: Vec<String> = status["defs"]
        .as_array()
        .expect("defs")
        .iter()
        .map(|d| d["name"].as_str().unwrap_or_default().to_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["app", "fail", "hello", "slow"]);
    assert!(
        status["defs"]
            .as_array()
            .expect("defs")
            .iter()
            .all(|d| d.get("env").is_none())
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn resources_read_the_same_data() {
    let t = seeded().await;
    assert_eq!(read(&t, "board://alpha/summary").await["project"], "alpha");
    let to_run = read(&t, "board://alpha/tests/to-run").await;
    let groups = to_run["groups"].as_array().expect("groups");
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["run"], "I-21");
    assert_eq!(groups[0]["build"], "build-1");
    assert_eq!(ids(&groups[0], "tests"), ["T-2"]);
    assert_eq!(read(&t, "runs://alpha/active").await["runs"], json!([]));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn answer_applies_stamps_the_owner_and_the_client_and_reaches_the_inbox() {
    let t = seeded().await;
    let args = json!({ "project": "alpha", "entry": "Q-1", "text": "Yes, remember the user." });
    let r = tool(&t, "answer", args).await;
    assert_eq!(r["status"], "applied", "{r}");
    assert_eq!(r["entry"]["author"], "human");
    assert!(
        r["entry"]["via"]
            .as_str()
            .is_some_and(|v| v.starts_with("mcp:"))
    );
    assert_eq!(t.board("alpha")["entries"]["Q-1"]["status"], "answered");
    let ui = t.board("alpha")["inbox"]["ui"].clone();
    assert!(
        ui.as_array()
            .expect("ui inbox")
            .iter()
            .any(|m| m["ref"] == r["id"])
    );
    let live = t.get("/api/p/alpha/db/entries/Q-1").await.json();
    assert_eq!(live["data"]["status"], "answered");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn record_test_result_and_decision_land_on_the_boards() {
    let t = seeded().await;
    let args =
        json!({ "project": "alpha", "test": "T-2", "outcome": "pass", "text": "error shows" });
    let r = tool(&t, "record_test_result", args).await;
    assert_eq!(r["status"], "applied", "{r}");
    assert_eq!(t.board("alpha")["tests"]["T-2"]["status"], "pass");
    let d = tool(
        &t,
        "decision",
        json!({ "project": "beta", "text": "Use JSON", "item": "I-1" }),
    )
    .await;
    assert_eq!(d["status"], "pending", "{d}");
    let entries = t.board("beta")["entries"].clone();
    let found = entries.as_object().expect("entries").values();
    assert!(
        found
            .into_iter()
            .any(|e| e["kind"] == "decision" && e["text"] == "Use JSON")
    );
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn send_to_lead_delivers_to_the_project_lead_inbox() {
    let t = seeded().await;
    let r = tool(
        &t,
        "send_to_lead",
        json!({ "project": "beta", "text": "Please prioritise the API" }),
    )
    .await;
    assert_eq!(r["status"], "delivered", "{r}");
    let lead = t.board("beta")["inbox"]["coordinator"].clone();
    let sent = lead.as_array().expect("lead inbox");
    assert!(sent.iter().any(|m| {
        m["text"]
            .as_str()
            .is_some_and(|x| x.contains("Please prioritise the API"))
    }));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn attach_proof_keeps_the_picture_on_the_test_and_returns_a_url_that_serves_it() {
    let t = seeded().await;
    let base64 = base64::engine::general_purpose::STANDARD.encode(png());
    let args =
        json!({ "project": "alpha", "test": "T-2", "caption": "the error", "base64": base64 });
    let r = tool(&t, "attach_proof", args).await;
    assert_eq!(r["$error"], false, "{r}");
    let url = r["url"].as_str().expect("url");
    let prefix = format!("http://127.0.0.1:{}/files/alpha/proofs/T-2-", t.port);
    let is_png = std::path::Path::new(url)
        .extension()
        .is_some_and(|e| e == "png");
    assert!(url.starts_with(&prefix) && is_png, "{url}");
    assert_eq!(
        t.board("alpha")["tests"]["T-2"]["proofs"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
    let img = t
        .get(
            url.strip_prefix(&format!("http://127.0.0.1:{}", t.port))
                .expect("path"),
        )
        .await;
    assert_eq!(img.status, 200);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_default_project_and_client_headers_reach_the_tools() {
    let t = seeded().await;
    let headers = [
        ("X-Coord-Default-Project", "beta"),
        ("X-Coord-Client", "claude-code"),
    ];
    let items = tool_with(&t, "list_items", json!({}), &headers).await;
    assert_eq!(items["items"][0]["title"], "Health endpoint");
    let d = tool_with(&t, "decision", json!({ "text": "go" }), &headers).await;
    assert_eq!(d["status"], "pending", "{d}");
    assert_eq!(d["entry"]["via"], "mcp:claude-code");
    let none = tool(&t, "list_items", json!({})).await;
    assert_eq!(none["error"]["code"], "bad_request");
    t.stop(false).await;
}

struct Fake {
    t: TestServer,
    host: FakeHost,
}

async fn with_fake_host() -> Fake {
    let host = FakeHost::new();
    let wired = host.clone();
    let t = start_with(move |opts| opts.run_service.host = Some(wired.clone())).await;
    Fake { t, host }
}

async fn status_of(t: &TestServer, rid: &str) -> Value {
    tool(t, "run_status", json!({ "project": "alpha", "run": rid })).await["run"].clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn run_start_is_hidden_finishes_and_run_logs_shows_its_output() {
    let Fake { t, host } = with_fake_host().await;
    let args = json!({ "project": "alpha", "name": "hello", "reason": "mcp test" });
    let started = tool(&t, "run_start", args).await;
    assert_eq!(started["$error"], false, "{started}");
    let run = &started["run"];
    assert_eq!(run["visible"], false);
    assert_eq!(run["origin"], "mcp");
    assert!(run["by"].as_str().is_some_and(|b| b.starts_with("mcp:")));
    let rid = run["id"].as_str().expect("run id").to_owned();
    let pid = u32::try_from(run["pid"].as_u64().expect("pid")).expect("pid");
    host.say(pid, "hi\n").await;
    host.exit(pid, Some(0)).await;
    let done = until("the run to exit", || async {
        let run = status_of(&t, &rid).await;
        (run["status"] == "exited").then_some(run)
    })
    .await;
    assert_eq!(done["status"], "exited");
    let logs = tool(&t, "run_logs", json!({ "project": "alpha", "run": rid })).await;
    let lines = logs["lines"].as_array().expect("lines");
    assert!(
        lines
            .iter()
            .any(|l| l["text"].as_str().is_some_and(|x| x.contains("hi"))),
        "{logs}"
    );
    assert_eq!(logs["status"], "exited");
    let unknown = tool(
        &t,
        "run_start",
        json!({ "project": "alpha", "name": "not-defined" }),
    )
    .await;
    assert_eq!(unknown["$error"], true);
    assert_eq!(unknown["error"]["code"], "unknown_run");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn run_stop_stops_a_live_run() {
    let Fake { t, host } = with_fake_host().await;
    let started = tool(
        &t,
        "run_start",
        json!({ "project": "alpha", "name": "slow" }),
    )
    .await;
    let rid = started["run"]["id"].as_str().expect("run id").to_owned();
    let stopped = tool(&t, "run_stop", json!({ "project": "alpha", "run": rid })).await;
    assert_eq!(stopped["$error"], false, "{stopped}");
    until("the run to be stopped", || async {
        (status_of(&t, &rid).await["status"] == "stopped").then_some(())
    })
    .await;
    assert_eq!(host.killed().len(), 1);
    t.stop(false).await;
}

fn read_only_policy() -> McpConfig {
    McpConfig {
        allow: json!("*"),
        deny: Vec::new(),
        read_only: true,
        extra: Map::new(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn read_only_in_the_config_refuses_a_write_on_the_live_server() {
    let t = seeded().await;
    let saved = t.svc.config().mcp;
    set_policy(&t, read_only_policy());
    let r = tool(
        &t,
        "send_to_lead",
        json!({ "project": "beta", "text": "blocked" }),
    )
    .await;
    assert_eq!(r["$error"], true);
    assert!(
        r["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("config.mcp.readOnly"))
    );
    assert_eq!(tool(&t, "health", json!({})).await["ok"], true);
    set_policy(&t, saved);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn shadow_mode_refuses_an_owner_write_with_read_only() {
    let t = TestServer::start(TestOptions {
        seed: true,
        shadow: true,
        ..TestOptions::default()
    })
    .await;
    let r = tool(
        &t,
        "send_to_lead",
        json!({ "project": "beta", "text": "no" }),
    )
    .await;
    assert_eq!(r["$error"], true);
    assert_eq!(r["error"]["code"], "read_only");
    assert_eq!(tool(&t, "health", json!({})).await["shadow"], true);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn notify_owner_is_validated_recorded_as_the_mcp_client_and_shows_a_toast() {
    let toasts = Arc::new(Toasts::default());
    let given: Arc<dyn Notifier> = toasts.clone();
    let t = start_with(move |opts| opts.notify.notifier = Some(Arc::clone(&given))).await;
    let client = [("X-Coord-Client", "Claude Desktop")];
    let args = json!({ "text": "ready for you", "title": "From an agent", "project": "alpha" });
    let r = tool_with(&t, "notify_owner", args, &client).await;
    assert_eq!(r["$error"], false, "{r}");
    assert_eq!(
        (&r["recorded"], &r["toast"], &r["level"]),
        (&json!(true), &json!("shown"), &json!("ping"))
    );
    let listed = t.get("/api/notify").await.json();
    let n = &listed["notifications"][0];
    assert_eq!(
        [&n["by"], &n["thread"], &n["project"], &n["title"]],
        [
            &json!("thread"),
            &json!("mcp:Claude Desktop"),
            &json!("alpha"),
            &json!("From an agent")
        ]
    );
    assert_eq!(toasts.count(), 1);
    let info = tool_with(
        &t,
        "notify_owner",
        json!({ "text": "fyi", "level": "info" }),
        &client,
    )
    .await;
    assert_eq!(info["toast"], "skipped");
    let long = tool(&t, "notify_owner", json!({ "text": "x".repeat(241) })).await;
    assert_eq!(long["error"]["code"], "bad_request");
    assert_eq!(
        tool(&t, "notify_owner", json!({ "level": "ping" })).await["error"]["code"],
        "bad_request"
    );
    let loud = tool(&t, "notify_owner", json!({ "text": "x", "level": "loud" })).await;
    assert_eq!(loud["error"]["code"], "bad_request");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_health_tool_answers_what_the_health_route_does() {
    let t = seeded().await;
    let route = t.get("/api/health").await.json();
    let tool = tool(&t, "health", json!({})).await;
    for field in ["ok", "version", "instance", "port", "scopes", "impl"] {
        assert_eq!(tool[field], route[field], "{field}");
    }
    t.stop(false).await;
}
