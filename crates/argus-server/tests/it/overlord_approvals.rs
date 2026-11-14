// test-tag: critical
//! The approvals of the hosted Overlord over HTTP: the private endpoint answers only the bearer of
//! the running child (K4), a held call waits for the owner and gets the documented reply (K2), and
//! the tab's routes list and decide (S2). The toast goes to a recording notifier.

use std::sync::Arc;
use std::time::Duration;

use argus_native::Notifier;
use argus_overlord::approvals::Prompt;
use argus_server::ServeOptions;
use argus_server::adapters::overlord::gate::Ask;
use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::{Value, json};

use crate::toasts::Toasts;

async fn start(toasts: &Arc<Toasts>) -> TestServer {
    let toasts: Arc<dyn Notifier> = toasts.clone();
    TestServer::start(TestOptions {
        tweak: Some(Arc::new(move |opts: &mut ServeOptions| {
            opts.notify.notifier = Some(Arc::clone(&toasts));
        })),
        ..TestOptions::default()
    })
    .await
}

/// A bearer made as a child start makes it, read back from the temp file the child would get.
fn begin(t: &TestServer) -> (String, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp folder");
    let endpoint = format!("{}/mcp/overlord", t.url);
    let file = t
        .svc
        .overlord_approvals
        .begin(&endpoint, dir.path())
        .expect("mcp config");
    let config: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let header = &config["mcpServers"]["argus-overlord"]["headers"]["Authorization"];
    (header.as_str().unwrap().to_owned(), dir)
}

fn rpc(body: &Value) -> Request {
    Request::new("POST", "/mcp/overlord").json(body)
}

fn call(tool: &str, input: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": 9, "method": "tools/call", "params": {
        "name": "permission_prompt",
        "arguments": { "tool_name": tool, "input": input, "tool_use_id": "tu1" }
    } })
}

fn reply_of(body: &Value) -> Value {
    let text = body["result"]["content"][0]["text"].as_str().expect("text");
    serde_json::from_str(text).expect("reply json")
}

async fn answer_when_pending(t: &TestServer, decision: &Value) {
    loop {
        let list = t
            .send(t.page(Request::new("GET", "/api/overlord/approvals")))
            .await;
        let pending = list.json()["approvals"]
            .as_array()
            .and_then(|a| a.iter().find(|x| x["status"] == "pending").cloned());
        if let Some(item) = pending {
            let path = format!("/api/overlord/approvals/{}", item["id"].as_str().unwrap());
            let sent = t
                .send(t.page(Request::new("POST", &path).json(decision)))
                .await;
            assert_eq!(sent.status, 200, "{}", sent.text());
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_call_without_the_childs_bearer_is_401() {
    let toasts = Arc::new(Toasts::default());
    let t = start(&toasts).await;
    let list = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    assert_eq!(t.send(rpc(&list)).await.status, 401, "no child yet");
    let (header, _dir) = begin(&t);
    assert_eq!(t.send(rpc(&list)).await.status, 401, "no header");
    let dashboard = t.send(t.bearer(rpc(&list))).await;
    assert_eq!(dashboard.status, 401, "the dashboard token is not the child's");
    let wrong = t.send(rpc(&list).header("Authorization", "Bearer nope")).await;
    assert_eq!(wrong.status, 401);
    let right = t.send(rpc(&list).header("Authorization", &header)).await;
    assert_eq!(right.status, 200, "{}", right.text());
    assert_eq!(
        right.json()["result"]["tools"][0]["name"],
        json!("permission_prompt")
    );
    t.svc.overlord_approvals.end();
    let after = t.send(rpc(&list).header("Authorization", &header)).await;
    assert_eq!(after.status, 401, "the bearer dies with the child");
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_config_file_is_gone_when_the_child_ends() {
    let toasts = Arc::new(Toasts::default());
    let t = start(&toasts).await;
    let dir = tempfile::tempdir().unwrap();
    let file = t
        .svc
        .overlord_approvals
        .begin("http://127.0.0.1:1/mcp/overlord", dir.path())
        .unwrap();
    assert!(file.exists());
    t.svc.overlord_approvals.end();
    assert!(!file.exists());
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn approve_lets_the_call_run_and_the_toast_links_the_tab() {
    let toasts = Arc::new(Toasts::default());
    let t = start(&toasts).await;
    let (header, _dir) = begin(&t);
    let input = json!({ "command": "git push" });
    let held = t.send(rpc(&call("Bash", &input)).header("Authorization", &header));
    let (reply, ()) = tokio::join!(held, answer_when_pending(&t, &json!({ "decision": "allow" })));
    assert_eq!(reply.status, 200, "{}", reply.text());
    assert_eq!(
        reply_of(&reply.json()),
        json!({ "behavior": "allow", "updatedInput": { "command": "git push" } })
    );
    let toast = toasts.last();
    assert!(toast.text.contains("git push"));
    assert!(toast.url.unwrap().contains("tab=overlord&approval=ap-"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn deny_returns_the_owners_message() {
    let toasts = Arc::new(Toasts::default());
    let t = start(&toasts).await;
    let (header, _dir) = begin(&t);
    let held = t.send(rpc(&call("Bash", &json!({ "command": "rm x" }))).header("Authorization", &header));
    let no = json!({ "decision": "deny", "message": "not today" });
    let (reply, ()) = tokio::join!(held, answer_when_pending(&t, &no));
    assert_eq!(
        reply_of(&reply.json()),
        json!({ "behavior": "deny", "message": "not today" })
    );
    let list = t.send(t.page(Request::new("GET", "/api/overlord/approvals"))).await;
    assert_eq!(list.json()["approvals"][0]["status"], json!("denied"));
    assert_eq!(list.json()["approvals"][0]["decidedBy"], json!("owner"));
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn no_answer_in_time_is_denied_and_a_late_answer_is_a_conflict() {
    let toasts = Arc::new(Toasts::default());
    let t = start(&toasts).await;
    let ask = Ask {
        prompt: Prompt {
            id: json!(1),
            tool_name: "Bash".to_owned(),
            input: json!({ "command": "ls" }),
            tool_use_id: None,
        },
        wait_ms: 50,
        link_base: t.url.clone(),
    };
    let sink = Arc::new(argus_server::adapters::overlord::approvals::NotifyToast(None));
    let reply = t.svc.overlord_approvals.ask(&ask, sink).await;
    assert_eq!(
        reply,
        json!({ "behavior": "deny", "message": "the owner did not answer" })
    );
    let id = t.svc.overlord_approvals.recent(1)[0].id.clone();
    let late = Request::new("POST", &format!("/api/overlord/approvals/{id}"))
        .json(&json!({ "decision": "allow" }));
    assert_eq!(t.send(t.page(late)).await.status, 409);
    t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_tab_routes_are_for_the_page_only_and_check_their_input() {
    let toasts = Arc::new(Toasts::default());
    let t = start(&toasts).await;
    let list = Request::new("GET", "/api/overlord/approvals");
    assert_eq!(t.send(t.bearer(list)).await.status, 403);
    let decide = || Request::new("POST", "/api/overlord/approvals/ap-0-1");
    let body = json!({ "decision": "allow" });
    assert_eq!(t.send(t.bearer(decide().json(&body))).await.status, 403);
    assert_eq!(t.send(decide().json(&body)).await.status, 403, "no origin");
    let bad = json!({ "decision": "maybe" });
    assert_eq!(t.send(t.page(decide().json(&bad))).await.status, 400);
    assert_eq!(t.send(t.page(decide().json(&body))).await.status, 404);
    t.stop(false).await;
}
