//! The dependency trail over HTTP (I-77): an item of alpha waits on an item of beta that has an
//! open question; the tree and the numbers of the board come back, and a board or an item that is
//! not there is a 404. The walk itself is tested in `argus-board`.
// test-tag: critical

use argus_core::fixture::doc;
use argus_testkit::{TestOptions, TestServer, alpha_board, beta_board, make_home, write_board};
use serde_json::json;
use tempfile::TempDir;

async fn server() -> (TestServer, TempDir) {
    let home = tempfile::tempdir().expect("coord home");
    make_home(home.path());
    let mut alpha = alpha_board(home.path());
    let waits = json!([{
        "dep": "D-7", "project": "beta", "item": "I-1", "title": "Health endpoint",
        "state": "open", "since": "2026-10-30T08:00:00Z",
    }]);
    alpha["items"]["I-90"] = doc(
        "I-90",
        "ui",
        json!({ "title": "Show the health", "thread": "ui", "status": "doing", "waitsOn": waits }),
        9000,
    );
    write_board(home.path(), "alpha", &alpha);
    write_board(home.path(), "beta", &beta_board());
    let t = TestServer::start(TestOptions {
        coord_home: Some(home.path().to_path_buf()),
        ..TestOptions::default()
    })
    .await;
    (t, home)
}

#[tokio::test]
async fn an_item_that_waits_across_boards_has_its_whole_chain_served() {
    let (t, _home) = server().await;
    let reply = t.get("/api/p/alpha/items/I-90/trail").await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let trail = reply.json();
    assert_eq!(trail["depth"], json!(3));
    assert_eq!(trail["open"], json!(3));
    assert_eq!(trail["root"]["project"], json!("alpha"));
    let middle = &trail["root"]["children"][0];
    assert_eq!(middle["project"], json!("beta"));
    assert_eq!(middle["item"], json!("I-1"));
    assert_eq!(middle["edge"]["kind"], json!("dep-waits"));
    assert_eq!(middle["edge"]["id"], json!("D-7"));
    let leaf = &middle["children"][0];
    assert_eq!(leaf["item"], json!("Q-1"));
    assert_eq!(leaf["type"], json!("question"));
}

#[tokio::test]
async fn the_trails_of_a_board_list_each_blocked_item_with_its_numbers() {
    let (t, _home) = server().await;
    let reply = t.get("/api/p/alpha/trails").await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["trails"]["I-90"], json!({ "depth": 3, "open": 3 }));
    let again = t.get("/api/p/alpha/trails").await;
    assert_eq!(again.json(), body, "the kept answer is the same");
}

#[tokio::test]
async fn a_board_or_an_item_that_is_not_there_is_a_404() {
    let (t, _home) = server().await;
    for path in [
        "/api/p/alpha/items/I-999/trail",
        "/api/p/nowhere/items/I-1/trail",
        "/api/p/nowhere/trails",
    ] {
        let reply = t.get(path).await;
        assert_eq!(reply.status, 404, "{path}: {}", reply.text());
    }
}
