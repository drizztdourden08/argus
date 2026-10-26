//! Two wakes about the launchers of a project (DESIGN.md 13.7), from one tick service
//! (every 10 minutes):
//!
//! - `launcher-master-missing`: a project that has a board, was registered at least
//!   `master_wait_ms` ago and has no live `master` launcher: its lead is woken once on first
//!   detection, and again every `master_repeat_ms` while it stays missing.
//! - `launcher-stale`: a standalone launcher unused for longer than its `useEvery` window.
//!   The thread that made it (the lead when that thread ended) gets ONE wake per day listing
//!   them, worst first; it analyses: still needed? keep it, retire it or change the window.
//!
//! Both go through the wake engine like any other (coalescing window, queued replay);
//! neither is immediate. The last wake of a kind is asked of the wake store, so a restart
//! of the dashboard does not send it again at once.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde_json::Value;
use tokio::task::JoinHandle;

use super::join::has_master;
use super::record::{Launcher, LauncherBoard};
use super::stale_text::{StaleItem, master_text, stale_text};
use super::usage::{Usage, usage_of};
use crate::clock::Clock;
use crate::time::{format_iso, parse_iso};

pub const REASON_STALE: &str = "launcher-stale";
pub const REASON_MASTER: &str = "launcher-master-missing";
const PORTFOLIO: &str = "_portfolio";
const ENDED: [&str; 2] = ["done", "archived"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaleConfig {
    pub tick_ms: i64,
    pub master_wait_ms: i64,
    pub master_repeat_ms: i64,
    pub stale_repeat_ms: i64,
}

impl Default for StaleConfig {
    fn default() -> Self {
        Self {
            tick_ms: 600_000,
            master_wait_ms: 600_000,
            master_repeat_ms: 86_400_000,
            stale_repeat_ms: 86_400_000,
        }
    }
}

/// A wake for the engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WakeEvent {
    pub scope: String,
    pub thread: String,
    pub reason: &'static str,
    pub subject: String,
    pub text: String,
}

/// An entry for the activity log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaleNote {
    pub level: &'static str,
    pub text: String,
    pub project: String,
    pub thread: Option<String>,
}

/// Everything the wakes read and the one thing they do.
pub trait StaleEnv: Send + Sync {
    /// The projects that have a board.
    fn scopes(&self) -> Vec<String>;
    fn board(&self, scope: &str) -> Option<LauncherBoard>;
    /// The lead thread of a project.
    fn coordinator(&self, scope: &str) -> Option<String>;
    /// The `queuedAt` (ms) of the newest Play run of `launcher-<name>`; 0 when unknown.
    fn played_ms(&self, scope: &str, name: &str) -> i64;
    /// When the wake store last sent this wake (ms); 0 when never.
    fn last_sent_ms(&self, scope: &str, thread: &str, reason: &str, subject: &str) -> i64;
    /// Hands wakes to the engine.
    fn submit(&self, events: Vec<WakeEvent>) -> Result<(), String>;
    fn note(&self, _note: StaleNote) {}
}

/// A live standalone launcher with its usage and the thread to tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LauncherUsage {
    pub scope: String,
    pub thread: Option<String>,
    pub name: String,
    pub usage: Usage,
}

pub struct LauncherStale {
    env: Arc<dyn StaleEnv>,
    clock: Arc<dyn Clock>,
    config: StaleConfig,
    sent: Mutex<HashMap<String, i64>>,
}

fn time_of(iso: Option<&str>) -> i64 {
    iso.and_then(parse_iso).unwrap_or(0)
}

fn natural_key(name: &str) -> Vec<(u64, String)> {
    let chars: Vec<char> = name.chars().collect();
    chars
        .chunk_by(|a, b| a.is_ascii_digit() == b.is_ascii_digit())
        .map(|run| {
            let text: String = run.iter().collect();
            if run[0].is_ascii_digit() {
                (text.parse().unwrap_or(0), String::new())
            } else {
                (0, text)
            }
        })
        .collect()
}

/// Numbers inside names compare as numbers (`build-2` before `build-10`).
fn natural_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    natural_key(a).cmp(&natural_key(b))
}

