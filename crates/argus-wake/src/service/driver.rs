//! The tokio driver: sleeps until the service has work, then runs `tick`.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tokio::task::JoinHandle;

use super::WakeService;

/// The service as the server shares it.
pub type SharedWake = Arc<Mutex<WakeService>>;

const MIN_WAIT_MS: u64 = 5;
const MAX_WAIT_MS: u64 = 1000;

/// Lock the shared service, even when a holder panicked.
pub fn lock(service: &SharedWake) -> MutexGuard<'_, WakeService> {
    service.lock().unwrap_or_else(PoisonError::into_inner)
}

fn wait_for(service: &SharedWake) -> Duration {
    let ms = u64::try_from(lock(service).until_next_ms()).unwrap_or(0);
    Duration::from_millis(ms.clamp(MIN_WAIT_MS, MAX_WAIT_MS))
}

/// Run the service until the returned task is aborted. A submit wakes the driver early so a new,
/// earlier deadline is not slept through.
pub fn spawn(service: SharedWake) -> JoinHandle<()> {
    let notify = lock(&service).notifier();
    tokio::spawn(async move {
        loop {
            let wait = wait_for(&service);
            tokio::select! {
                () = tokio::time::sleep(wait) => {}
                () = notify.notified() => {}
            }
            lock(&service).tick();
        }
    })
}
