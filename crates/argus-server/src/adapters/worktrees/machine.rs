//! The machine's processes for a removal's "is the folder in use" check: the process monitor's
//! last sample when it is fresh, else one sample taken now, each with the folder it runs in (one read
//! for the whole listing, see `cwds`). And Argus's own live runs, with the
//! folder each runs in, for a refusal that names who holds a folder.

use std::sync::Arc;

use argus_core::clock::now_ms;
use argus_launchers::{ListFilter, RunService};
use argus_procs::{ProcRow, ProcsService, Sampler, SystemSampler};
use argus_worktrees::{LiveRun, ProcInfo, ProcessLister};

use super::cwds::cwds_of;
use crate::adapters::runs::RunHost;

const FRESH_MS: i64 = 5_000;
/// Live runs looked at, at most.
const RUNS_MAX: i64 = 500;

/// The processes of the machine.
pub struct MachineProcs {
    monitor: Option<Arc<ProcsService>>,
    runs: Option<Arc<RunService<RunHost>>>,
}

impl MachineProcs {
    /// The processes, read from `monitor` when it has a fresh sample; the live runs from `runs`.
    #[must_use]
    pub fn new(monitor: Option<Arc<ProcsService>>, runs: Option<Arc<RunService<RunHost>>>) -> Self {
        MachineProcs { monitor, runs }
    }

    fn rows(&self) -> Result<Vec<ProcRow>, String> {
        let fresh = self
            .monitor
            .as_ref()
            .and_then(|m| m.snapshot())
            .filter(|s| now_ms() - s.at < FRESH_MS);
        match fresh {
            Some(sample) => Ok(sample.procs),
            None => SystemSampler.sample().map(|s| s.procs),
        }
    }
}

impl ProcessLister for MachineProcs {
    fn list(&self) -> Result<Vec<ProcInfo>, String> {
        let rows = self.rows()?;
        let pids: Vec<u32> = rows.iter().map(|row| row.pid).collect();
        let mut cwds = cwds_of(&pids);
        Ok(rows
            .into_iter()
            .map(|row| ProcInfo {
                cwd: cwds.remove(&row.pid),
                pid: row.pid,
                name: row.name,
                cmd: row.cmd,
                path: row.path,
            })
            .collect())
    }

    fn runs(&self) -> Vec<LiveRun> {
        let Some(runs) = &self.runs else {
            return Vec::new();
        };
        let filter = ListFilter {
            limit: Some(RUNS_MAX),
            ..ListFilter::default()
        }
        .with_statuses("running");
        runs.list_all(&filter)
            .into_iter()
            .map(|row| LiveRun {
                id: row.id,
                project: row.scope,
                cmd: row.cmd,
                cwd: row.cwd,
            })
            .collect()
    }
}
