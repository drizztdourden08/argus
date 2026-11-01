//! The service loop: one poll at start, then one per period, until it is stopped.

use argus_agents::{Tick, start};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

struct Counter(AtomicUsize);

impl Tick for Counter {
    fn period_ms(&self) -> u64 {
        10
    }

    fn run_once(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn polls_at_start_and_every_period_until_stopped() {
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let handle = start(Arc::clone(&counter));
    for _ in 0..200 {
        if counter.0.load(Ordering::SeqCst) >= 3 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(counter.0.load(Ordering::SeqCst) >= 3);
    handle.stop();
    tokio::time::sleep(Duration::from_millis(100)).await;
    let settled = counter.0.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(counter.0.load(Ordering::SeqCst), settled);
}
