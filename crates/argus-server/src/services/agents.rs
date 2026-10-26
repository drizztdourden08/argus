//! The sub-agent services: the live view of transcripts, the reaper and the reconciler. Not
//! wired yet: every build answers `None`. The wiring fills the functions and adds
//! `adapters/agents.rs`.

use std::sync::Arc;

use argus_agents::{AgentReaper, AgentReconciler, AgentWatcher};

use crate::{ServeOptions, Services};

pub(crate) fn build_watcher(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<AgentWatcher>> {
    None
}

pub(crate) fn build_reaper(_svc: &Services, _opts: &ServeOptions) -> Option<Arc<AgentReaper>> {
    None
}

pub(crate) fn build_reconciler(
    _svc: &Services,
    _opts: &ServeOptions,
) -> Option<Arc<AgentReconciler>> {
    None
}

pub(crate) fn start(_svc: &Arc<Services>) {}

pub(crate) fn stop(_svc: &Services) {}
