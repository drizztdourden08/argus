//! A fake machine for the process monitor: a list of processes that is the sampler, and a
//! killer that removes the ones it ends.

use std::sync::{Arc, Mutex, PoisonError};

use argus_core::clock::now_ms;
use argus_procs::{KillResult, Killer, NoResponder, ProcRow, Sample, Sampler};
use argus_server::ServeOptions;

pub struct Machine {
    rows: Mutex<Vec<ProcRow>>,
}

/// A process of the fake machine: a node started by the test, `mem` MB, started at `start`.
pub fn row(pid: u32, tag: &str, mem: u64, start: i64) -> ProcRow {
    ProcRow {
        pid,
        ppid: 1,
        name: "node.exe".into(),
        cmd: format!("node procs-http-test {tag}"),
        path: Some("C:\\nvm4w\\nodejs\\node.exe".into()),
        mem_bytes: mem * 1_048_576,
        start_ms: start,
        cpu_ms: 0,
        responding: None,
    }
}

impl Machine {
    /// A machine with only this process on it.
    pub fn with_self() -> Self {
        let own = row(std::process::id(), "self", 50, now_ms() - 60_000);
        Machine {
            rows: Mutex::new(vec![own]),
        }
    }

    fn rows(&self) -> std::sync::MutexGuard<'_, Vec<ProcRow>> {
        self.rows.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Puts a process on the machine and answers its pid.
    pub fn add(&self, row: ProcRow) -> u32 {
        let pid = row.pid;
        self.rows().push(row);
        pid
    }

    /// Whether the process is still on the machine.
    pub fn alive(&self, pid: u32) -> bool {
        self.rows().iter().any(|r| r.pid == pid)
    }
}

impl Sampler for Machine {
    fn sample(&self) -> Result<Sample, String> {
        Ok(Sample {
            at: now_ms(),
            platform: "linux".into(),
            sampler_pid: None,
            procs: self.rows().clone(),
        })
    }
}

impl Killer for Machine {
    fn kill(&self, pid: u32, _tree: bool, _descendants: &[u32]) -> KillResult {
        self.rows().retain(|r| r.pid != pid);
        KillResult {
            ok: true,
            detail: None,
        }
    }

    fn is_alive(&self, pid: u32) -> bool {
        self.alive(pid)
    }
}

/// Points the monitor of a test server at the machine, with no timer.
pub fn install(machine: &Arc<Machine>, opts: &mut ServeOptions) {
    opts.procs.enabled = true;
    opts.procs.timer = false;
    opts.procs.sampler = Some(Arc::clone(machine) as Arc<dyn Sampler>);
    opts.procs.killer = Some(Arc::clone(machine) as Arc<dyn Killer>);
    opts.procs.responder = Some(Arc::new(NoResponder));
}
