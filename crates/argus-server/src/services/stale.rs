//! The launcher wakes (no master launcher, a launcher unused past its window). Not wired yet:
//! `build` answers `None`. It needs the wake engine, so the wiring comes with or after the
//! wake wiring.

use std::sync::Arc;

use argus_launchers::launcher::LauncherStale;

use crate::{ServeOptions, Services};

pub(crate) fn build(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<LauncherStale>> {
    None
}

pub(crate) fn start(_svc: &Arc<Services>) {}

pub(crate) fn stop(_svc: &Services) {}
