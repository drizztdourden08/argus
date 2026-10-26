//! The records of the run subsystem: a run row, the patch a status change applies, a log
//! line and the limits that shape them. All of them serialise in the camelCase shape the
//! pages and the API use.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::def::RunKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Queued,
    Running,
    Exited,
    Failed,
    Stopped,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Exited => "exited",
            Self::Failed => "failed",
            Self::Stopped => "stopped",
        }
    }

    /// True while the run still occupies a slot or waits for one.
    pub fn is_live(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}

/// Who asked for a run: a board page, an MCP tool or the `coord` CLI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    Page,
    Mcp,
    #[default]
    Cli,
}

impl Origin {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Mcp => "mcp",
            Self::Cli => "cli",
        }
    }
}

/// The public shape of a run (`RunRow` of the contract).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRow {
    pub id: String,
    pub scope: String,
    pub name: String,
    pub kind: RunKind,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
    pub pid: Option<u32>,
    pub visible: bool,
    pub stdin: bool,
    pub origin: Origin,
    pub by: String,
    pub agent: Option<String>,
    pub reason: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub log_bytes: u64,
    pub log_dropped: u64,
    pub log_seq: u64,
    pub orphan: bool,
    pub error: Option<String>,
    pub last_line: Option<String>,
    pub cmd: String,
    pub cwd: String,
}

impl RunRow {
    /// What the page sees: the row without its command line, working directory, scope and
    /// log cursor.
    pub fn to_doc(&self) -> Value {
        let mut doc = serde_json::to_value(self).unwrap_or(Value::Null);
        if let Some(map) = doc.as_object_mut() {
            for hidden in ["scope", "cmd", "cwd", "logSeq"] {
                map.remove(hidden);
            }
        }
        doc
    }
}

/// The values a new run starts with; the store fills in the rest (`queued`, zero counters).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewRun {
    pub scope: String,
    pub rid: String,
    pub name: String,
    pub kind: RunKind,
    pub cmd: String,
    pub cwd: String,
    pub visible: bool,
    pub origin: Origin,
    pub by: String,
    pub agent: Option<String>,
    pub reason: Option<String>,
    pub queued_at: String,
}

/// A partial update of a run row. `Some(None)` clears a nullable field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunPatch {
    pub status: Option<RunStatus>,
    pub exit_code: Option<Option<i32>>,
    pub signal: Option<Option<String>>,
    pub pid: Option<Option<u32>>,
    pub stdin: Option<bool>,
    pub by: Option<String>,
    pub started_at: Option<Option<String>>,
    pub ended_at: Option<Option<String>>,
    pub orphan: Option<bool>,
    pub error: Option<Option<String>>,
}

impl RunPatch {
    pub fn apply_to(&self, row: &mut RunRow) {
        if let Some(v) = self.status {
            row.status = v;
        }
        if let Some(v) = self.exit_code {
            row.exit_code = v;
        }
        if let Some(v) = &self.signal {
            row.signal.clone_from(v);
        }
        if let Some(v) = self.pid {
            row.pid = v;
        }
        if let Some(v) = self.stdin {
            row.stdin = v;
        }
        if let Some(v) = &self.by {
            row.by.clone_from(v);
        }
        self.apply_times_and_flags(row);
    }

    fn apply_times_and_flags(&self, row: &mut RunRow) {
        if let Some(v) = &self.started_at {
            row.started_at.clone_from(v);
        }
        if let Some(v) = &self.ended_at {
            row.ended_at.clone_from(v);
        }
        if let Some(v) = self.orphan {
            row.orphan = v;
        }
        if let Some(v) = &self.error {
            row.error.clone_from(v);
        }
    }
}

/// Which runs a listing wants. No statuses means every status.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListFilter {
    pub statuses: Vec<String>,
    pub name: Option<String>,
    pub limit: Option<i64>,
}

impl ListFilter {
    /// `"running, queued"` as a status filter.
    #[must_use]
    pub fn with_statuses(mut self, csv: &str) -> Self {
        self.statuses = csv
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect();
        self
    }

    /// The row limit: 1 to 500, 50 when unset or invalid.
    pub fn row_limit(&self) -> usize {
        let wanted = self.limit.filter(|n| *n > 0).unwrap_or(50);
        usize::try_from(wanted.clamp(1, 500)).unwrap_or(50)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stream {
    Out,
    Err,
    Sys,
}

impl Stream {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Out => "out",
            Self::Err => "err",
            Self::Sys => "sys",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogLine {
    pub seq: u64,
    pub stream: Stream,
    pub at: String,
    pub text: String,
}

/// A stored line and whether it belongs to the head that is never dropped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredLine {
    pub line: LogLine,
    pub head: bool,
}

/// One batch of the log writer, applied by the store in one transaction: the new lines,
/// the oldest non-head lines to delete, and the run counters afterwards.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogCommit {
    pub lines: Vec<StoredLine>,
    pub drop_up_to: Option<u64>,
    pub log_bytes: u64,
    pub log_dropped: u64,
    pub log_seq: u64,
    pub last_line: Option<String>,
}

/// What a flushed line looks like on the live tail.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEvent {
    pub scope: String,
    pub rid: String,
    pub seq: u64,
    pub stream: Stream,
    pub at: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadMode {
    Tail(usize),
    After { after: u64, limit: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogRead {
    pub stream: Option<Stream>,
    pub mode: ReadMode,
}

/// The limits of the run service (`config.runs`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunLimits {
    pub max_concurrent: usize,
    pub log_cap_bytes: u64,
    pub log_head_bytes: u64,
    pub keep_per_def: usize,
    pub keep_days: i64,
    pub stop_grace_ms: u64,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            log_cap_bytes: 2_097_152,
            log_head_bytes: 262_144,
            keep_per_def: 30,
            keep_days: 14,
            stop_grace_ms: 5000,
        }
    }
}
