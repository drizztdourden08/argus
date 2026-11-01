//! The wake engine and its registry: owner posts to wakes, the coalescing windows, the
//! listeners, the registry of who can be woken, the faults and the reminders. A
//! [`WakeService`] over SQLite, the doc store and the boards; its driver sleeps until the next
//! deadline. Other services wake threads through [`super::wake_hook::WakeHook`].

use std::sync::{Arc, Mutex};

use argus_core::clock::SystemClock;
use argus_sync::SyncService;
use argus_wake::config::WakeConfig;
use argus_wake::service::driver::{self, SharedWake, lock};
use argus_wake::service::{WakeDeps, WakeService};
use argus_wake::source::FileSyncs;
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::wake::{
    SqlFaultStore, SqlWakeStore, WakeBoards, WakeDocs, WatcherLive, applied_of,
};
use crate::{ServeOptions, Services};

fn deps(svc: &Services, boards: &Arc<WakeBoards>) -> WakeDeps {
    WakeDeps {
        store: Box::new(SqlWakeStore::new(Arc::clone(&svc.db))),
        faults: Box::new(SqlFaultStore::new(Arc::clone(&svc.db))),
        docs: Arc::new(WakeDocs::new(Arc::clone(&svc.db))),
        boards: Arc::clone(boards) as _,
        live: Arc::new(WatcherLive::new(svc.agent_watcher.clone())),
        syncs: Arc::new(FileSyncs::new(svc.paths.coord_home.clone())),
        activity: Some(Arc::new(ActivityAdapter::of(svc))),
        clock: Arc::new(SystemClock),
    }
}

/// Hangs the wake service on the sync service: each applied owner post asks the policy, each
/// exported board refreshes the registry. Hooks that fire after the stop are ignored.
fn link(sync: &SyncService, wake: &SharedWake, boards: &Arc<WakeBoards>, stop: &CancellationToken) {
    let (service, stop) = (Arc::clone(wake), stop.clone());
    sync.hooks().on_applied(move |post| {
        if !stop.is_cancelled() {
            lock(&service).on_applied(&applied_of(post));
        }
    });
    let (service, boards) = (Arc::clone(wake), Arc::clone(boards));
    sync.on_board(move |key| {
        boards.refresh(key);
        lock(&service).on_board(key);
    });
}

/// Whether the wake service runs.
#[derive(Clone, Copy)]
pub struct WakeOptions {
    /// Run it; it needs the sync service.
    pub enabled: bool,
}

impl Default for WakeOptions {
    fn default() -> Self {
        WakeOptions { enabled: true }
    }
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<Mutex<WakeService>>> {
    let sync = svc.sync.as_ref().filter(|_| opts.wake.enabled)?;
    let boards = Arc::new(WakeBoards::new(svc.boards()));
    let settings = Value::Object(svc.config().extra);
    let config = WakeConfig::from_json(&settings);
    let wake = Arc::new(Mutex::new(WakeService::new(
        deps(svc, &boards),
        &config,
        true,
    )));
    link(sync, &wake, &boards, &svc.stop);
    Some(wake)
}

pub(crate) fn start(svc: &Arc<Services>) {
    let Some(wake) = &svc.wake else {
        return;
    };
    lock(wake).start();
    let driver = driver::spawn(Arc::clone(wake));
    let stop = svc.stop.clone();
    tokio::spawn(async move {
        stop.cancelled().await;
        driver.abort();
    });
    let service = Arc::clone(wake);
    svc.keep(svc.bus.on_reset(move |e| lock(&service).on_reset(&e.scope)));
}

pub(crate) fn stop(svc: &Services) {
    if let Some(wake) = &svc.wake {
        lock(wake).stop();
    }
}
