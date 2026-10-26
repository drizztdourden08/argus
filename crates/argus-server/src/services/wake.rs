//! The wake engine and its registry. Not wired yet: `build` answers `None`. The wiring fills
//! the three functions, adds `adapters/wake.rs` and `routes/wake.rs`, and points the process
//! monitor's host (`adapters/procs.rs`, `submit_wake`) at the engine.

use std::sync::{Arc, Mutex};

use argus_wake::service::WakeService;

use crate::{ServeOptions, Services};

pub(crate) fn build(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<Mutex<WakeService>>> {
    None
}

pub(crate) fn start(_svc: &Arc<Services>) {}

pub(crate) fn stop(_svc: &Services) {}
