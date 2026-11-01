//! The reaper: after each sample it ends, by pid and with its tree, a search left running (an
//! orphan, or one over its age) and a test or build run that hung (idle tree, or past the cap).
//! Only inside a known project folder, never the dashboard, a Claude session or anything the
//! thread kill refuses. Each kill is one activity line, and the project's lead gets one wake per
//! batch, debounced. Shadow mode only writes what it would do.

mod debounce;
mod pick;

use crate::config::setting;
use crate::guard::GuardCtx;
use crate::ports::{Activity, Level, WakeRequest};
use crate::scope::{Scope, refusal};
use crate::self_kill::exe_of;
use crate::service::Inner;
use crate::tree::chain_of;
use crate::types::{ProcRow, Sample};
use debounce::WakeDebounce;
use pick::{Candidate, pick};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// The kind of the reaper's activity lines.
pub const REAP_KIND: &str = "proc-reap";
/// The reason of the reaper's wakes: the thread's own run hung.
pub const REAP_REASON: &str = "process-reaped";

const MINUTE_MS: i64 = 60_000;
const MIN_SETTING_MS: f64 = 1000.0;
/// A run whose tree gained less CPU time than this since its last gain is idle.
const CPU_STEP_MS: i64 = 2000;

type Key = (u32, i64);

/// `procs.reaper` of config.json.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReaperConfig {
    /// `enabled`, on unless `false`.
    pub enabled: bool,
    /// `shadow`: write what it would reap, kill nothing. The server also sets it in its shadow mode.
    pub shadow: bool,
    /// `capMs`: a test or build run older than this is ended (30 minutes).
    pub cap_ms: i64,
    /// `idleMs`: a test or build run whose tree gained no CPU time for this long is ended (5 minutes).
    pub idle_ms: i64,
    /// `searchMs`: a search (find, grep, rg, du) running longer than this is ended (10 minutes).
    pub search_ms: i64,
    /// `wakeDebounceMs`: the least time between two wakes to one thread (10 minutes).
    pub wake_debounce_ms: i64,
}

impl Default for ReaperConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            shadow: false,
            cap_ms: 30 * MINUTE_MS,
            idle_ms: 5 * MINUTE_MS,
            search_ms: 10 * MINUTE_MS,
            wake_debounce_ms: 10 * MINUTE_MS,
        }
    }
}

fn millis(config: &Value, key: &str, default: i64) -> i64 {
    let fallback = u64::try_from(default).unwrap_or(0);
    i64::try_from(setting(config, key, fallback, MIN_SETTING_MS)).unwrap_or(default)
}

