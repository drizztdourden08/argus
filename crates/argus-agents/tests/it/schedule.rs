//! The service loop: one poll at start, then one per period, on tokio's paused clock.
//! K2: the service loop polls at start and every period.
// test-tag: critical

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

#[tokio::test(start_paused = true)]
async fn polls_at_start_and_every_period() {
    let counter = Arc::new(Counter(AtomicUsize::new(0)));
    let handle = start(Arc::clone(&counter));
    while counter.0.load(Ordering::SeqCst) < 3 {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    handle.stop();
}
