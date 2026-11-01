//! The stdin route of a run, with the refusals a terminal row can meet: page and CLI callers
//! both type, an empty line is a valid Enter, 4 KB is the inclusive limit and a body far over it
//! is refused unread, JSON only, GET is not a way to type, and an ended run, a run without stdin,
//! an unknown run and a bad run id are refused with their codes. Ported from the stdin route
//! case of `runs.terminal.test.mjs`.

use crate::runs_rig::Rig;
use argus_testkit::{Reply, Request};
use serde_json::{Value, json};

const PATH: &str = "/api/p/alpha/runs/R-1/stdin";

fn error_field(reply: &Reply, field: &str) -> String {
    reply.json()["error"][field]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

async fn type_text(r: &Rig, body: &Value) -> Reply {
    r.page("POST", PATH, body).await
}

async fn term() -> (Rig, u32) {
    let r = Rig::start().await;
    r.define(&[("term", json!({ "kind": "launch" })), ("plain", json!({}))]);
    let run = r.begin("term").await;
    let pid = u32::try_from(run["pid"].as_u64().expect("a pid")).expect("a pid fits");
    (r, pid)
}

#[tokio::test(flavor = "multi_thread")]
async fn page_and_cli_lines_reach_the_process_and_an_empty_line_is_a_valid_enter() {
    let (r, pid) = term().await;
    let from_page = type_text(&r, &json!({ "text": "from page" })).await;
    assert_eq!(from_page.status, 200);
    assert_eq!(from_page.json()["ok"], true);
    let from_cli = r.cli("POST", PATH, &json!({ "text": "from cli" })).await;
    assert_eq!(from_cli.status, 200);
    assert_eq!(type_text(&r, &json!({ "text": "" })).await.status, 200);
    assert_eq!(r.host.stdin_text(pid).await, "from page\nfrom cli\n\n");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_missing_or_wrong_text_and_a_body_that_is_not_an_object_are_refused() {
    let (r, _) = term().await;
    let missing = type_text(&r, &json!({})).await;
    assert_eq!(missing.status, 400);
    assert_eq!(error_field(&missing, "code"), "bad_request");
    assert_eq!(type_text(&r, &json!({ "text": 5 })).await.status, 400);
    assert_eq!(type_text(&r, &json!([])).await.status, 400);
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn four_kb_is_the_inclusive_limit_and_a_body_far_over_it_is_refused() {
    let (r, _) = term().await;
    let exact = type_text(&r, &json!({ "text": "x".repeat(4096) })).await;
    assert_eq!(exact.status, 200, "4 KB is the limit, inclusive");
    let big = type_text(&r, &json!({ "text": "x".repeat(4097) })).await;
    assert_eq!(big.status, 413);
    assert_eq!(error_field(&big, "code"), "too_large");
    let far = type_text(&r, &json!({ "text": "x".repeat(40_000) })).await;
    assert_eq!(far.status, 413, "refused before the body is read");
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn only_json_is_taken_and_a_get_is_not_a_way_to_type() {
    let (r, _) = term().await;
    let plain = Request::new("POST", PATH)
        .header("Content-Type", "text/plain")
        .body(b"text=hello".to_vec());
    assert_eq!(r.t.send(r.t.page(plain)).await.status, 415, "JSON only");
    assert_eq!(
        r.t.get(PATH).await.status,
        405,
        "refused; the Node route said 404"
    );
    r.t.stop(false).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_run_a_bad_run_id_a_run_without_stdin_and_an_ended_run_are_refused() {
    let (r, pid) = term().await;
    let line = json!({ "text": "a" });
    let unknown = r.page("POST", "/api/p/alpha/runs/R-99/stdin", &line).await;
    assert_eq!(unknown.status, 404);
    let bad = r.page("POST", "/api/p/alpha/runs/nope/stdin", &line).await;
    assert_eq!(bad.status, 400);
    let plain = r.begin("plain").await;
    assert_eq!(plain["stdin"], false, "a task has no stdin pipe");
    let none = r.page("POST", "/api/p/alpha/runs/R-2/stdin", &line).await;
    assert_eq!(none.status, 409);
    assert!(error_field(&none, "message").contains("no stdin"));
    r.host.exit(pid, Some(0)).await;
    r.until_status("R-1", "exited").await;
    let late = type_text(&r, &json!({ "text": "late" })).await;
    assert_eq!(late.status, 409);
    assert!(error_field(&late, "message").contains("not running"));
    r.t.stop(false).await;
}
