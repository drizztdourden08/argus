//! The sub-agent services: the live view of transcripts (the watcher), the reaper that closes
//! agents gone quiet and the reconciler that compares the board with what Claude keeps. All
//! three read the transcripts under the Claude home; the last two write boards under the
//! lock, and wake threads through the wake service.

use std::sync::Arc;

use argus_agents::config::{ReaperOptions, ReconcilerOptions, WatcherOptions};
use argus_agents::{
    AgentReaper, AgentReconciler, AgentWatcher, AgentsConfig, Ports, StdFs, SystemClock, Tick,
    start as every,
};
use argus_core::Coll;
use serde_json::Value;
use tokio::runtime::Handle;

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::agents::{AgentBoards, AgentDocs, AgentWakes};
use crate::{ServeOptions, Services};

/// Which of the sub-agent services run; a test turns one off.
#[derive(Clone, Copy)]
pub struct AgentsOptions {
    /// The live view of transcripts.
    pub watcher: bool,
    /// The reaper.
    pub reaper: bool,
    /// The reconciler.
    pub reconciler: bool,
}

impl Default for AgentsOptions {
    fn default() -> Self {
        AgentsOptions {
            watcher: true,
            reaper: true,
            reconciler: true,
        }
    }
}

fn config(svc: &Services) -> AgentsConfig {
    let settings = Value::Object(svc.config().extra);
    AgentsConfig::from_json(settings.get("agents").unwrap_or(&Value::Null))
}

fn ports(svc: &Services, wakes: bool) -> Ports {
    let boards = AgentBoards::new(
        svc.boards(),
        svc.paths.coord_home.clone(),
        svc.queue.clone(),
        Handle::current(),
    );
    let ports = Ports::new(Arc::new(boards), Arc::new(StdFs), Arc::new(SystemClock))
        .with_activity(Arc::new(ActivityAdapter::of(svc)));
    if wakes && svc.wake.is_some() {
        ports.with_wakes(Arc::new(AgentWakes::new(svc.wake_hook())))
    } else {
        ports
    }
}

pub(crate) fn build_watcher(svc: &Services, opts: &ServeOptions) -> Option<Arc<AgentWatcher>> {
    let options = WatcherOptions::default().configured(&config(svc));
    let docs = Arc::new(AgentDocs::new(Arc::clone(&svc.db)));
    let watcher = opts
        .agents
        .watcher
        .then(|| Arc::new(AgentWatcher::new(ports(svc, false), docs, options)))?;
    overlay(svc, &watcher);
    Some(watcher)
}

pub(crate) fn build_reaper(svc: &Services, opts: &ServeOptions) -> Option<Arc<AgentReaper>> {
    let options = ReaperOptions::default().configured(&config(svc));
    opts.agents
        .reaper
        .then(|| Arc::new(AgentReaper::new(ports(svc, true), options)))
}

pub(crate) fn build_reconciler(
    svc: &Services,
    opts: &ServeOptions,
) -> Option<Arc<AgentReconciler>> {
    let options = ReconcilerOptions::default().configured(&config(svc));
    opts.agents
        .reconciler
        .then(|| Arc::new(AgentReconciler::new(ports(svc, true), options)))
}

/// Runs a service on its period until the server stops.
fn run_until_stop<T: Tick>(svc: &Services, service: &Arc<T>) {
    let handle = every(Arc::clone(service));
    let stop = svc.stop.clone();
    tokio::spawn(async move {
        stop.cancelled().await;
        handle.stop();
    });
}

/// The live fields join the `agents` docs of every project export before they are stored.
fn overlay(svc: &Services, watcher: &Arc<AgentWatcher>) {
    let Some(sync) = &svc.sync else {
        return;
    };
    let watcher = Arc::clone(watcher);
    sync.hooks().on_overlay(move |key, docs| {
        for doc in docs.iter_mut().filter(|d| d.coll == Coll::Agents) {
            if let Some(data) = doc.data.as_mut().and_then(Value::as_object_mut) {
                watcher.overlay_doc(key, &doc.id, data);
            }
        }
    });
}

pub(crate) fn start(svc: &Arc<Services>) {
    if let Some(reaper) = &svc.agent_reaper {
        run_until_stop(svc, reaper);
    }
    if let Some(reconciler) = &svc.agent_reconciler {
        run_until_stop(svc, reconciler);
    }
    if let Some(watcher) = &svc.agent_watcher {
        run_until_stop(svc, watcher);
    }
}

/// Nothing to end here: the three loops stop when the server's token rings, just before the
/// services stop.
pub(crate) fn stop(_svc: &Services) {}
