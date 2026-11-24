//! KEEP: K4, S2.
//! The stall watch over HTTP: a dependency request nobody reads becomes a stall in `GET /api/stalls` and in the
//! `_portfolio/stalls` docs the page streams; only the Overlord acknowledges it, never from a browser; an own inbox
//! read stamps the thread and acknowledges the dependency. The ladder itself is tested in `argus-wake`.
// test-tag: critical

use argus_core::ids::PORTFOLIO;
use argus_testkit::{
    Request, TestOptions, TestServer, Wakes, alpha_board, beta_board, make_home, write_board,
};
use serde_json::{Value, json};
use tempfile::TempDir;

const MIN_MS: i64 = 60_000;
const NOTE: &str = "looked into it, the thread was in a long build";

struct Stalled {
    t: TestServer,
    _home: TempDir,
}

fn overlord_board() -> Value {
    let mut board = beta_board();
    board["project"] = json!("portfolio");
    board
}

async fn server() -> (TestServer, TempDir) {
    let home = tempfile::tempdir().expect("coord home");
    make_home(home.path());
    write_board(home.path(), "alpha", &alpha_board(home.path()));
    write_board(home.path(), "portfolio", &overlord_board());
    let t = TestServer::start(TestOptions {
        coord_home: Some(home.path().to_path_buf()),
        wakes: Wakes::Manual,
        ..TestOptions::default()
    })
    .await;
    (t, home)
}

/// alpha/I-1 asks beta for something: the open dependency.
async fn asked(t: &TestServer) -> Value {
    let body = json!({
        "project": "alpha", "item": "I-1", "actor": "alpha/ui", "target": "beta",
        "title": "Expose the save hook",
    });
    let reply = t.write("POST", "/api/deps", &body).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()["dep"].clone()
}

/// The request is asked and nobody reads it for 11 minutes. The beta board is exported at once, so the wake engine
/// sees the item the request added instead of waiting for the file watcher.
async fn stalled() -> Stalled {
    let (t, home) = server().await;
    asked(&t).await;
    let resync = t
        .write("POST", "/api/admin/resync", &json!({ "project": "beta" }))
        .await;
    assert_eq!(resync.status, 200, "{}", resync.text());
    t.advance_wakes(MIN_MS);
    t.advance_wakes(11 * MIN_MS);
    Stalled { t, _home: home }
}

async fn stalls_of(t: &TestServer, project: &str) -> Value {
    t.get(&format!("/api/stalls?project={project}"))
        .await
        .json()
}

fn unread(list: &Value) -> Option<Value> {
    list["stalls"]
        .as_array()?
        .iter()
        .find(|s| s["kind"] == "unread")
        .cloned()
}

fn ack_body(by: &str) -> Value {
    json!({ "project": "beta", "note": NOTE, "by": by })
}

async fn ack(t: &TestServer, body: &Value) -> (u16, Value) {
    let reply = t.write("POST", "/api/stalls/ack", body).await;
    (reply.status, reply.json())
}

