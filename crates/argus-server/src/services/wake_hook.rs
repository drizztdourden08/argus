//! The one door other services use to wake a thread: [`WakeHook::submit`]. The runs, the
//! launcher wakes, the worktrees, the process monitor and the sub-agent services each have
//! their own wake type in their crate; their adapters turn it into a [`WakeRequest`] and call
//! this. With the wake service off, a submit is a no-op that succeeds.

use std::sync::{Arc, Mutex};

use argus_wake::service::WakeService;
use argus_wake::service::driver::lock;
use argus_wake::wake::WakeInput;

use super::Services;

/// One wake for a thread: `WakeRequest::about(scope, thread, reason, subject, text)`.
pub type WakeRequest = WakeInput;

/// A handle on the wake service that any service may keep.
#[derive(Clone)]
pub struct WakeHook {
    wake: Option<Arc<Mutex<WakeService>>>,
}

impl WakeHook {
    /// Hands wakes to the engine, which coalesces and queues them. Takes the engine's lock and
    /// writes to the database for a moment, so a caller on the runtime goes through
    /// `spawn_blocking`.
    ///
    /// # Errors
    /// The reason the engine could not record them; they are then not sent.
    pub fn submit(&self, wakes: &[WakeRequest]) -> Result<(), String> {
        let Some(wake) = &self.wake else {
            return Ok(());
        };
        lock(wake)
            .submit(wakes)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

impl Services {
    /// The door to the wake service; take it after the wake service is built.
    #[must_use]
    pub fn wake_hook(&self) -> WakeHook {
        WakeHook {
            wake: self.wake.clone(),
        }
    }
}
