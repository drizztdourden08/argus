//! The worktree scanner and its removals. Not wired yet: `build` answers `None`. The wiring
//! fills the three functions, adds `adapters/worktrees.rs` and `routes/worktrees.rs`, and gives
//! the service the process monitor's last sample (`svc.procs`).

use std::sync::Arc;

use argus_worktrees::Service as WorktreeService;

use crate::{ServeOptions, Services};

pub(crate) fn build(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<WorktreeService>> {
    None
}

pub(crate) fn start(_svc: &Arc<Services>) {}

pub(crate) fn stop(_svc: &Services) {}