fn page_docs(t: &TestServer) -> Vec<Value> {
    let rows = t.svc.db.docs().list(PORTFOLIO, argus_core::Coll::Stalls);
    rows.expect("stall docs")
        .into_iter()
        .map(|row| row.data)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn an_unread_request_is_a_stall_for_the_cli_and_a_doc_for_the_page() {
    let s = stalled().await;
    let list = stalls_of(&s.t, "beta").await;
    let stall = unread(&list).expect("an unread stall of beta");
    assert_eq!(stall["project"], "beta");
    assert_eq!(stall["ackedAt"], Value::Null);
    let text = stall["text"].as_str().expect("text");
    assert!(text.starts_with("stall: "), "{text}");
    assert!(list["text"].as_str().is_some_and(|t| t.contains(text)));
    assert_eq!(
        unread(&stalls_of(&s.t, "alpha").await),
        None,
        "alpha has none"
    );

    let doc = page_docs(&s.t)
        .into_iter()
        .find(|d| d["id"] == stall["id"])
        .expect("the stall doc reaches _portfolio/stalls");
    assert_eq!(doc["kind"], "unread");
    let route = s.t.get("/api/p/_portfolio/db/stalls").await.json();
    assert!(
        route["docs"]
            .as_array()
            .is_some_and(|docs| docs.iter().any(|d| d["id"] == stall["id"])),
        "the page reads it through the db route"
    );
    assert_eq!(s.t.get("/api/stalls?project=Bad%20Key").await.status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn stall_ack_is_refused_from_the_page_and_for_anyone_but_the_overlord() {
    let s = stalled().await;
    let t = &s.t;
    let page = Request::new("POST", "/api/stalls/ack").json(&ack_body("lead"));
    let refused = t.send(t.page(page)).await;
    assert_eq!(refused.status, 403);
    assert!(
        refused.json()["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("cannot be acknowledged from the page"))
    );
    let meta = Request::new("POST", "/api/stalls/ack")
        .json(&ack_body("lead"))
        .header("Sec-Fetch-Site", "same-origin");
    assert_eq!(t.send(t.bearer(meta)).await.status, 403);
    for by in ["api", "nobody", ""] {
        let (status, body) = ack(t, &ack_body(by)).await;
        assert_eq!(status, 403, "by \"{by}\"");
        assert_eq!(body["error"]["code"], "forbidden");
    }
    let mut short = ack_body("lead");
    short["note"] = json!("too");
    assert_eq!(ack(t, &short).await.0, 400, "a short note");
    let mut nowhere = ack_body("lead");
    nowhere["project"] = json!("Bad Key");
    assert_eq!(ack(t, &nowhere).await.0, 400, "not a project key");

    let stall = unread(&stalls_of(t, "beta").await).expect("still open");
    assert_eq!(stall["ackedAt"], Value::Null, "nothing was acknowledged");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_overlord_acknowledges_every_open_stall_of_a_project_and_the_page_docs_follow() {
    let s = stalled().await;
    let t = &s.t;
    let (status, done) = ack(t, &ack_body("lead")).await;
    assert_eq!(status, 200, "{done}");
    let acked = done["acked"].as_array().expect("acked stalls");
    assert_ne!(acked.len(), 0);
    assert!(acked.iter().all(|a| a["project"] == "beta"));
    let text = done["text"].as_str().expect("text");
    assert!(
        text.starts_with("acked ") && text.ends_with(" of beta"),
        "{text}"
    );

    let stall = unread(&stalls_of(t, "beta").await).expect("it stays until it clears");
    assert_eq!(stall["ackedBy"], "portfolio/lead");
    assert_eq!(stall["ackNote"], NOTE);
    let doc = page_docs(t)
        .into_iter()
        .find(|d| d["id"] == stall["id"])
        .expect("the doc");
    assert_eq!(doc["ackNote"], NOTE, "the page sees the ack at once");

    let (again, body) = ack(t, &ack_body("lead")).await;
    assert_eq!(again, 409, "nothing left to acknowledge");
    assert_eq!(body["error"]["message"], "beta has no open stall");
    let mut other = ack_body("lead");
    other["project"] = json!("alpha");
    assert_eq!(ack(t, &other).await.0, 409, "alpha has no open stall");
}

#[tokio::test(flavor = "multi_thread")]
async fn inbox_read_acks_the_dep_and_stamps_the_thread() {
    let (t, _home) = server().await;
    let dep = asked(&t).await;
    let thread = dep["to"]["thread"].as_str().expect("the assignee thread");
    let path = "/api/p/beta/inbox/read";

    let alpha = json!({ "thread": "ui", "deps": ["D-1"], "actor": "alpha/ui" });
    let from_alpha = t.write("POST", "/api/p/alpha/inbox/read", &alpha).await;
    assert_eq!(from_alpha.status, 200);
    assert_eq!(from_alpha.json()["acked"], json!([]), "not asked of alpha");

    let ghost = json!({ "thread": "ghost", "deps": ["D-1"] });
    assert_eq!(t.write("POST", path, &ghost).await.status, 404);
    assert_eq!(t.write("POST", path, &json!({})).await.status, 400);

    let body = json!({ "thread": thread, "deps": ["D-1", "D-99"], "actor": "beta/api" });
    let read = t.write("POST", path, &body).await;
    assert_eq!(read.status, 200, "{}", read.text());
    assert_eq!(read.json()["acked"], json!(["D-1"]));
    assert!(read.json()["readAt"].is_string());
    let shown = t.get("/api/deps/D-1?project=beta").await.json();
    assert!(shown["dep"]["ackedAt"].is_string());
    assert_eq!(shown["dep"]["ackedBy"], "beta/api");
    let stamp = t.svc.db.kv_get(&format!("inbox-read:beta:{thread}"));
    assert!(stamp.expect("kv").is_some(), "the read is stamped");

    let twice = t.write("POST", path, &body).await;
    assert_eq!(twice.json()["acked"], json!([]), "acknowledged once");
}
