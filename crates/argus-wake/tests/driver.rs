//! The tokio driver on the wall clock: a submitted wake is delivered once its window has passed, with no
//! one calling `tick`.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use argus_wake::SystemClock;
use argus_wake::config::WakeConfig;
use argus_wake::engine::{AttachOptions, ListenerMessage, channel};
use argus_wake::memory::{MemoryBoards, MemoryDocs, MemoryFaults, MemoryStore};
use argus_wake::service::driver::{SharedWake, lock, spawn};
use argus_wake::service::{WakeDeps, WakeService};
use argus_wake::source::NoLive;
use argus_wake::testkit::wake;

fn shared() -> SharedWake {
    let deps = WakeDeps {
        store: Box::new(MemoryStore::new()),
        faults: Box::new(MemoryFaults::new()),
        docs: Arc::new(MemoryDocs::new()),
        boards: Arc::new(MemoryBoards::new()),
        live: Arc::new(NoLive),
        activity: None,
        clock: Arc::new(SystemClock),
    };
    let config = WakeConfig {
        window_ms: 30,
        ..WakeConfig::default()
    };
    Arc::new(Mutex::new(WakeService::new(deps, &config, false)))
}

#[tokio::test]
async fn a_wake_is_delivered_when_its_window_closes_without_anyone_ticking() {
    let service = shared();
    let (sink, mut rx) = channel();
    lock(&service).attach("alpha", "ui", Box::new(sink), AttachOptions::default());
    let driver = spawn(Arc::clone(&service));
    lock(&service)
        .submit(&[wake("ui", "T-1 failed: x"), wake("ui", "T-2 failed: y")])
        .expect("submit");
    let message = tokio::time::timeout(Duration::from_secs(5), rx.recv())
        .await
        .expect("delivered in time")
        .expect("open");
    let ListenerMessage::Wake(event) = message else {
        panic!("expected a wake, got {message:?}");
    };
    assert_eq!((event.thread.as_str(), event.count), ("ui", 2));
    assert_eq!(event.lines, ["T-1 failed: x", "T-2 failed: y"]);
    driver.abort();
}

#[tokio::test]
async fn a_second_listener_replaces_the_first_and_the_first_is_told_through_its_channel() {
    let service = shared();
    let (first, mut old) = channel();
    let (second, _new) = channel();
    lock(&service).attach("alpha", "ui", Box::new(first), AttachOptions::default());
    lock(&service).attach("alpha", "ui", Box::new(second), AttachOptions::default());
    assert_eq!(old.recv().await, Some(ListenerMessage::Replaced));
    assert_eq!(old.recv().await, None, "and its stream ends");
    assert_eq!(lock(&service).listeners("alpha").len(), 1);
}
