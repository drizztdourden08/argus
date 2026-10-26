//! The reconciler service: one poll of every project, the write under the board lock, the wakes
//! after it, and the report of the run.

use super::apply::apply_patches;
use super::pass::{Outcome, Pass};
use super::report::{Class, Counts, Finding, RunReport};
use crate::board::Board;
use crate::config::ReconcilerOptions;
use crate::error::AgentsError;
use crate::evidence::Env;
use crate::iso::format_iso;
use crate::ports::{ActivityLine, Ports, WakeEvent};
use crate::schedule::Tick;
use crate::text::clip;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

const SLEEP_FACTOR: i64 = 3;
const FAULT_MAX: usize = 300;

#[derive(Default)]
struct Run {
    projects: BTreeMap<String, Counts>,
    findings: Vec<Finding>,
    wakes: Vec<WakeEvent>,
    lost: Vec<(String, String)>,
    lines: Vec<ActivityLine>,
}

#[derive(Default)]
struct History {
    last_tick: Option<i64>,
    last_run: Option<RunReport>,
}

/// The agent reconciler.
pub struct AgentReconciler {
    ports: Ports,
    opts: ReconcilerOptions,
    boot_at: i64,
    project_dirs: Mutex<HashMap<String, PathBuf>>,
    history: Mutex<History>,
    gate: Mutex<()>,
}

fn agent_line(project: &str, finding: &Finding) -> ActivityLine {
    let verb = if finding.class == Class::Reopened {
        "reopened"
    } else {
        "closed"
    };
    let name = finding
        .name
        .as_deref()
        .map_or_else(String::new, |n| format!(" ({})", clip(n, 40)));
    ActivityLine {
        kind: "agent".into(),
        level: "info".into(),
        text: format!(
            "reconciler {verb} {project}/{}{name}: {}",
            finding.id, finding.detail
        ),
        project: project.to_owned(),
        thread: finding.thread.clone(),
    }
}

fn wake_line(wake: &WakeEvent) -> ActivityLine {
    let why = match wake.reason.as_str() {
        "agent-orphan" => "Claude has no task for it",
        "agent-idle" => "its turn ended",
        other => other,
    };
    ActivityLine {
        kind: "agent".into(),
        level: "info".into(),
        text: format!(
            "reconciler woke {}/{} about {}: {why}",
            wake.scope, wake.thread, wake.subject
        ),
        project: wake.scope.clone(),
        thread: Some(wake.thread.clone()),
    }
}

impl AgentReconciler {
    /// A reconciler over the boards of the ports. The closing clocks start now.
    #[must_use]
    pub fn new(ports: Ports, opts: ReconcilerOptions) -> Self {
        let boot_at = ports.clock.now_ms();
        Self {
            ports,
            opts,
            boot_at,
            project_dirs: Mutex::new(HashMap::new()),
            history: Mutex::new(History::default()),
            gate: Mutex::new(()),
        }
    }

    /// The report of the last run.
    #[must_use]
    pub fn last(&self) -> Option<RunReport> {
        let history = self.history.lock().unwrap_or_else(PoisonError::into_inner);
        history.last_run.clone()
    }

    fn evaluate(&self, project: &str, board: &Board, at: i64, can_wake: bool) -> Outcome {
        let env = Env {
            opts: &self.opts,
            fs: self.ports.fs.as_ref(),
            project_dirs: &self.project_dirs,
            boot_at: self.boot_at,
        };
        let mut pass = Pass::new(&env, project, board, at, can_wake);
        pass.run();
        pass.finish()
    }

    fn slept(&self, at: i64) -> bool {
        let mut history = self.history.lock().unwrap_or_else(PoisonError::into_inner);
        let gap = history.last_tick.map_or(0, |last| at - last);
        history.last_tick = Some(at);
        gap > self.opts.reconcile_ms * SLEEP_FACTOR
    }

