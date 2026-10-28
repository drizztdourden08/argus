//! What a removal request is, as it is kept in the key-value table and as the page and the CLI read it.

use super::text::one_line;
use serde::{Deserialize, Serialize};

const BRIEF_SENTENCE_MAX: usize = 200;
const BRIEF_SENTENCES: usize = 3;

/// What the thread asked for.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Named worktrees or leftover folders.
    #[default]
    Targets,
    /// Every worktree of the project that is merged, or stale and clean.
    Finished,
    /// Every leftover folder old enough.
    Leftovers,
}

/// Where a request stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequestStatus {
    #[default]
    Queued,
    Running,
    /// Nothing was refused.
    Done,
    /// Some targets were removed, some refused.
    Partial,
    /// Nothing was removed.
    Refused,
}

impl RequestStatus {
    /// Is the request still to run, or running?
    pub fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Partial => "partial",
            Self::Refused => "refused",
        }
    }
}

/// One worktree or leftover folder of a request: the id (or path) to remove, and how to name it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub label: String,
}

/// How one target ended, in a sentence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// The targets a `finished` or `leftovers` request left alone, and why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedSummary {
    pub count: usize,
    pub reasons: Vec<String>,
}

/// A removal request: only a record. Deciding and deleting stay in the removal itself.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RequestRec {
    pub id: String,
    pub key: String,
    pub by: String,
    pub at: String,
    pub mode: Mode,
    pub branch: bool,
    pub discard: bool,
    pub owner_entry: Option<String>,
    pub reason: Option<String>,
    pub older_than_days: Option<f64>,
    /// `finished` only: also every clean worktree with nothing ahead untouched for this many minutes.
    pub idle_minutes: Option<f64>,
    pub targets: Vec<Target>,
    pub resolved: bool,
    pub results: Vec<Outcome>,
    pub skipped: Option<SkippedSummary>,
    pub note: Option<String>,
    pub status: RequestStatus,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub notified: bool,
}

impl RequestRec {
    pub fn removed(&self) -> usize {
        self.results.iter().filter(|r| r.ok).count()
    }

    pub fn refused(&self) -> usize {
        self.results.len() - self.removed()
    }

    /// The record as the page's summary reads it.
    pub fn brief(&self) -> Brief {
        Brief {
            id: self.id.clone(),
            by: self.by.clone(),
            at: self.at.clone(),
            mode: self.mode,
            status: self.status,
            branch: self.branch,
            reason: self.reason.clone(),
            total: self.targets.len(),
            removed: self.removed(),
            refused: self.refused(),
            skipped: self.skipped.as_ref().map_or(0, |s| s.count),
            note: self.note.clone(),
            started_at: self.started_at.clone(),
            finished_at: self.finished_at.clone(),
            sentences: self
                .results
                .iter()
                .filter(|r| !r.ok)
                .take(BRIEF_SENTENCES)
                .map(|r| one_line(&r.text, BRIEF_SENTENCE_MAX))
                .collect(),
        }
    }
}

/// The brief form of a request: what `meta/worktrees` carries for the Worktrees tab.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Brief {
    pub id: String,
    pub by: String,
    pub at: String,
    pub mode: Mode,
    pub status: RequestStatus,
    pub branch: bool,
    pub reason: Option<String>,
    pub total: usize,
    pub removed: usize,
    pub refused: usize,
    pub skipped: usize,
    pub note: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub sentences: Vec<String>,
}
