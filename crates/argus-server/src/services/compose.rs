//! The three orders of the services. A service is built from the ones above it, started in
//! the Node boot order (the wake engine before the sync service, whose boot recovery may wake
//! threads) and stopped in the reverse. A new service adds one line to each function.

use std::sync::Arc;

use super::{activity, agents, cache, notify, procs, records, runs, stale, sync, wake, worktrees};
use crate::{ServeOptions, Services};

pub(crate) fn build_all(svc: &mut Services, opts: &ServeOptions) {
    svc.runs = runs::build(svc, opts);
    svc.sync = sync::build(svc, opts);
    svc.agent_watcher = agents::build_watcher(svc, opts);
    svc.wake = wake::build(svc, opts);
    svc.agent_reaper = agents::build_reaper(svc, opts);
    svc.agent_reconciler = agents::build_reconciler(svc, opts);
    svc.launcher_stale = stale::build(svc, opts);
    svc.procs = procs::build(svc, opts);
    svc.worktrees = worktrees::build(svc, opts);
    svc.cache = cache::build(svc, opts);
    svc.records = records::build(svc, opts);
    svc.notifier = notify::build_notifier(svc, opts);
    svc.notify = notify::build(svc, opts);
}

pub(crate) async fn start_all(svc: &Arc<Services>) {
    activity::start(svc);
    runs::start(svc);
    wake::start(svc);
    stale::start(svc);
    notify::start(svc);
    sync::start(svc).await;
    agents::start(svc);
    procs::start(svc);
    worktrees::start(svc);
    cache::start(svc);
    records::start(svc).await;
    activity::started(svc);
}

pub(crate) fn stop_all(svc: &Services) {
    activity::stopped(svc);
    notify::stop(svc);
    cache::stop(svc);
    records::stop(svc);
    worktrees::stop(svc);
    procs::stop(svc);
    agents::stop(svc);
    stale::stop(svc);
    wake::stop(svc);
    runs::stop(svc);
    sync::stop(svc);
}
