//! The `agents` keys of `config.json` and the options of each service.

use serde_json::Value;
use std::path::PathBuf;

/// One minute in milliseconds.
pub const MINUTE_MS: i64 = 60_000;

/// What `config.json` says under `agents`; a missing or non-positive number is `None`.
#[derive(Debug, Clone, Default)]
pub struct AgentsConfig {
    /// `agents.stallMs`: quiet time that counts as stalled.
    pub stall_ms: Option<i64>,
    /// `agents.reapMs`: quiet time before the reaper closes a record.
    pub reap_ms: Option<i64>,
    /// `agents.reapOpusMs`: quiet time before the reaper closes an opus-level record.
    pub reap_opus_ms: Option<i64>,
    /// `agents.reconcileMs`: time between reconciler polls.
    pub reconcile_ms: Option<i64>,
    /// `agents.orphanWakeMs`: age of a record Claude has nothing for before the thread is told.
    pub orphan_wake_ms: Option<i64>,
    /// `agents.orphanClearMs`: time after the wake before the record is closed.
    pub orphan_clear_ms: Option<i64>,
    /// `agents.idleWakeMs`: time an ended turn waits before the thread is told.
    pub idle_wake_ms: Option<i64>,
    /// `agents.claudeHome`: where Claude keeps its projects.
    pub claude_home: Option<PathBuf>,
    /// `agents.claudeTemp`: where Claude keeps its task outputs.
    pub claude_temp: Option<PathBuf>,
}

fn positive(agents: &Value, key: &str) -> Option<i64> {
    agents.get(key)?.as_i64().filter(|n| *n > 0)
}

fn path(agents: &Value, key: &str) -> Option<PathBuf> {
    let text = agents.get(key)?.as_str()?;
    (!text.is_empty()).then(|| PathBuf::from(text))
}

impl AgentsConfig {
    /// Reads the `agents` object of the config.
    #[must_use]
    pub fn from_json(agents: &Value) -> Self {
        Self {
            stall_ms: positive(agents, "stallMs"),
            reap_ms: positive(agents, "reapMs"),
            reap_opus_ms: positive(agents, "reapOpusMs"),
            reconcile_ms: positive(agents, "reconcileMs"),
            orphan_wake_ms: positive(agents, "orphanWakeMs"),
            orphan_clear_ms: positive(agents, "orphanClearMs"),
            idle_wake_ms: positive(agents, "idleWakeMs"),
            claude_home: path(agents, "claudeHome"),
            claude_temp: path(agents, "claudeTemp"),
        }
    }
}

/// Options of the transcript watcher.
#[derive(Debug, Clone)]
pub struct WatcherOptions {
    /// Time between polls.
    pub poll_ms: i64,
    /// The least time between two updates of one agent's doc.
    pub emit_ms: i64,
    /// Quiet time that counts as stalled.
    pub stall_ms: i64,
    /// How long after its end a finished agent is still read once.
    pub recent_ms: i64,
    /// How long a transcript may be missing before the note is set.
    pub missing_ms: i64,
    /// Longest last message kept.
    pub text_max: usize,
    /// Bytes read at a time.
    pub chunk_bytes: usize,
    /// Most bytes read from one transcript per poll.
    pub max_bytes_per_tick: usize,
    /// Longest line parsed; a longer one is a tool dump and is dropped.
    pub max_line_bytes: usize,
}

impl Default for WatcherOptions {
    fn default() -> Self {
        Self {
            poll_ms: 1000,
            emit_ms: 1000,
            stall_ms: 180_000,
            recent_ms: 10 * MINUTE_MS,
            missing_ms: 20_000,
            text_max: 500,
            chunk_bytes: 256 * 1024,
            max_bytes_per_tick: 4 * 1024 * 1024,
            max_line_bytes: 4 * 1024 * 1024,
        }
    }
}

impl WatcherOptions {
    /// The options with the config's stall time.
    #[must_use]
    pub fn configured(mut self, config: &AgentsConfig) -> Self {
        self.stall_ms = config.stall_ms.unwrap_or(self.stall_ms);
        self
    }
}

/// Options of the reaper.
#[derive(Debug, Clone)]
pub struct ReaperOptions {
    /// Time between polls.
    pub poll_ms: i64,
    /// Quiet time before a record is closed.
    pub reap_ms: i64,
    /// Quiet time before an opus-level record (anything but haiku and sonnet) is closed; at least `reap_ms`.
    pub reap_opus_ms: i64,
    /// A transcript or task that grew within this time is alive.
    pub stall_ms: i64,
    /// Where Claude keeps its projects.
    pub claude_home: PathBuf,
    /// Where Claude keeps its task outputs.
    pub claude_temp: PathBuf,
}

