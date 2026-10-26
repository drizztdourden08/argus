//! The sampling timer: a first run after a delay, then one every interval, until stopped.

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

/// A background thread that runs a task on a schedule.
#[derive(Debug)]
pub struct Timer {
    stop: Arc<(Mutex<bool>, Condvar)>,
}

fn sleep_unless_stopped(stop: &(Mutex<bool>, Condvar), ms: u64) -> bool {
    let (flag, signal) = stop;
    let Ok(guard) = flag.lock() else { return true };
    let result = signal.wait_timeout_while(guard, Duration::from_millis(ms), |stopped| !*stopped);
    result.map_or(true, |(stopped, _)| *stopped)
}

impl Timer {
    /// Runs `task` after `delay_ms`, then every `every_ms`; the thread ends once stopped.
    pub fn start(delay_ms: u64, every_ms: u64, task: impl Fn() + Send + 'static) -> Self {
        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let seen = Arc::clone(&stop);
        thread::spawn(move || {
            let mut wait = delay_ms;
            while !sleep_unless_stopped(&seen, wait) {
                task();
                wait = every_ms;
            }
        });
        Self { stop }
    }

    /// Stops the schedule; a run in progress finishes first.
    pub fn stop(&self) {
        let (flag, signal) = &*self.stop;
        if let Ok(mut stopped) = flag.lock() {
            *stopped = true;
        }
        signal.notify_all();
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
    }
}
