//! Wakes about the sub-agents a thread runs, so nobody has to look at the board to notice them.
//!
//! * `agent-stale`: a working or waiting agent has shown no sign of life for `stale_wake_ms` (default 20
//!   minutes: no transcript line, no update on its record). The thread that owns it is woken to check it
//!   out, and again every `stale_repeat_ms` (30 minutes) while it stays that way.
//! * `agent-checkin`: an agent on a model of opus level or above (anything that is not haiku or sonnet)
//!   wakes its thread every `checkinMin` minutes (15 to 30, the lead's choice, default 20). The clock runs
//!   from its start, or from when the dashboard started when that was later.
//!
//! A record the reconciler flagged (`orphanSince`, `idleSince`) gets neither: its own wakes say it better.
//! The state is in memory only: a restart forgets what was sent.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use serde_json::Value;

use crate::board::{AgentView, BoardView, LIVE_AGENT};
use crate::policy::coordinator_in;
use crate::source::{AgentLive, BoardSource};
use crate::text::cut;
use crate::time::{ms_or_zero, parse_ms};
use crate::wake::WakeInput;

/// The shortest check-in period a lead may choose, in minutes.
pub const CHECKIN_MIN: f64 = 15.0;
/// The longest check-in period a lead may choose, in minutes.
pub const CHECKIN_MAX: f64 = 30.0;

/// The periods of the agent wakes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentWakeConfig {
    /// How often the owner calls `tick`.
    pub tick_ms: i64,
    /// Quiet this long and the thread is woken.
    pub stale_wake_ms: i64,
    /// A quiet agent wakes its thread again after this long.
    pub stale_repeat_ms: i64,
    /// The default check-in period.
    pub checkin_ms: i64,
}

impl Default for AgentWakeConfig {
    fn default() -> Self {
        Self {
            tick_ms: 60_000,
            stale_wake_ms: 20 * 60_000,
            stale_repeat_ms: 30 * 60_000,
            checkin_ms: 20 * 60_000,
        }
    }
}

impl AgentWakeConfig {
    /// The periods from the `agents` section of config.json; a value that is not a positive number keeps its default.
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let d = Self::default();
        let num = |key: &str, default: i64| crate::config::positive_ms(section.get(key), default);
        Self {
            tick_ms: num("tickMs", d.tick_ms),
            stale_wake_ms: num("staleWakeMs", d.stale_wake_ms),
            stale_repeat_ms: num("staleRepeatMs", d.stale_repeat_ms),
            checkin_ms: num("checkinMs", d.checkin_ms),
        }
    }
}

/// Is the agent on a model above haiku and sonnet? The model it really runs on wins.
#[must_use]
pub fn high_tier(agent: &AgentView) -> bool {
    let name = agent
        .model_used
        .some()
        .or_else(|| agent.model.some())
        .unwrap_or("")
        .to_lowercase();
    !name.is_empty() && !name.contains("haiku") && !name.contains("sonnet")
}

/// The check-in period of an agent: the lead's choice clamped to 15 to 30 minutes, else the default.
#[must_use]
pub fn checkin_every(agent: &AgentView, default_ms: i64) -> i64 {
    agent.checkin_min.get().map_or(default_ms, |minutes| {
        let period = Duration::from_secs_f64(minutes.clamp(CHECKIN_MIN, CHECKIN_MAX) * 60.0);
        i64::try_from(period.as_millis()).unwrap_or(default_ms)
    })
}

/// `20m`: whole minutes, at least one.
fn mins(ms: i64) -> String {
    format!("{}m", ((ms + 30_000) / 60_000).max(1))
}

#[derive(Clone, Copy, Default)]
struct Sent {
    stale: i64,
    checkin: i64,
}

/// The agent reminders.
pub struct AgentWakes {
    config: AgentWakeConfig,
    started_at: i64,
    sent: HashMap<String, Sent>,
}

