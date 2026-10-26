//! The real sampler: every process of this machine, read natively with no child process.

use crate::clock::now_ms;
use crate::ports::Sampler;
use crate::types::{ProcRow, Sample};
use std::ffi::OsStr;
use sysinfo::{Process, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// Reads the process table through the operating system.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemSampler;

fn lossy(text: &OsStr) -> String {
    text.to_string_lossy().into_owned()
}

fn row_of(process: &Process) -> ProcRow {
    let seconds = i64::try_from(process.start_time()).unwrap_or(0);
    ProcRow {
        pid: process.pid().as_u32(),
        ppid: process.parent().map_or(0, sysinfo::Pid::as_u32),
        name: lossy(process.name()),
        cmd: process
            .cmd()
            .iter()
            .map(|a| lossy(a))
            .collect::<Vec<_>>()
            .join(" "),
        path: process.exe().map(|p| p.display().to_string()),
        mem_bytes: process.memory(),
        start_ms: seconds.saturating_mul(1000),
        cpu_ms: i64::try_from(process.accumulated_cpu_time()).unwrap_or(0),
        responding: None,
    }
}

/// The platform in the names the dashboard has always used (`win32`, `linux`, `darwin`).
#[must_use]
pub fn platform_name() -> String {
    match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        other => other,
    }
    .to_string()
}

impl Sampler for SystemSampler {
    fn sample(&self) -> Result<Sample, String> {
        let mut system = System::new();
        let kind = ProcessRefreshKind::nothing()
            .with_memory()
            .with_cpu()
            .with_exe(UpdateKind::Always)
            .with_cmd(UpdateKind::Always);
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, kind);
        let at = now_ms();
        let procs: Vec<ProcRow> = system.processes().values().map(row_of).collect();
        if procs.is_empty() {
            return Err("the process table came back empty".to_string());
        }
        Ok(Sample {
            at,
            platform: platform_name(),
            sampler_pid: None,
            procs,
        })
    }
}