    /// One poll of every project. Returns the report of the run, also kept as [`Self::last`].
    pub fn tick(&self) -> RunReport {
        let Ok(_gate) = self.gate.try_lock() else {
            return self.last().unwrap_or_default();
        };
        let started = self.ports.clock.now_ms();
        let asleep = self.slept(started);
        let mut run = Run::default();
        for project in self.ports.store.projects() {
            self.reconcile_project(&project, started, asleep, &mut run);
        }
        self.send_wakes(&mut run);
        for line in std::mem::take(&mut run.lines) {
            self.ports.log_activity(line);
        }
        self.tell_main(&run.lost);
        let report = RunReport {
            at: format_iso(started),
            ms: self.ports.clock.now_ms() - started,
            skipped: asleep,
            projects: run.projects.clone(),
            findings: run
                .findings
                .iter()
                .take(self.opts.max_findings)
                .cloned()
                .collect(),
        };
        if run.projects.values().any(Counts::acted) {
            tracing::info!(projects = ?run.projects, wakes = run.wakes.len(), lost = run.lost.len(), "agents reconciled");
        }
        let mut history = self.history.lock().unwrap_or_else(PoisonError::into_inner);
        history.last_run = Some(report.clone());
        report
    }

    fn reconcile_project(&self, project: &str, at: i64, asleep: bool, run: &mut Run) {
        let Some(mirror) = self.ports.store.snapshot(project) else {
            return;
        };
        if mirror.agents.is_empty() {
            return;
        }
        let can_wake = self.ports.wakes.is_some() && !asleep;
        let mut outcome = self.evaluate(project, &mirror, at, can_wake);
        if !outcome.patches.is_empty() && !asleep {
            match self.write_through(project, at) {
                Ok(written) => outcome = written,
                Err(err) => {
                    tracing::warn!(project, %err, "agent reconcile failed");
                    return;
                }
            }
            run.wakes.extend(outcome.wakes.iter().cloned());
            run.lost
                .extend(outcome.lost.iter().map(|l| (project.to_owned(), l.clone())));
            let classes = [Class::Closed, Class::Duplicate, Class::Reopened];
            run.lines.extend(
                outcome
                    .findings
                    .iter()
                    .filter(|f| classes.contains(&f.class))
                    .map(|f| agent_line(project, f)),
            );
        }
        run.projects.insert(project.to_owned(), outcome.counts);
        run.findings.extend(outcome.findings);
    }

    /// Decides again on the board as the lock found it, and writes the decisions.
    fn write_through(&self, project: &str, at: i64) -> Result<Outcome, AgentsError> {
        let can_wake = self.ports.wakes.is_some();
        let mut slot = None;
        self.ports.store.write(project, &mut |board| {
            let outcome = self.evaluate(project, board, at, can_wake);
            apply_patches(board, &outcome.patches, at);
            slot = Some(outcome);
        })?;
        slot.ok_or_else(|| AgentsError::Board(project.to_owned(), "no change was run".into()))
    }

    fn send_wakes(&self, run: &mut Run) {
        let Some(engine) = &self.ports.wakes else {
            return;
        };
        if run.wakes.is_empty() {
            return;
        }
        match engine.submit(&run.wakes) {
            Ok(()) => run.lines.extend(run.wakes.iter().map(wake_line)),
            Err(err) => tracing::warn!(%err, "agent reconcile wakes not sent"),
        }
    }

    /// The main coordinator hears one line when records were closed as lost.
    fn tell_main(&self, lost: &[(String, String)]) {
        let Some(engine) = &self.ports.wakes else {
            return;
        };
        if lost.is_empty() {
            return;
        }
        let main = self
            .ports
            .store
            .snapshot("portfolio")
            .and_then(|b| b.coordinator())
            .unwrap_or_else(|| "coordinator".to_owned());
        let mut by: Vec<&str> = Vec::new();
        for (project, _) in lost {
            if !by.contains(&project.as_str()) {
                by.push(project);
            }
        }
        let names: Vec<String> = lost.iter().map(|(p, what)| format!("{p}/{what}")).collect();
        let text = format!(
            "{}: the agent reconciler closed {} record(s) as lost, no Claude task found ({}); their threads were told first. GET /api/agents/reconcile has the run.",
            by.join(", "),
            lost.len(),
            names.join(", ")
        );
        let event = WakeEvent {
            scope: "portfolio".into(),
            thread: main,
            reason: "wake-fault".into(),
            subject: "agent-reconciler".into(),
            text: clip(&text, FAULT_MAX),
        };
        if let Err(err) = engine.submit(&[event]) {
            tracing::warn!(%err, "agent reconcile main wake not sent");
        }
    }
}

impl Tick for AgentReconciler {
    fn period_ms(&self) -> u64 {
        u64::try_from(self.opts.reconcile_ms).unwrap_or(120_000)
    }

    fn run_once(&self) {
        self.tick();
    }
}