impl Default for ReaperOptions {
    fn default() -> Self {
        let evidence = ReconcilerOptions::default();
        Self {
            poll_ms: MINUTE_MS,
            reap_ms: 20 * MINUTE_MS,
            reap_opus_ms: 60 * MINUTE_MS,
            stall_ms: evidence.stall_ms,
            claude_home: evidence.claude_home,
            claude_temp: evidence.claude_temp,
        }
    }
}

impl ReaperOptions {
    /// The options with what the config sets.
    #[must_use]
    pub fn configured(mut self, config: &AgentsConfig) -> Self {
        self.reap_ms = config.reap_ms.unwrap_or(self.reap_ms);
        self.reap_opus_ms = config.reap_opus_ms.unwrap_or(self.reap_opus_ms);
        self.stall_ms = config.stall_ms.unwrap_or(self.stall_ms);
        if let Some(home) = &config.claude_home {
            self.claude_home.clone_from(home);
        }
        if let Some(temp) = &config.claude_temp {
            self.claude_temp.clone_from(temp);
        }
        self
    }
}

/// Options of the reconciler.
#[derive(Debug, Clone)]
pub struct ReconcilerOptions {
    /// Time between polls.
    pub reconcile_ms: i64,
    /// Age of an orphan record before its thread is told.
    pub orphan_wake_ms: i64,
    /// Time after the wake before a record is closed.
    pub orphan_clear_ms: i64,
    /// Time an ended turn waits before its thread is told.
    pub idle_wake_ms: i64,
    /// A transcript that grew within this time is alive.
    pub stall_ms: i64,
    /// A quiet transcript this old with an end turn as its last line counts as ended.
    pub settle_ms: i64,
    /// How far a transcript may be ahead of its meta's `finished` mark.
    pub meta_slack_ms: i64,
    /// A transcript may be this much older than the record it is linked to.
    pub link_before_ms: i64,
    /// A transcript may be this much newer than the record it is linked to.
    pub link_after_ms: i64,
    /// A record closed longer ago than this is never reopened.
    pub reopen_ms: i64,
    /// The growth after a close that proves an agent was resumed.
    pub reopen_gap_ms: i64,
    /// Most findings in a report.
    pub max_findings: usize,
    /// Where Claude keeps its projects.
    pub claude_home: PathBuf,
    /// Where Claude keeps its task outputs.
    pub claude_temp: PathBuf,
}

fn user_home() -> PathBuf {
    let var = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"));
    var.map_or_else(|| PathBuf::from("."), PathBuf::from)
}

impl Default for ReconcilerOptions {
    fn default() -> Self {
        Self {
            reconcile_ms: 2 * MINUTE_MS,
            orphan_wake_ms: 15 * MINUTE_MS,
            orphan_clear_ms: 30 * MINUTE_MS,
            idle_wake_ms: 5 * MINUTE_MS,
            stall_ms: 180_000,
            settle_ms: 30_000,
            meta_slack_ms: 5000,
            link_before_ms: 60 * MINUTE_MS,
            link_after_ms: 10 * MINUTE_MS,
            reopen_ms: 24 * 60 * MINUTE_MS,
            reopen_gap_ms: MINUTE_MS,
            max_findings: 200,
            claude_home: user_home().join(".claude"),
            claude_temp: std::env::temp_dir().join("claude"),
        }
    }
}

impl ReconcilerOptions {
    /// The options with what the config sets.
    #[must_use]
    pub fn configured(mut self, config: &AgentsConfig) -> Self {
        self.reconcile_ms = config.reconcile_ms.unwrap_or(self.reconcile_ms);
        self.orphan_wake_ms = config.orphan_wake_ms.unwrap_or(self.orphan_wake_ms);
        self.orphan_clear_ms = config.orphan_clear_ms.unwrap_or(self.orphan_clear_ms);
        self.idle_wake_ms = config.idle_wake_ms.unwrap_or(self.idle_wake_ms);
        self.stall_ms = config.stall_ms.unwrap_or(self.stall_ms);
        if let Some(home) = &config.claude_home {
            self.claude_home.clone_from(home);
        }
        if let Some(temp) = &config.claude_temp {
            self.claude_temp.clone_from(temp);
        }
        self
    }
}
