//! Runs a service on its period with tokio timers: once at start, then every period.

use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::{MissedTickBehavior, interval};

/// A service that does one poll when asked.
pub trait Tick: Send + Sync + 'static {
    /// Milliseconds between polls.
    fn period_ms(&self) -> u64;
    /// One poll. Faults are logged by the service itself; none stops the loop.
    fn run_once(&self);
}

/// A running service loop.
#[derive(Debug)]
pub struct ServiceHandle(JoinHandle<()>);

impl ServiceHandle {
    /// Stops the loop. A poll already running finishes on its own.
    pub fn stop(self) {
        self.0.abort();
    }
}

/// Starts the loop on the current tokio runtime. Each poll runs on a blocking thread because it
/// reads files and the board lock.
#[must_use]
pub fn start<T: Tick>(service: Arc<T>) -> ServiceHandle {
    let task = tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(service.period_ms().max(1)));
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let svc = Arc::clone(&service);
            if let Err(err) = tokio::task::spawn_blocking(move || svc.run_once()).await {
                tracing::warn!(%err, "agent service poll failed");
            }
        }
    });
    ServiceHandle(task)
}
