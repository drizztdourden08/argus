//! The run service: the processes a lead defined, started, logged and stopped. Its storage is
//! the dashboard database, its run docs reach the page streams, its definitions are read from
//! each board file. Shadow mode starts nothing: the service reads, and every write answers
//! `read_only`.

use std::sync::Arc;

use argus_launchers::{
    BoardFileDefs, FakeHost, RunConfig, RunDeps, RunLimits, RunService, SystemClock, SystemHost,
};
use tokio::runtime::{Handle, RuntimeFlavor};

use crate::adapters::link::WakeLink;
use crate::adapters::runs::{DocRunEvents, RunHost, SqliteRuns};
use crate::{ServeOptions, Services};

/// How the run service starts; the fields after `enabled` are seams for tests.
#[derive(Clone)]
pub struct RunsOptions {
    /// Run the service.
    pub enabled: bool,
    /// Pipes a test drives by hand instead of real processes.
    pub host: Option<FakeHost>,
    /// Tick the launcher wakes on their timer.
    pub stale_timer: bool,
}

impl Default for RunsOptions {
    fn default() -> Self {
        RunsOptions {
            enabled: true,
            host: None,
            stale_timer: true,
        }
    }
}

fn config_of(svc: &Services) -> RunConfig {
    let config = svc.config();
    let runs = &config.runs;
    RunConfig {
        limits: RunLimits {
            max_concurrent: usize::try_from(runs.max_concurrent).unwrap_or(4),
            log_cap_bytes: runs.log_cap_bytes,
            log_head_bytes: runs.log_head_bytes,
            keep_per_def: usize::try_from(runs.keep_per_def).unwrap_or(30),
            keep_days: i64::try_from(runs.keep_days).unwrap_or(14),
            stop_grace_ms: runs.stop_grace_ms,
        },
        token: Some(config.token.clone()),
        attachments_dir: svc.paths.attachments.clone(),
        attach_max_bytes: usize::try_from(config.attach.max_bytes).unwrap_or(10 << 20),
    }
}

fn deps_of(svc: &Services, opts: &RunsOptions) -> RunDeps<RunHost> {
    RunDeps {
        host: opts
            .host
            .clone()
            .map_or(RunHost::System(SystemHost), RunHost::Fake),
        store: Arc::new(SqliteRuns::new(Arc::clone(&svc.db), Arc::clone(&svc.log))),
        events: Arc::new(DocRunEvents::new(
            Arc::clone(&svc.db),
            Arc::clone(&svc.run_logs),
            Arc::clone(&svc.log),
        )),
        defs: Arc::new(BoardFileDefs::new(svc.paths.coord_home.clone(), None)),
        config: config_of(svc),
        clock: Arc::new(SystemClock),
    }
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<RunService<RunHost>>> {
    let options = &opts.run_service;
    options.enabled.then(|| {
        let service = RunService::new(deps_of(svc, options));
        let link = Arc::clone(&svc.wake_link);
        service.on_end(move |run| link.run_ended(run));
        Arc::new(service)
    })
}

/// Before serving: nothing survives a restart as running or queued.
pub(crate) fn start(svc: &Arc<Services>) {
    if let Some(runs) = svc.runs.as_ref().filter(|_| !svc.shadow) {
        runs.recover();
    }
}

/// Ends the runs: queued ones fail, hidden ones are stopped, visible ones are left running.
/// The server runs on the multi-thread runtime, where a blocking wait is allowed.
pub(crate) fn stop(svc: &Services) {
    let Some(runs) = svc.runs.as_ref().filter(|_| !svc.shadow) else {
        return;
    };
    match Handle::try_current() {
        Ok(rt) if rt.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(|| rt.block_on(runs.shutdown()));
        }
        _ => svc
            .log
            .warn("the runs were not shut down", serde_json::json!({})),
    }
}
