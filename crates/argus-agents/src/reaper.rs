//! Closes the sub-agents that went quiet. A hook-recorded agent normally ends with its
//! `SubagentStop`; one that never gets it (the session died, the hook never ran) would stay
//! `working` for good. Every poll this looks at the agents that are working or waiting and have a
//! transcript (`transcriptFile`, which only hook-recorded agents have): when the file has not grown
//! for longer than `agents.reapMs` (default 20 minutes), the agent is closed as `done` with the note
//! `no activity for N min, closed automatically`, and `reaped` and `closedBy: reaper` are set so a
//! `SubagentStop` that does arrive later still fills in the real answer.
//!
//! A quiet transcript alone does not prove an agent idle: a long tool call or a big file write does not touch it
//! (an opus agent was closed this way while it was writing files). Before closing, the reaper therefore looks at the
//! same evidence the reconciler uses (`evidence`): the agent's task state by its Claude id (the transcript's turn and
//! the background task's output file) and, when the record names a worktree, the newest file in it. The agent is
//! closed only when every one of them has been quiet for the limit. The limit is `agents.reapMs`, and for an
//! opus-level agent (anything but haiku and sonnet) at least `agents.reapOpusMs` (default 60 minutes). A closed
//! record that turns out to be alive is reopened by the reconciler; a really dead one is still reported by the
//! `agent-stale` wake, which does not wait for the reaper.
//!
//! An agent with no transcript (a manual `coord agent start` record) is never closed here: nothing
//! shows that it is dead. A transcript that does not exist counts from the agent's start.
//!
//! The write goes through the board store under the board lock, stamped `dashboard`. A poll that
//! comes much later than the last one (the machine slept) is skipped: the transcripts stopped
//! growing because the agents were frozen too, not because they died.

use crate::board::Board;
use crate::config::{MINUTE_MS, ReaperOptions, ReconcilerOptions};
use crate::evidence::Env;
use crate::evidence::session::Session;
use crate::evidence::sign::{Look, last_sign};
use crate::holds::release;
use crate::iso::format_iso;
use crate::model_rules::high_tier;
use crate::ports::{ActivityLine, Ports};
use crate::record::{Record, filled, id_of, is_live, ms_of, stamp, text};
use crate::schedule::Tick;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

const SLEEP_FACTOR: i64 = 3;
const ACTOR: &str = "dashboard";

/// The agent reaper.
pub struct AgentReaper {
    ports: Ports,
    opts: ReaperOptions,
    evidence: ReconcilerOptions,
    project_dirs: Mutex<HashMap<String, PathBuf>>,
    last_tick: Mutex<Option<i64>>,
    gate: Mutex<()>,
}

impl AgentReaper {
    /// A reaper over the boards of the ports.
    #[must_use]
    pub fn new(ports: Ports, opts: ReaperOptions) -> Self {
        let evidence = ReconcilerOptions {
            stall_ms: opts.stall_ms,
            claude_home: opts.claude_home.clone(),
            claude_temp: opts.claude_temp.clone(),
            ..ReconcilerOptions::default()
        };
        Self {
            ports,
            opts,
            evidence,
            project_dirs: Mutex::new(HashMap::new()),
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

    /// The quiet time that closes an agent: opus-level agents (long tool calls, big writes) get at least `reap_opus_ms`.
    fn limit_of(&self, agent: &Record) -> i64 {
        let model = text(agent, "modelUsed")
            .or_else(|| text(agent, "model"))
            .unwrap_or_default();
        if high_tier(model) {
            self.opts.reap_ms.max(self.opts.reap_opus_ms)
        } else {
            self.opts.reap_ms
        }
    }

    /// How long the agent has shown no sign of life at `at`, or `None` when it is not a candidate: the transcript alone
    /// says it is quiet; Claude's task data (turn, background task output) and the record's worktree must agree.
    fn idle_for(&self, agent: &Record, at: i64, board: &Board) -> Option<i64> {
        let reapable = matches!(text(agent, "status"), Some("working" | "waiting"));
        if filled(agent, "transcriptFile").is_none() || !reapable || !is_live(agent) {
            return None;
        }
        let limit = self.limit_of(agent);
        let quiet = self.quiet_for(agent, at);
        if quiet <= limit {
            return None;
        }
        let env = Env {
            opts: &self.evidence,
            fs: self.ports.fs.as_ref(),
            project_dirs: &self.project_dirs,
            boot_at: 0,
        };
        let session = Session::open(&env, board, text(agent, "thread"));
        let look = Look {
            need_beyond: Some(limit),
            claude_id: None,
        };
        let sign = last_sign(&env, &session, board, agent, at, look);
        if sign.running {
            return None;
        }
        let idle = if sign.last_ms > 0 {
            quiet.min(at - sign.last_ms)
        } else {
            quiet
        };
        (idle > limit).then_some(idle)
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
            .filter(|a| self.idle_for(a, at, &mirror).is_some())
            .map(|a| id_of(a).to_owned())
            .collect();
        if ids.is_empty() {
            return Vec::new();
        }
        let mut lines = Vec::new();
        let mut closed = Vec::new();
        let result = self.ports.store.write(project, &mut |board| {
            for id in &ids {
                let idle = board
                    .agents
                    .iter()
                    .find(|a| id_of(a) == id)
                    .and_then(|a| self.idle_for(a, at, board));
                let (Some(idle), Some(agent)) = (idle, board.agent_mut(id)) else {
                    continue;
                };
                lines.push(Self::close(project, agent, idle, at));
                release(board, id, at);
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

    fn close(project: &str, agent: &mut Record, idle_ms: i64, at: i64) -> ActivityLine {
        let quiet = idle_ms / MINUTE_MS;
        agent.insert("status".into(), Value::from("done"));
        agent.insert("finishedAt".into(), Value::from(format_iso(at)));
        agent.insert(
            "note".into(),
            Value::from(format!("no activity for {quiet} min, closed automatically")),
        );
        agent.insert("reaped".into(), Value::Bool(true));
        agent.insert("closedBy".into(), Value::from("reaper"));
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