impl ReaperConfig {
    /// Reads `procs.reaper`; a missing or bad value keeps its default.
    #[must_use]
    pub fn from_json(config: &Value) -> Self {
        let d = Self::default();
        Self {
            enabled: config.get("enabled") != Some(&Value::Bool(false)),
            shadow: config.get("shadow") == Some(&Value::Bool(true)),
            cap_ms: millis(config, "capMs", d.cap_ms),
            idle_ms: millis(config, "idleMs", d.idle_ms),
            search_ms: millis(config, "searchMs", d.search_ms),
            wake_debounce_ms: millis(config, "wakeDebounceMs", d.wake_debounce_ms),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RunCpu {
    base: i64,
    since: i64,
}

/// What the reaper keeps between samples: the CPU time of each run's tree, the lines already
/// written for a process it could not end (or would end, in shadow mode), and the wakes waiting.
#[derive(Debug, Default)]
pub(crate) struct ReaperState {
    runs: HashMap<Key, RunCpu>,
    noted: HashSet<Key>,
    debounce: WakeDebounce,
}

impl ReaperState {
    /// How long the run's tree has gained no CPU time, given its total now.
    fn idle_for(&mut self, key: Key, cpu: i64, now: i64) -> i64 {
        let run = self.runs.entry(key).or_insert(RunCpu {
            base: cpu,
            since: now,
        });
        if cpu >= run.base + CPU_STEP_MS {
            run.base = cpu;
            run.since = now;
        } else if cpu < run.base {
            run.base = cpu;
        }
        now - run.since
    }

    fn keep_runs(&mut self, live: &HashSet<Key>) {
        self.runs.retain(|key, _| live.contains(key));
    }
}

/// One reaper pass over one sample.
struct Pass<'a> {
    sample: &'a Sample,
    guard: &'a GuardCtx,
    scope: Scope,
    shadow: bool,
}

fn wake_text(lines: &[String]) -> String {
    let what = if lines.len() == 1 {
        "a process tree".to_string()
    } else {
        format!("{} process trees", lines.len())
    };
    format!(
        "Argus reaped {what} of this project that hung: {}. Each was a run of this project (a search, or a test or build run) that was orphaned, idle or past its cap. If it is still needed, run it again scoped to one folder and with a timeout.",
        lines.join("; ")
    )
}

impl Inner {
    fn reap_note(&self, level: Level, text: String) {
        self.ports.host.activity(&Activity {
            kind: REAP_KIND,
            level,
            text,
        });
    }

    /// The project the tree is reaped for, or `None` when it may not be reaped.
    fn reapable(&self, pass: &Pass<'_>, rows: &[ProcRow]) -> Option<String> {
        let root = rows.first()?;
        let parents = chain_of(root.pid, &pass.sample.procs);
        if parents.contains(&self.options.own_pid) {
            return None;
        }
        let project = pass.scope.owner(root)?;
        let windows = pass.sample.is_windows();
        let refused = rows.iter().any(|row| {
            let guard = pass.guard.guard_of(row, windows);
            refusal(row, guard, &pass.scope, &project).is_some()
        });
        (!refused).then_some(project)
    }

    fn reap_one(&self, state: &mut ReaperState, pass: &Pass<'_>, candidate: Candidate) {
        let procs = &pass.sample.procs;
        let kids = candidate
            .kids
            .iter()
            .filter_map(|k| procs.iter().find(|p| p.pid == *k).cloned());
        let mut rows: Vec<ProcRow> = std::iter::once(candidate.root).chain(kids).collect();
        self.fill_cwds(&mut rows);
        let Some(project) = self.reapable(pass, &rows) else {
            return;
        };
        let root = &rows[0];
        let key = (root.pid, root.start_ms);
        let tree = match candidate.kids.len() {
            0 => String::new(),
            n => format!(" and its tree ({n} more)"),
        };
        let line = format!("{} {}{tree}: {}", exe_of(root), root.pid, candidate.why);
        if pass.shadow {
            if state.noted.insert(key) {
                let text =
                    format!("would reap {line} (project {project}; shadow mode, nothing killed)");
                self.reap_note(Level::Info, text);
            }
            return;
        }
        let result = self.ports.killer.kill(root.pid, true, &candidate.kids);
        if !result.ok {
            if state.noted.insert(key) {
                let detail = result.detail.as_deref().unwrap_or("refused");
                let text = format!("could not reap {line} (project {project}): {detail}");
                self.reap_note(Level::Error, text);
            }
            return;
        }
        let closing = if self.wait_gone(root.pid) {
            ""
        } else {
            ", it may still be closing"
        };
        self.reap_note(
            Level::Warn,
            format!("reaped {line} (project {project}{closing})"),
        );
        if let Some(lead) = self.ports.host.lead_of(&project) {
            state.debounce.push(&project, &lead, line);
        }
    }

    fn send_reaped(&self, state: &mut ReaperState, now: i64, window_ms: i64) {
        let host = &self.ports.host;
        for (project, thread, lines) in state.debounce.due(now, window_ms) {
            let wake = WakeRequest {
                scope: project,
                thread,
                reason: REAP_REASON.to_string(),
                subject: "procs".to_string(),
                text: wake_text(&lines),
            };
            if let Err(error) = host.submit_wake(&wake) {
                host.warn("reaper wake not sent", &error);
            }
        }
    }

    /// One reaper pass over a sample, after it was evaluated.
    pub(crate) fn reap(&self, state: &mut ReaperState, sample: &Sample, guard: &GuardCtx) {
        let cfg = self.options.config.reaper;
        if !cfg.enabled {
            return;
        }
        let picked = pick(sample, state, &cfg);
        let seen: HashSet<Key> = sample.procs.iter().map(|p| (p.pid, p.start_ms)).collect();
        state.noted.retain(|key| seen.contains(key));
        if !picked.is_empty() {
            let pass = Pass {
                sample,
                guard,
                scope: Scope::new(&self.ports.host.project_folders()),
                shadow: cfg.shadow,
            };
            for candidate in picked {
                self.reap_one(state, &pass, candidate);
            }
        }
        self.send_reaped(state, sample.at, cfg.wake_debounce_ms);
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_settings_read_with_their_defaults() {
        let cfg =
            ReaperConfig::from_json(&json!({"enabled": false, "capMs": 600_000, "idleMs": "bad"}));
        assert!(!cfg.enabled && !cfg.shadow);
        assert_eq!(cfg.cap_ms, 600_000);
        assert_eq!(cfg.idle_ms, 5 * MINUTE_MS);
        assert_eq!(
            ReaperConfig::from_json(&Value::Null),
            ReaperConfig::default()
        );
        assert!(ReaperConfig::from_json(&json!({"shadow": true})).shadow);
    }

    #[test]
    fn a_run_is_idle_from_its_last_real_cpu_gain() {
        let mut state = ReaperState::default();
        let key = (7, 0);
        assert_eq!(state.idle_for(key, 1000, 0), 0);
        assert_eq!(state.idle_for(key, 1500, MINUTE_MS), MINUTE_MS);
        assert_eq!(state.idle_for(key, 9000, 2 * MINUTE_MS), 0);
        assert_eq!(state.idle_for(key, 9000, 7 * MINUTE_MS), 5 * MINUTE_MS);
    }
}
