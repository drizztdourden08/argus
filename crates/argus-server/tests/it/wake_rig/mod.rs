//! What the wake tests share: a server on the fixture boards with short windows, the streams a
//! thread listens on, and the rows the page mirrors.

use std::path::Path;

use crate::wait::until;
use argus_testkit::{Reply, SseReader, TestOptions, TestServer, make_home, write_board};
use serde_json::{Value, json};
use tempfile::TempDir;

pub const WINDOW_MS: u64 = 150;

/// A running server and the folder its boards live in.
pub struct Rig {
    pub t: TestServer,
    _home: TempDir,
}

/// The alpha fixture with three more manual tests of build `b1`, the way the Node test seeds it.
fn alpha_with_tests(home: &Path) -> Value {
    let mut board = argus_testkit::alpha_board(home);
    for (n, title) in [(3, "Wrong user name"), (4, "Empty form"), (5, "Logout")] {
        let mut test = board["tests"]["T-2"].clone();
        test["id"] = json!(format!("T-{n}"));
        test["title"] = json!(title);
        test["rank"] = json!(1024 * n);
        board["tests"][format!("T-{n}")] = test;
    }
    board["counters"]["T"] = json!(5);
    board
}

/// A server whose config is the fixture's with `config` merged over it.
pub async fn rig_with(config: Value) -> Rig {
    let home = tempfile::tempdir().expect("coord home");
    make_home(home.path());
    write_board(home.path(), "alpha", &alpha_with_tests(home.path()));
    let t = TestServer::start(TestOptions {
        coord_home: Some(home.path().to_path_buf()),
        config,
        ..TestOptions::default()
    })
    .await;
    Rig { t, _home: home }
}

/// A server with a short coalescing window and no debounce (the wake policy tests turn it on).
pub async fn rig() -> Rig {
    rig_with(json!({ "wake": { "windowMs": WINDOW_MS }, "wakePolicy": { "debounceMs": 0 } })).await
}

/// A v2 wake stream of a thread, past its `ready`.
pub async fn wake_stream(t: &TestServer, thread: &str, key: &str) -> SseReader {
    let path = format!("/api/p/{key}/wakes/stream?thread={thread}&v=2");
    let s = t.sse(&path).await;
    assert_eq!(s.status, 200);
    let ready = s.next(Some("ready"), 5000).await.expect("ready");
    assert_eq!(ready.data["scope"], key);
    assert_eq!(ready.data["thread"], thread);
    s
}

/// The rows of a project the page shows, newest first.
pub async fn wakes(t: &TestServer, key: &str) -> Vec<Value> {
    let body = t.get(&format!("/api/p/{key}/wakes")).await.json();
    body["wakes"].as_array().cloned().unwrap_or_default()
}

/// Waits for the first row that `keep` accepts.
pub async fn wake_where(t: &TestServer, key: &str, keep: impl Fn(&Value) -> bool) -> Value {
    until("a matching wake row", || async {
        wakes(t, key).await.into_iter().find(|w| keep(w))
    })
    .await
}

/// `POST /api/p/<key>/wakes` with the CLI's headers.
pub async fn post_wake(t: &TestServer, key: &str, body: &Value) -> Reply {
    t.write("POST", &format!("/api/p/{key}/wakes"), body).await
}

/// A one-click result of the owner.
pub async fn result(t: &TestServer, test_id: &str, outcome: &str, note: &str) -> Value {
    let body = json!({ "outcome": outcome, "text": note });
    let path = format!("/api/p/alpha/tests/{test_id}/result");
    t.write("POST", &path, &body).await.json()
}

/// The open streams of a project.
pub async fn listeners(t: &TestServer, key: &str) -> Vec<Value> {
    let body = t.get(&format!("/api/p/{key}/wakes/listeners")).await.json();
    body.as_array().cloned().unwrap_or_default()
}

/// The lines of an event.
pub fn lines(event: &Value) -> Vec<String> {
    event["lines"]
        .as_array()
        .map(|l| {
            l.iter()
                .map(|s| s.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}
