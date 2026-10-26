//! What the watcher test binaries share: a seeded server, the stream predicate of a new item, the
//! time a new item takes to reach the stream, and the wait for a board that appeared after start.

use std::time::{Duration, Instant};

use argus_testkit::{Event, TestOptions, TestServer};
use serde_json::Value;

pub async fn seeded() -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        ..TestOptions::default()
    })
    .await
}

pub fn has_item(title: &'static str) -> impl Fn(&Event) -> bool {
    move |e| {
        e.event == "changes"
            && e.data["changes"].as_array().is_some_and(|cs| {
                cs.iter().any(|c| {
                    c["coll"] == "items" && c["op"] == "set" && c["data"]["title"] == title
                })
            })
    }
}

/// Opens the item stream of `alpha`, runs `change`, and answers how long the item titled `title`
/// took to arrive, or `None` when it did not within 3 seconds.
pub async fn item_arrival(
    t: &TestServer,
    title: &'static str,
    change: impl AsyncFnOnce(),
) -> Option<Duration> {
    let s = t.sse("/api/p/alpha/stream?c=items").await;
    assert!(s.next(Some("ready"), 2000).await.is_some());
    change().await;
    let started = Instant::now();
    let arrived = s.next_where(has_item(title), 3000).await;
    s.close();
    arrived.map(|_| started.elapsed())
}

/// The `board` meta document of project `key`, polled for up to 12 seconds until it exists.
pub async fn mirrored_meta(t: &TestServer, key: &str) -> Value {
    let until = Instant::now() + Duration::from_secs(12);
    let mut meta = Value::Null;
    while Instant::now() < until && meta["exists"] != true {
        tokio::time::sleep(Duration::from_millis(100)).await;
        meta = t.get(&format!("/api/p/{key}/db/meta/board")).await.json();
    }
    meta
}
