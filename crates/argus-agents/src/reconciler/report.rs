//! What a reconciler run reports: counts per project and the findings behind them.

use serde::Serialize;
use std::collections::BTreeMap;

/// What the reconciler decided about one record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Class {
    /// The transcript grew lately. Counted, never a finding.
    Alive,
    /// The record was matched to a transcript.
    Linked,
    /// The record was a second record of one Claude agent and was closed.
    Duplicate,
    /// The transcript exists but stopped growing and its turn did not end.
    Quiet,
    /// The turn ended and nothing grew since.
    Idle,
    /// Claude has nothing for the record.
    Orphan,
    /// Claude has nothing for the record yet, but the record is recent.
    Pending,
    /// Claude keeps nothing to compare the record with.
    Unknown,
    /// The record was closed.
    Closed,
    /// A closed record whose agent grew again was reopened.
    Reopened,
}

/// How many records fell in each class, for one project.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Counts {
    /// Transcript grew within the stall window.
    pub alive: u32,
    /// Transcript quiet, turn not ended.
    pub quiet: u32,
    /// Turn ended.
    pub idle: u32,
    /// Claude has nothing for the record.
    pub orphan: u32,
    /// Claude has nothing yet, the record is recent.
    pub pending: u32,
    /// No session to compare with.
    pub unknown: u32,
    /// Records closed.
    pub closed: u32,
    /// Records reopened.
    pub reopened: u32,
    /// Records linked to a transcript.
    pub linked: u32,
    /// Duplicate records closed.
    pub duplicates: u32,
    /// Wakes sent.
    pub woken: u32,
}

impl Counts {
    /// Whether the run changed or told anything.
    #[must_use]
    pub fn acted(&self) -> bool {
        self.closed + self.reopened + self.linked + self.duplicates + self.woken > 0
    }
}

/// One record and what is the matter with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    /// The board key.
    pub project: String,
    /// The record id.
    pub id: String,
    /// The record name.
    pub name: Option<String>,
    /// The owning thread.
    pub thread: Option<String>,
    /// The class.
    pub class: Class,
    /// The Claude agent id.
    pub claude_id: Option<String>,
    /// One sentence on why.
    pub detail: String,
}

/// The last run, as `GET /api/agents/reconcile` reports it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RunReport {
    /// When the run started.
    pub at: String,
    /// How long it took.
    pub ms: i64,
    /// Whether the run only looked because the machine slept.
    pub skipped: bool,
    /// Counts by board key.
    pub projects: BTreeMap<String, Counts>,
    /// The findings, at most `max_findings`.
    pub findings: Vec<Finding>,
}
