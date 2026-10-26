//! Closes the sub-agents that went quiet. A hook-recorded agent normally ends with its
//! `SubagentStop`; one that never gets it (the session died, the hook never ran) would stay
//! `working` for good. Every poll this looks at the agents that are working or waiting and have a
//! transcript (`transcriptFile`, which only hook-recorded agents have): when the file has not grown
//! for longer than `agents.reapMs` (default 20 minutes), the agent is closed as `done` with the note
//! `no activity for N min, closed automatically`, and `reaped` is set so a `SubagentStop` that does
//! arrive later still fills in the real answer.
//!
//! An agent with no transcript (a manual `coord agent start` record) is never closed here: nothing
//! shows that it is dead. A transcript that does not exist counts from the agent's start.
//!
//! The write goes through the board store under the board lock, stamped `dashboard`. A poll that
//! comes much later than the last one (the machine slept) is skipped: the transcripts stopped
//! growing because the agents were frozen too, not because they died.

use crate::config::{MINUTE_MS, ReaperOptions};
use crate::iso::format_iso;
use crate::ports::{ActivityLine, Ports};
use crate::record::{Record, filled, id_of, is_live, ms_of, stamp, text};
use crate::schedule::Tick;
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

const SLEEP_FACTOR: i64 = 3;
const ACTOR: &str = "dashboard";

/// The agent reaper.
pub struct AgentReaper {
    ports: Ports,
    opts: ReaperOptions,
    last_tick: Mutex<Option<i64>>,
    gate: Mutex<()>,
}

impl AgentReaper {
    /// A reaper over the boards of the ports.
    #[must_use]
    pub fn new(ports: Ports, opts: ReaperOptions) -> Self {
        Self {
            ports,
            opts,
            last_tick: Mutex::new(None),
            gate: Mutex::new(()),
        }
    }

    /// How long the agent's transcript has been quiet at `at`; from its start when the file is not there.
    fn quiet_for(&self, agent: &Record, at: i64) -> i64 {
        let file = filled(agent, "transcriptFile").map(Path::new);
        let base = file
            .and_then(|f| self.ports.fs.stat(f))
            .map(|s| s.mtime_ms)
            .or_else(|| ms_of(agent, "startedAt"));
        base.map_or(0, |ms| at - ms)
    }

    fn due(&self, agent: &Record, at: i64) -> bool {
        let reapable = matches!(text(agent, "status"), Some("working" | "waiting"));
        filled(agent, "transcriptFile").is_some()
            && reapable
            && is_live(agent)
            && self.quiet_for(agent, at) > self.opts.reap_ms
    }

    /// Whether the machine slept since the last poll; records this poll.
    fn slept(&self, at: i64) -> bool {
        let mut last = self
            .last_tick
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let gap = last.map_or(0, |l| at - l);
        *last = Some(at);
        gap > self.opts.poll_ms * SLEEP_FACTOR
    }

    /// One poll of every project. Returns the ids closed, as `project/A-n`.
    pub fn tick(&self) -> Vec<String> {
        let Ok(_gate) = self.gate.try_lock() else {
            return Vec::new();
        };
        let at = self.ports.clock.now_ms();
        if self.slept(at) {
            return Vec::new();
        }
        let mut closed = Vec::new();
        for project in self.ports.store.projects() {
            closed.extend(self.reap_project(&project, at));
        }
        if !closed.is_empty() {
            tracing::info!(agents = ?closed, "quiet agents closed");
        }
        closed
    }

    fn reap_project(&self, project: &str, at: i64) -> Vec<String> {
        let Some(mirror) = self.ports.store.snapshot(project) else {
            return Vec::new();
        };
        let ids: Vec<String> = mirror
            .agents
            .iter()
            .filter(|a| self.due(a, at))
            .map(|a| id_of(a).to_owned())
            .collect();
        if ids.is_empty() {
            return Vec::new();
        }
        let mut lines = Vec::new();
        let mut closed = Vec::new();
        let result = self.ports.store.write(project, &mut |board| {
            for id in &ids {
                let Some(agent) = board.agent_mut(id) else {
                    continue;
                };
                if !self.due(agent, at) {
                    continue;
                }
                lines.push(self.close(project, agent, at));
                closed.push(format!("{project}/{id}"));
            }
        });
        if let Err(err) = result {
            tracing::warn!(project, %err, "quiet agents could not be closed");
            return Vec::new();
        }
        for line in lines {
            self.ports.log_activity(line);
        }
        closed
    }

    fn close(&self, project: &str, agent: &mut Record, at: i64) -> ActivityLine {
        let quiet = self.quiet_for(agent, at) / MINUTE_MS;
        agent.insert("status".into(), Value::from("done"));
        agent.insert("finishedAt".into(), Value::from(format_iso(at)));
        agent.insert(
            "note".into(),
            Value::from(format!("no activity for {quiet} min, closed automatically")),
        );
        agent.insert("reaped".into(), Value::Bool(true));
        stamp(agent, ACTOR, at);
        let name = filled(agent, "name").map_or_else(String::new, |n| {
            format!(" ({})", n.chars().take(40).collect::<String>())
        });
        ActivityLine {
            kind: "agent".into(),
            level: "info".into(),
            text: format!(
                "reaper closed {project}/{}{name}: no activity {quiet} min",
                id_of(agent)
            ),
            project: project.to_owned(),
            thread: text(agent, "thread").map(str::to_owned),
        }
    }
}

impl Tick for AgentReaper {
    fn period_ms(&self) -> u64 {
        u64::try_from(self.opts.poll_ms).unwrap_or(60_000)
    }

    fn run_once(&self) {
        self.tick();
    }
}
