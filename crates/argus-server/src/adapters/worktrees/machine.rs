//! The machine's processes for a removal's "is the folder in use" check: the process monitor's
//! last sample when it is fresh, else one sample taken now.

use std::sync::Arc;

use argus_core::clock::now_ms;
use argus_procs::{ProcRow, ProcsService, Sampler, SystemSampler};
use argus_worktrees::{ProcInfo, ProcessLister};

const FRESH_MS: i64 = 60_000;

/// The processes of the machine.
pub struct MachineProcs {
    monitor: Option<Arc<ProcsService>>,
}

impl MachineProcs {
    /// The processes, read from `monitor` when it has a fresh sample.
    #[must_use]
    pub fn new(monitor: Option<Arc<ProcsService>>) -> Self {
        MachineProcs { monitor }
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

#[cfg(target_os = "linux")]
fn cwd_of(pid: u32) -> Option<String> {
    std::fs::read_link(format!("/proc/{pid}/cwd"))
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(not(target_os = "linux"))]
fn cwd_of(_pid: u32) -> Option<String> {
    None
}

impl ProcessLister for MachineProcs {
    fn list(&self) -> Result<Vec<ProcInfo>, String> {
        Ok(self
            .rows()?
            .into_iter()
            .map(|row| ProcInfo {
                cwd: cwd_of(row.pid),
                pid: row.pid,
                name: row.name,
                cmd: row.cmd,
                path: row.path,
            })
            .collect())
    }
}
