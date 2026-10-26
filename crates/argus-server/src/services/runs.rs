//! The run service. Not wired yet: `build` answers `None`. The wiring fills the three
//! functions, adds `adapters/runs.rs`, moves the run routes out of `routes/unavailable.rs`
//! into `routes/runs.rs`, and hands the service to the sync service as its `RunsView`
//! (`services/sync.rs`).

use std::sync::Arc;

use argus_launchers::{RunService, SystemHost};

use crate::{ServeOptions, Services};

pub(crate) fn build(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<RunService<SystemHost>>> {
    None
}

pub(crate) fn start(_svc: &Arc<Services>) {}

pub(crate) fn stop(_svc: &Services) {}
