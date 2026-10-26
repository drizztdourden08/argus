//! What the monitor remembers between samples: CPU and idle time per process, and the status it is in.

use crate::num::{round1, to_f64};
use crate::types::{ProcRow, Status};
use std::collections::{HashMap, HashSet};

/// A process under this share of one core counts as idle.
pub const IDLE_CPU_PCT: f64 = 0.5;
const START_SLACK_MS: i64 = 2000;

type Key = (u32, i64);

#[derive(Debug, Clone)]
struct ProcState {
    start_ms: i64,
    cpu_ms: Option<i64>,
    at: Option<i64>,
    idle_since: Option<i64>,
    status: Option<Status>,
    status_since: i64,
}

/// What one sample learned about a process, against the one before.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tracked {
    pub key: (u32, i64),
    pub start_ms: i64,
    pub cpu_pct: Option<f64>,
    pub idle_since: Option<i64>,
}

/// Per-process memory of the monitor.
#[derive(Debug, Default)]
pub struct Tracker {
    procs: HashMap<Key, ProcState>,
}

fn cpu_share(state: &ProcState, cpu_ms: i64, now: i64) -> Option<f64> {
    let (before, at) = (state.cpu_ms?, state.at?);
    (now > at).then(|| ((to_f64(cpu_ms - before) / to_f64(now - at)) * 100.0).max(0.0))
}

impl Tracker {
    /// The id of a process: `<pid>-<startMs>`.
    #[must_use]
    pub fn id_of(key: (u32, i64)) -> String {
        format!("{}-{}", key.0, key.1)
    }

    fn known_start(&self, proc: &ProcRow) -> i64 {
        if self.procs.contains_key(&(proc.pid, proc.start_ms)) {
            return proc.start_ms;
        }
        self.procs
            .iter()
            .filter(|((pid, _), st)| {
                *pid == proc.pid && (st.start_ms - proc.start_ms).abs() <= START_SLACK_MS
            })
            .map(|(_, st)| st.start_ms)
            .last()
            .unwrap_or(proc.start_ms)
    }

    /// Records the sample's CPU for the process and answers its CPU share and idle start.
    pub fn track(&mut self, proc: &ProcRow, now: i64) -> Tracked {
        let start_ms = self.known_start(proc);
        let key = (proc.pid, start_ms);
        let state = self.procs.entry(key).or_insert(ProcState {
            start_ms,
            cpu_ms: None,
            at: None,
            idle_since: None,
            status: None,
            status_since: now,
        });
        let cpu_pct = cpu_share(state, proc.cpu_ms, now);
        if let Some(pct) = cpu_pct {
            state.idle_since = (pct < IDLE_CPU_PCT)
                .then(|| state.idle_since.or(state.at))
                .flatten();
        }
        state.cpu_ms = Some(proc.cpu_ms);
        state.at = Some(now);
        Tracked {
            key,
            start_ms,
            cpu_pct: cpu_pct.map(round1),
            idle_since: state.idle_since,
        }
    }

    /// Notes the status a listed process is in and answers when that status began.
    pub fn settle(&mut self, key: (u32, i64), status: Status, now: i64) -> i64 {
        let Some(state) = self.procs.get_mut(&key) else {
            return now;
        };
        if state.status != Some(status) {
            state.status = Some(status);
            state.status_since = now;
        }
        state.status_since
    }

    /// Forgets the processes the sample no longer shows.
    pub fn retain(&mut self, seen: &HashSet<(u32, i64)>) {
        self.procs.retain(|key, _| seen.contains(key));
    }
}
