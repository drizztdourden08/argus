//! The watcher: a board saved by another process (the CLI) reaches the page stream within a
//! second, an owner post produces one change on the stream and no echo when the watcher then
//! reads the board the server just saved, and a board that appears after start is mirrored
//! (port of `sync.watch.test.mjs`; the CLI save is simulated, no Node runs).

use std::time::{Duration, Instant};

use argus_board::store::empty_board;
use argus_core::clock::now_iso;
use argus_testkit::{Event, Request, TestOptions, TestServer, write_board};
use serde_json::{Value, json};

async fn seeded() -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        ..TestOptions::default()
    })
    .await
}

fn has_item(title: &'static str) -> impl Fn(&Event) -> bool {
    move |e| {
        e.event == "changes"
            && e.data["changes"].as_array().is_some_and(|cs| {
                cs.iter().any(|c| {
                    c["coll"] == "items" && c["op"] == "set" && c["data"]["title"] == title
                })
            })
    }
}

fn smoke_item() -> Value {
    let at = now_iso();
    json!({
        "id": "I-3", "title": "smoke", "description": null, "subject": "ui", "thread": "ui",
        "status": "todo", "origin": "coordinator", "ac": [], "createdAt": at,
        "createdBy": "coordinator", "updatedAt": at, "updatedBy": "coordinator", "rank": 3072,
        "rev": 1,
    })
}

/// What `coord item add` does: read the board, change it, write a temp file next to it, rename
/// it over `board.json`.
fn cli_adds_an_item(t: &TestServer) {
    let mut board = t.board("alpha");
    board["items"]["I-3"] = smoke_item();
    board["counters"]["I"] = json!(3);
    let dir = t.coord_home.join("alpha");
    let tmp = dir.join(format!("board.json.{}.tmp", std::process::id()));
    let text = serde_json::to_string_pretty(&board).expect("board json") + "\n";
    std::fs::write(&tmp, text).expect("temp board");
    std::fs::rename(&tmp, dir.join("board.json")).expect("rename over the board");
}

fn changes_with_id(id: &str) -> impl Fn(&Event) -> bool {
    let id = id.to_owned();
    move |e| {
        e.event == "changes"
            && e.data["changes"]
                .as_array()
                .is_some_and(|cs| cs.iter().any(|c| c["id"] == id.as_str()))
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cli_change_reaches_the_stream_within_a_second() {
    let t = seeded().await;
    let s = t.sse("/api/p/alpha/stream?c=items").await;
    assert!(s.next(Some("ready"), 2000).await.is_some());
    cli_adds_an_item(&t);
    let started = Instant::now();
    let e = s.next_where(has_item("smoke"), 3000).await;
    let took = started.elapsed();
    s.close();
    assert!(e.is_some(), "the new item arrived");
    assert!(took < Duration::from_secs(1), "arrived after {took:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_owner_post_is_on_the_stream_once_with_no_echo_from_the_watcher() {
    let t = seeded().await;
    let s = t.sse("/api/p/alpha/stream?c=entries").await;
    assert!(s.next(Some("ready"), 2000).await.is_some());
    let body = json!({ "kind": "followup", "replyTo": "Q-1", "text": "No echo please", "createdAt": now_iso() });
    let reply = t
        .send(Request::new("POST", "/api/p/alpha/entries").json(&body))
        .await;
    let r = reply.json();
    assert_eq!(r["status"], "applied", "{}", reply.text());
    let id = r["id"].as_str().expect("the entry id").to_owned();
    let on_stream = changes_with_id(&id);
    let first = s
        .next_where(on_stream, 2000)
        .await
        .expect("the post reached the stream");
    let first_seq: i64 = first
        .id
        .as_deref()
        .and_then(|i| i.parse().ok())
        .expect("an id");
    assert!(
        first_seq <= r["seq"].as_i64().unwrap(),
        "the change was written before the reply"
    );
    let echo = s.next(Some("changes"), 1800).await;
    s.close();
    assert!(
        echo.is_none(),
        "unexpected changes: {:?}",
        echo.map(|e| e.data)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_board_that_appears_after_start_is_mirrored_by_the_poll_or_the_watcher() {
    let t = seeded().await;
    let mut gamma = empty_board("gamma");
    gamma["name"] = json!("Gamma");
    write_board(&t.coord_home, "gamma", &gamma);
    let until = Instant::now() + Duration::from_secs(12);
    let mut meta = Value::Null;
    while Instant::now() < until && meta["exists"] != true {
        tokio::time::sleep(Duration::from_millis(100)).await;
        meta = t.get("/api/p/gamma/db/meta/board").await.json();
    }
    assert_eq!(meta["data"]["name"], "Gamma");
}