impl LauncherStale {
    pub fn new(env: Arc<dyn StaleEnv>, clock: Arc<dyn Clock>, config: StaleConfig) -> Self {
        Self {
            env,
            clock,
            config,
            sent: Mutex::new(HashMap::new()),
        }
    }

    fn sent(&self) -> std::sync::MutexGuard<'_, HashMap<String, i64>> {
        self.sent.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn last_of(&self, reason: &str, scope: &str, thread: &str, subject: &str) -> i64 {
        let remembered = self
            .sent()
            .get(&format!("{reason}/{scope}/{thread}"))
            .copied()
            .unwrap_or(0);
        remembered.max(self.env.last_sent_ms(scope, thread, reason, subject))
    }

    fn remember(&self, reason: &str, scope: &str, thread: &str, now: i64) {
        self.sent()
            .insert(format!("{reason}/{scope}/{thread}"), now);
    }

    /// The thread that made the launcher, or the project's lead when that one ended, is
    /// gone or is the owner.
    fn owner_of(&self, scope: &str, board: &LauncherBoard, rec: &Launcher) -> Option<String> {
        let maker = rec
            .created_by
            .as_ref()
            .and_then(|id| board.threads.get(id))
            .filter(|t| !ENDED.contains(&t.status.as_str()) && !t.placeholder);
        match maker {
            Some(thread) => Some(thread.id.clone()),
            None => self.env.coordinator(scope),
        }
    }

    fn project_boards(&self) -> Vec<(String, LauncherBoard)> {
        self.env
            .scopes()
            .into_iter()
            .filter(|scope| scope != PORTFOLIO)
            .filter_map(|scope| self.env.board(&scope).map(|board| (scope, board)))
            .filter(|(_, board)| board.kind.as_deref() != Some("portfolio"))
            .collect()
    }

    /// Every live standalone launcher with its usage.
    pub fn usages(&self) -> Vec<LauncherUsage> {
        let now = self.clock.now_ms();
        let mut out = Vec::new();
        for (scope, board) in self.project_boards() {
            for rec in board.launchers.values() {
                let played = self.env.played_ms(&scope, &rec.name);
                if let Some(usage) = usage_of(&board, rec, now, played) {
                    out.push(LauncherUsage {
                        thread: self.owner_of(&scope, &board, rec),
                        scope: scope.clone(),
                        name: rec.name.clone(),
                        usage,
                    });
                }
            }
        }
        out
    }

    /// Launchers over their window right now.
    pub fn stale_launchers(&self) -> Vec<LauncherUsage> {
        self.usages()
            .into_iter()
            .filter(|u| u.usage.stale)
            .collect()
    }

    fn master_wakes(&self, now: i64) -> Vec<WakeEvent> {
        let mut events = Vec::new();
        for (scope, board) in self.project_boards() {
            let Some(lead) = self.env.coordinator(&scope) else {
                continue;
            };
            let waited = now - time_of(board.created_at.as_deref()) >= self.config.master_wait_ms;
            let due = now - self.last_of(REASON_MASTER, &scope, &lead, &scope)
                >= self.config.master_repeat_ms;
            if has_master(&board) || !waited || !due {
                continue;
            }
            self.remember(REASON_MASTER, &scope, &lead, now);
            self.env.note(StaleNote {
                level: "warn",
                text: format!("{scope} has no master launcher: its lead was told"),
                project: scope.clone(),
                thread: None,
            });
            events.push(WakeEvent {
                subject: scope.clone(),
                scope,
                thread: lead,
                reason: REASON_MASTER,
                text: master_text(),
            });
        }
        events
    }

