//! The real kill: `taskkill` on Windows, SIGTERM elsewhere. Only the dashboard's guarded kill paths call it.

use crate::ports::{KillResult, Killer};
use crate::shell::{TIMEOUT, run_hidden};
use sysinfo::{Pid, ProcessesToUpdate, Signal, System};

/// Ends processes through the operating system.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemKiller;

fn failure(text: &str) -> KillResult {
    let detail: String = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(200)
        .collect();
    KillResult {
        ok: false,
        detail: Some(detail),
    }
}

fn taskkill(pid: u32, tree: bool) -> KillResult {
    let mut args = vec!["/PID".to_string(), pid.to_string()];
    if tree {
        args.push("/T".to_string());
    }
    args.push("/F".to_string());
    match run_hidden("taskkill", &args, TIMEOUT) {
        Ok(out) if out.ok => KillResult {
            ok: true,
            detail: None,
        },
        Ok(out) => failure(if out.stderr.trim().is_empty() {
            &out.stdout
        } else {
            &out.stderr
        }),
        Err(message) => failure(&message),
    }
}

fn terminate(pids: &[u32]) -> KillResult {
    let ids: Vec<Pid> = pids.iter().map(|p| Pid::from_u32(*p)).collect();
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&ids), true);
    let Some(target) = pids.last().and_then(|p| system.process(Pid::from_u32(*p))) else {
        return failure("it is gone already");
    };
    for id in &ids[..ids.len() - 1] {
        if let Some(process) = system.process(*id) {
            process.kill_with(Signal::Term);
        }
    }
    match target.kill_with(Signal::Term) {
        Some(true) => KillResult {
            ok: true,
            detail: None,
        },
        _ => failure("the signal was refused"),
    }
}

impl Killer for SystemKiller {
    fn kill(&self, pid: u32, tree: bool, descendants: &[u32]) -> KillResult {
        if cfg!(windows) {
            return taskkill(pid, tree);
        }
        let mut pids: Vec<u32> = if tree {
            descendants.to_vec()
        } else {
            Vec::new()
        };
        pids.push(pid);
        terminate(&pids)
    }

    fn is_alive(&self, pid: u32) -> bool {
        let id = Pid::from_u32(pid);
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::Some(&[id]), true);
        system.process(id).is_some()
    }
}
