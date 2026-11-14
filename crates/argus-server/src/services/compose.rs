//! The three orders of the services. A service is built from the ones above it, started in
//! the Node boot order (the wake engine before the sync service, whose boot recovery may wake
//! threads) and stopped in the reverse. A new service adds one line to each function.

use std::sync::Arc;

use super::{
    accounts, activity, agents, cache, deps, notify, procs, records, reminders, runs, stale,
    stalls, sync, wake, wake_relay, worktrees,
};
use super::overlord::live;
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
    svc.accounts = accounts::build(svc, opts);
    svc.worktrees = worktrees::build(svc, opts);
    svc.cache = cache::build(svc, opts);
    svc.records = records::build(svc, opts);
    svc.notifier = notify::build_notifier(svc, opts);
    svc.notify = notify::build(svc, opts);
    reminders::wire(svc);
    wake_relay::wire(svc);
    stalls::wire(svc);
}

pub(crate) async fn start_all(svc: &Arc<Services>) {
    activity::start(svc);
    runs::start(svc);
    wake::start(svc);
    stale::start(svc);
    notify::start(svc);
    deps::start(svc);
    sync::start(svc).await;
    agents::start(svc);
    procs::start(svc);
    accounts::start(svc);
    worktrees::start(svc);
    cache::start(svc);
    records::start(svc).await;
    live::start(svc);
    activity::started(svc);
}

pub(crate) fn stop_all(svc: &Services) {
    activity::stopped(svc);
    live::stop(svc);
    notify::stop(svc);
    cache::stop(svc);
    records::stop(svc);
    worktrees::stop(svc);
    accounts::stop(svc);
    procs::stop(svc);
    agents::stop(svc);
    deps::stop(svc);
    stale::stop(svc);
    wake::stop(svc);
    runs::stop(svc);
    sync::stop(svc);
}