    fn stale_wakes(&self, now: i64) -> Vec<WakeEvent> {
        let mut groups: BTreeMap<(String, String), (Vec<LauncherUsage>, Vec<LauncherUsage>)> =
            BTreeMap::new();
        for usage in self.usages() {
            let Some(thread) = usage.thread.clone() else {
                continue;
            };
            if !usage.usage.stale && !usage.usage.legacy {
                continue;
            }
            let entry = groups.entry((usage.scope.clone(), thread)).or_default();
            if usage.usage.stale {
                entry.0.push(usage);
            } else {
                entry.1.push(usage);
            }
        }
        let mut events = Vec::new();
        for ((scope, thread), (stale, legacy)) in groups {
            if now - self.last_of(REASON_STALE, &scope, &thread, &thread)
                < self.config.stale_repeat_ms
            {
                continue;
            }
            if let Some(event) = self.stale_event(&scope, &thread, stale, &legacy, now) {
                events.push(event);
            }
        }
        events
    }

    fn stale_event(
        &self,
        scope: &str,
        thread: &str,
        mut stale: Vec<LauncherUsage>,
        legacy: &[LauncherUsage],
        now: i64,
    ) -> Option<WakeEvent> {
        stale.sort_by(|a, b| {
            b.usage
                .unused_ms
                .cmp(&a.usage.unused_ms)
                .then_with(|| natural_cmp(&a.name, &b.name))
        });
        let mut legacy_names: Vec<String> = legacy.iter().map(|l| l.name.clone()).collect();
        legacy_names.sort();
        let items: Vec<StaleItem> = stale
            .iter()
            .map(|u| StaleItem {
                name: u.name.clone(),
                use_every: u.usage.use_every.clone(),
                legacy: u.usage.legacy,
                unused_ms: u.usage.unused_ms,
            })
            .collect();
        let text = stale_text(&items, &legacy_names);
        if text.is_empty() {
            return None;
        }
        self.remember(REASON_STALE, scope, thread, now);
        if !stale.is_empty() {
            let names: Vec<&str> = stale.iter().map(|s| s.name.as_str()).collect();
            self.env.note(StaleNote {
                level: "info",
                text: format!(
                    "{scope}/{thread}: {} launcher{} not used within their window ({})",
                    stale.len(),
                    if stale.len() == 1 { "" } else { "s" },
                    names.join(", ")
                ),
                project: scope.to_owned(),
                thread: Some(thread.to_owned()),
            });
        }
        Some(WakeEvent {
            scope: scope.to_owned(),
            thread: thread.to_owned(),
            reason: REASON_STALE,
            subject: thread.to_owned(),
            text,
        })
    }

    /// The wakes due now; returns how many were sent.
    pub fn tick(&self) -> usize {
        let now = self.clock.now_ms();
        let mut events = self.master_wakes(now);
        events.extend(self.stale_wakes(now));
        let count = events.len();
        if count > 0
            && let Err(message) = self.env.submit(events)
        {
            self.env.note(StaleNote {
                level: "warn",
                text: format!("launcher wakes not sent: {message}"),
                project: String::new(),
                thread: None,
            });
        }
        count
    }

    /// The launchers docs of an export (they carry `useEveryMs`, `standalone`, `keptUntil`
    /// from the board file): the newest Play run of the launcher is a use too. Changes the
    /// docs in place.
    pub fn overlay(&self, scope: &str, docs: &mut [Value]) {
        let now = self.clock.now_ms();
        for doc in docs.iter_mut().filter(|d| d["coll"] == "launchers") {
            let launcher = &mut doc["data"];
            if launcher["standalone"] != true || !launcher["retiredAt"].is_null() {
                continue;
            }
            let played = launcher["name"]
                .as_str()
                .map_or(0, |name| self.env.played_ms(scope, name));
            if played <= time_of(launcher["lastUsedAt"].as_str()) {
                continue;
            }
            let kept = time_of(launcher["keptUntil"].as_str()) > now;
            let window = launcher["useEveryMs"].as_i64().unwrap_or(0);
            launcher["lastUsedAt"] = Value::String(format_iso(played));
            launcher["stale"] = Value::Bool(!kept && now - played > window);
        }
    }

    /// Ticks every `tick_ms` until the task is aborted.
    pub fn start(self: &Arc<Self>) -> JoinHandle<()> {
        let service = Arc::clone(self);
        let every = Duration::from_millis(u64::try_from(self.config.tick_ms).unwrap_or(600_000));
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(every);
            ticker.tick().await;
            loop {
                ticker.tick().await;
                service.tick();
            }
        })
    }
}
