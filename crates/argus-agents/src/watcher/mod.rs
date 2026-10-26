//! Live view of the sub-agents. Each sub-agent writes its own transcript while it works; the
//! `SubagentStart` hook stores its path on the agents record (`transcriptFile`). The watcher tails
//! those files and keeps, in memory only, a live state per project and agent id: the last message,
//! the tool in use, what it is doing now, counters, tokens, the model it really runs on and
//! whether it has gone quiet.
//!
//! The state is overlaid on the exported agents docs under `live` ([`AgentWatcher::overlay_doc`]),
//! and the watcher itself sets a changed doc at most once a second per agent, so the page updates
//! through its live docs and never polls. The board file is never touched.

mod live;
mod tail;
mod tracked;

pub use live::{LastTool, Live, Tokens};

use crate::config::WatcherOptions;
use crate::iso::parse_iso;
use crate::ports::{LiveDocs, Ports};
use crate::record::{Record, filled, id_of, is_live, ms_of, text};
use crate::schedule::Tick;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, PoisonError};
use tail::{Tail, tail};
use tracked::Tracked;

const NOTE_MISSING: &str = "no transcript found";
const EMIT_SLACK_MS: i64 = 50;

/// The transcript watcher.
pub struct AgentWatcher {
    ports: Ports,
    docs: Arc<dyn LiveDocs>,
    opts: WatcherOptions,
    states: Mutex<HashMap<String, Tracked>>,
}

fn state_key(project: &str, id: &str) -> String {
    format!("{project}/{id}")
}

impl AgentWatcher {
    /// A watcher over the boards and docs of the ports.
    #[must_use]
    pub fn new(ports: Ports, docs: Arc<dyn LiveDocs>, opts: WatcherOptions) -> Self {
        Self {
            ports,
            docs,
            opts,
            states: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Tracked>> {
        self.states.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The number of agents tracked.
    #[must_use]
    pub fn size(&self) -> usize {
        self.lock().len()
    }

    /// The live state of an agent, `None` while it shows nothing.
    #[must_use]
    pub fn live(&self, project: &str, id: &str) -> Option<Live> {
        let states = self.lock();
        let tracked = states.get(&state_key(project, id))?;
        tracked.visible().then(|| tracked.live.clone())
    }

    /// Puts the live state on the data of an agents doc about to be stored. Returns whether it did.
    pub fn overlay_doc(&self, project: &str, id: &str, data: &mut Record) -> bool {
        let mut states = self.lock();
        let Some(tracked) = states.get_mut(&state_key(project, id)) else {
            return false;
        };
        if !tracked.visible() {
            return false;
        }
        let snapshot = serde_json::to_value(&tracked.live).unwrap_or(Value::Null);
        tracked.published = Some(snapshot.to_string());
        data.insert("live".into(), snapshot);
        true
    }

    /// One poll of every tracked agent of every project.
    pub fn tick(&self) {
        let now = self.ports.clock.now_ms();
        let mut states = self.lock();
        let mut seen = HashSet::new();
        for project in self.ports.store.projects() {
            let Some(board) = self.ports.store.snapshot(&project) else {
                continue;
            };
            for agent in &board.agents {
                self.poll_agent(&mut states, &project, agent, now, &mut seen);
            }
        }
        states.retain(|key, _| seen.contains(key));
    }

    fn poll_agent(
        &self,
        states: &mut HashMap<String, Tracked>,
        project: &str,
        agent: &Record,
        now: i64,
        seen: &mut HashSet<String>,
    ) {
        let Some(file) = filled(agent, "transcriptFile") else {
            return;
        };
        let key = state_key(project, id_of(agent));
        seen.insert(key.clone());
        let live = is_live(agent);
        if states.get(&key).is_some_and(|t| t.file.as_os_str() != file) {
            states.remove(&key);
        }
        if !self.wanted(states.get(&key), agent, live, now) {
            return;
        }
        let tracked = states
            .entry(key)
            .or_insert_with(|| Tracked::new(project, id_of(agent), file.into()));
        self.follow(tracked, agent, live, now);
    }

    fn wanted(&self, tracked: Option<&Tracked>, agent: &Record, live: bool, now: i64) -> bool {
        if live {
            return true;
        }
        if let Some(tracked) = tracked {
            return !tracked.closed;
        }
        let ended = text(agent, "finishedAt")
            .or_else(|| text(agent, "updatedAt"))
            .and_then(parse_iso);
        ended.is_some_and(|ms| now - ms <= self.opts.recent_ms)
    }

    fn follow(&self, tracked: &mut Tracked, agent: &Record, live: bool, now: i64) {
        match tail(tracked, self.ports.fs.as_ref(), &self.opts, now) {
            Tail::Missing => self.note_missing(tracked, agent, live, now),
            Tail::Same | Tail::Read => tracked.live.live_note = None,
        }
        if live {
            tracked.live.stalled = self.stalled(tracked, agent, now);
        } else {
            tracked.closed = true;
            tracked.live.stalled = false;
        }
        self.publish(tracked, now, !live);
    }

    fn note_missing(&self, tracked: &mut Tracked, agent: &Record, live: bool, now: i64) {
        let began = ms_of(agent, "startedAt");
        let late = began.is_some_and(|b| now - b >= self.opts.missing_ms);
        if live && !tracked.noted && late {
            tracked.noted = true;
            tracked.live.live_note = Some(NOTE_MISSING.to_owned());
        }
    }

    fn stalled(&self, tracked: &Tracked, agent: &Record, now: i64) -> bool {
        if text(agent, "status") != Some("working") || tracked.closed {
            return false;
        }
        let last = tracked
            .live
            .last_activity_at
            .as_deref()
            .or_else(|| text(agent, "startedAt"));
        let base = last.and_then(parse_iso);
        base.is_some_and(|ms| now - ms > self.opts.stall_ms)
    }

    fn publish(&self, tracked: &mut Tracked, now: i64, force: bool) {
        if !tracked.visible() {
            return;
        }
        let snapshot = serde_json::to_value(&tracked.live).unwrap_or(Value::Null);
        let json = snapshot.to_string();
        if tracked.published.as_deref() == Some(json.as_str()) {
            return;
        }
        if !force && now - tracked.emitted_at < self.opts.emit_ms - EMIT_SLACK_MS {
            return;
        }
        let Some(mut data) = self.docs.get(&tracked.project, &tracked.id) else {
            return;
        };
        data.insert("live".into(), snapshot);
        if let Err(err) = self.docs.set(&tracked.project, &tracked.id, data) {
            tracing::warn!(project = %tracked.project, id = %tracked.id, %err, "live agent update failed");
            return;
        }
        tracked.published = Some(json);
        tracked.emitted_at = now;
    }
}

impl Tick for AgentWatcher {
    fn period_ms(&self) -> u64 {
        u64::try_from(self.opts.poll_ms).unwrap_or(1000)
    }

    fn run_once(&self) {
        self.tick();
    }
}
