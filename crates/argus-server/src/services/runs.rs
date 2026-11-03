//! The run service: the processes a lead defined, started, logged and stopped. Its storage is
//! the dashboard database, its run docs reach the page streams, its definitions are read from
//! each board file. Shadow mode starts nothing: the service reads, and every write answers
//! `read_only`.

use std::path::Path;
use std::sync::{Arc, PoisonError};

use argus_core::Config;
use argus_launchers::{
    BoardFileDefs, ConfigSource, FakeHost, RunConfig, RunDeps, RunLimits, RunService, SystemClock,
    SystemHost,
};
use argus_wake::service::driver::lock;
use tokio::runtime::{Handle, RuntimeFlavor};

use crate::adapters::runs::{DocRunEvents, RunHost, SqliteRuns, view_of};
use crate::routes::check_runs::link_on_finish;
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

/// The run settings of the config as it is now.
fn run_config(config: &Config, attachments_dir: &Path) -> RunConfig {
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
        attachments_dir: attachments_dir.to_path_buf(),
        attach_max_bytes: usize::try_from(config.attach.max_bytes).unwrap_or(10 << 20),
    }
}

/// Reads the live config on every call: `config.runs` and `config.attach` apply to the next
/// run and the next attachment, and a rotated token is scrubbed from the next log.
fn config_source(svc: &Services) -> ConfigSource {
    let live = Arc::clone(&svc.config);
    let attachments = svc.paths.attachments.clone();
    Arc::new(move || {
        let config = live.read().unwrap_or_else(PoisonError::into_inner);
        run_config(&config, &attachments)
    })
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
        config: config_source(svc),
        clock: Arc::new(SystemClock),
    }
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<RunService<RunHost>>> {
    let options = &opts.run_service;
    options
        .enabled
        .then(|| Arc::new(RunService::new(deps_of(svc, options))))
}

/// A run that ended is told to the wake service, which wakes the lead of a failed one. The wake
/// service is built after the runs, so the listener is added here.
fn tell_wake_of_ends(svc: &Services, runs: &RunService<RunHost>) {
    if let Some(wake) = svc.wake.clone() {
        runs.on_end(move |run| lock(&wake).on_run_end(&view_of(run)));
    }
}

/// Before serving: nothing survives a restart as running or queued.
pub(crate) fn start(svc: &Arc<Services>) {
    if let Some(runs) = &svc.runs {
        tell_wake_of_ends(svc, runs);
        link_on_finish(svc, runs);
    }
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
