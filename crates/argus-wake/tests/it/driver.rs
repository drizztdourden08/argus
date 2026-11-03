//! KEEP: S2 (the tokio driver and the service together).
//! The tokio driver on a manual clock: a submitted wake is delivered once its window has passed, with no one
//! calling `tick`. The driver sleeps in real time for a few milliseconds between ticks, but what the test waits for
//! is set by the clock it moves, never by how long the sleep took.

// test-tag: standard

use std::sync::{Arc, Mutex};

use argus_wake::ManualClock;
use argus_wake::config::WakeConfig;
use argus_wake::engine::{AttachOptions, ListenerMessage, channel};
use argus_wake::memory::{MemoryBoards, MemoryDocs, MemoryFaults, MemoryStore};
use argus_wake::service::driver::{SharedWake, lock, spawn};
use argus_wake::service::{WakeDeps, WakeService};
use argus_wake::source::{NoLive, NoSyncs};
use argus_wake::testkit::{START, wake};

const WINDOW_MS: i64 = 30;

fn shared(clock: &ManualClock) -> SharedWake {
    let deps = WakeDeps {
        store: Box::new(MemoryStore::new()),
        faults: Box::new(MemoryFaults::new()),
        docs: Arc::new(MemoryDocs::new()),
        boards: Arc::new(MemoryBoards::new()),
        live: Arc::new(NoLive),
        syncs: Arc::new(NoSyncs),
        activity: None,
        clock: Arc::new(clock.clone()),
    };
    let config = WakeConfig {
        window_ms: WINDOW_MS,
        ..WakeConfig::default()
    };
    Arc::new(Mutex::new(WakeService::new(deps, &config, false)))
}

#[tokio::test]
async fn a_wake_is_delivered_when_its_window_closes_without_anyone_ticking() {
    let clock = ManualClock::at(START);
    let service = shared(&clock);
    let (sink, mut rx) = channel();
    lock(&service).attach("alpha", "ui", Box::new(sink), AttachOptions::default());
    let driver = spawn(Arc::clone(&service));
    lock(&service)
        .submit(&[wake("ui", "T-1 failed: x"), wake("ui", "T-2 failed: y")])
        .expect("submit");
    clock.advance(WINDOW_MS);
    let message = rx.recv().await.expect("open");
    let ListenerMessage::Wake(event) = message else {
        panic!("expected a wake, got {message:?}");
    };
    assert_eq!((event.thread.as_str(), event.count), ("ui", 2));
    assert_eq!(event.lines, ["T-1 failed: x", "T-2 failed: y"]);
    driver.abort();
}
