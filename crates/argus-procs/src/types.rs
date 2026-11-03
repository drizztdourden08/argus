//! The data the monitor reads (a sample) and writes (the listing and its meta).

use crate::rule::Rule;
use serde::Serialize;

/// One process as the sampler saw it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcRow {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub cmd: String,
    pub path: Option<String>,
    pub mem_bytes: u64,
    pub start_ms: i64,
    pub cpu_ms: i64,
    pub responding: Option<bool>,
    /// The folder it runs in. The real sampler leaves it out (reading it costs a look into the
    /// process); the kill and the reaper ask [`crate::Sampler::cwds`] for the few they judge.
    pub cwd: Option<String>,
}

/// One look at every process of the machine.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub at: i64,
    pub platform: String,
    pub sampler_pid: Option<u32>,
    pub procs: Vec<ProcRow>,
}

impl Sample {
    /// Whether the sample is from Windows, where a parent must be older than its child.
    #[must_use]
    pub fn is_windows(&self) -> bool {
        self.platform == "win32"
    }
}

/// Why a process is listed as a problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Flag {
    Hung,
    Hog,
    Orphan,
    Stale,
    Old,
    Group,
}

impl Flag {
    /// The flags that make a status, in the order the first that applies wins.
    pub const KINDS: [Flag; 5] = [Flag::Hung, Flag::Hog, Flag::Orphan, Flag::Stale, Flag::Old];

    /// The flag as the dashboard names it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Hung => "hung",
            Self::Hog => "hog",
            Self::Orphan => "orphan",
            Self::Stale => "stale",
            Self::Old => "old",
            Self::Group => "group",
        }
    }
}

/// The state of a listed process: its first flag, else `ok`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Hung,
    Hog,
    Orphan,
    Stale,
    Old,
    Ok,
}

impl Status {
    /// The first of hung, hog, orphan, stale, old among the flags, else `Ok`.
    #[must_use]
    pub fn from_flags(flags: &[Flag]) -> Self {
        match Flag::KINDS.iter().find(|kind| flags.contains(kind)) {
            Some(Flag::Hung) => Self::Hung,
            Some(Flag::Hog) => Self::Hog,
            Some(Flag::Orphan) => Self::Orphan,
            Some(Flag::Stale) => Self::Stale,
            Some(Flag::Old) => Self::Old,
            _ => Self::Ok,
        }
    }
}

/// Why a process is never killed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Guard {
    System,
    Windows,
    Claude,
    Dashboard,
    Tray,
}

impl Guard {
    /// The reason as the dashboard names it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Windows => "windows",
            Self::Claude => "claude",
            Self::Dashboard => "dashboard",
            Self::Tray => "tray",
        }
    }
}

/// One listed process: the doc stored in collection `procs` of scope `_portfolio`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcDoc {
    pub id: String,
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub cmd: String,
    pub hint: Option<String>,
    pub rule: String,
    pub project: Option<String>,
    #[serde(rename = "memMB")]
    pub mem_mb: i64,
    pub cpu_pct: Option<f64>,
    pub started_at: Option<String>,
    pub age_min: i64,
    pub idle_min: i64,
    pub responding: Option<bool>,
    pub parent_alive: bool,
    pub status: Status,
    pub flags: Vec<Flag>,
    pub protected: bool,
    pub protected_why: Option<Guard>,
    pub killable: bool,
    pub since: String,
}

/// What one rule took in the last sample.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuleSummary {
    pub rule: String,
    pub count: usize,
    pub matched: usize,
    #[serde(rename = "memMB")]
    pub mem_mb: i64,
    #[serde(rename = "maxGroupMemMB")]
    pub max_group_mem_mb: Option<f64>,
    pub over: bool,
}

/// The summary of the last sample: doc `meta/procs`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub id: String,
    pub sampled_at: Option<String>,
    pub sample_ms: u64,
    pub count: usize,
    #[serde(rename = "totalMemMB")]
    pub total_mem_mb: i64,
    pub by_rule: Vec<RuleSummary>,
    pub rules: Vec<Rule>,
    pub rules_file: String,
    pub rules_error: Option<String>,
    pub platform: String,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub took_ms: Option<u64>,
}