fn target_of(board: &BoardView, agent: &AgentView) -> Option<String> {
    let owner = agent.thread.some().filter(|t| {
        board
            .threads
            .get(*t)
            .is_some_and(crate::board::ThreadView::is_live)
    });
    owner
        .map(str::to_owned)
        .or_else(|| coordinator_in(Some(board)))
}

fn who(agent: &AgentView) -> String {
    let name = if agent.name.some().is_some() {
        format!("{} ", cut(agent.name.as_str(), 40))
    } else {
        String::new()
    };
    let model = agent
        .model_used
        .some()
        .or_else(|| agent.model.some())
        .unwrap_or("model unknown");
    format!("{} {name}({model})", agent.rec.id.as_str())
}

impl AgentWakes {
    /// Reminders that start counting at `started_at` (epoch ms).
    #[must_use]
    pub fn new(config: AgentWakeConfig, started_at: i64) -> Self {
        Self {
            config,
            started_at,
            sent: HashMap::new(),
        }
    }

    /// The timing.
    #[must_use]
    pub fn config(&self) -> &AgentWakeConfig {
        &self.config
    }

    /// The wakes due now.
    pub fn tick(
        &mut self,
        now: i64,
        boards: &dyn BoardSource,
        live: &dyn AgentLive,
    ) -> Vec<WakeInput> {
        let mut events = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for scope in boards.scopes() {
            let Some(board) = boards.board(&scope) else {
                continue;
            };
            for agent in board.agents.values() {
                let running = agent.status.is_any(&LIVE_AGENT) && agent.rec.is_live();
                if !running {
                    continue;
                }
                let key = format!("{scope}/{}", agent.rec.id.as_str());
                seen.insert(key.clone());
                let activity = live.last_activity_at(&scope, agent.rec.id.as_str());
                events.extend(self.due(now, &scope, &board, agent, &key, activity.as_deref()));
            }
        }
        self.sent.retain(|key, _| seen.contains(key));
        events
    }

    fn due(
        &mut self,
        now: i64,
        scope: &str,
        board: &BoardView,
        agent: &AgentView,
        key: &str,
        activity: Option<&str>,
    ) -> Option<WakeInput> {
        if agent.orphan_since.some().is_some() || agent.idle_since.some().is_some() {
            return None;
        }
        let thread = target_of(board, agent)?;
        let mut rec = self.sent.get(key).copied().unwrap_or_default();
        let started = ms_or_zero(agent.started_at.some());
        let last = ms_or_zero(agent.rec.updated_at.some())
            .max(ms_or_zero(activity))
            .max(started);
        let quiet = if last > 0 { now - last } else { 0 };
        let id = agent.rec.id.as_str();
        let wake = if quiet >= self.config.stale_wake_ms
            && now - rec.stale >= self.config.stale_repeat_ms
        {
            rec.stale = now;
            let text = format!(
                "{} shows no sign of life for {}: check it out (stop or restart it, or close its record).",
                who(agent),
                mins(quiet)
            );
            Some(WakeInput::about(scope, &thread, "agent-stale", id, text))
        } else if high_tier(agent)
            && now - rec.checkin.max(started).max(self.started_at)
                >= checkin_every(agent, self.config.checkin_ms)
        {
            rec.checkin = now;
            Some(WakeInput::about(
                scope,
                &thread,
                "agent-checkin",
                id,
                checkin_text(agent, now),
            ))
        } else {
            None
        };
        self.sent.insert(key.to_owned(), rec);
        wake
    }
}

fn checkin_text(agent: &AgentView, now: i64) -> String {
    let run = parse_ms(agent.started_at.as_str())
        .filter(|s| *s > 0)
        .map_or(String::new(), |s| format!(" running {}", mins(now - s)));
    let task = if agent.task.some().is_some() {
        format!("Task: {}", cut(agent.task.as_str(), 80))
    } else {
        String::new()
    };
    format!(
        "{}{run}: check-in. Still on task and not going overboard (scope, cost, loops)? {task}",
        who(agent)
    )
    .trim()
    .to_owned()
}
