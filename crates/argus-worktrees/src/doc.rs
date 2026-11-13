//! The records the server mirrors for the board's Worktrees tab: one doc per worktree, one summary per project.

use crate::board::RequestDoc;
use crate::class::{Class, LockKind};
use crate::hygiene::{HygieneDoc, HygieneMeta};
use crate::jobs::JobDoc;
use crate::linking::LinkSource;
use crate::requests::Brief;
use serde::Serialize;
use std::collections::BTreeMap;

/// Where a worktree stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Active,
    Stale,
    Merged,
    DirtyStale,
    Missing,
    NotGit,
    Leftover,
    Deleting,
    Error,
    Kept,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Stale => "stale",
            Self::Merged => "merged",
            Self::DirtyStale => "dirty-stale",
            Self::Missing => "missing",
            Self::NotGit => "not-git",
            Self::Leftover => "leftover",
            Self::Deleting => "deleting",
            Self::Error => "error",
            Self::Kept => "kept",
        }
    }

    /// Does a worktree in this status need a decision from its thread?
    pub fn needs_decision(self) -> bool {
        matches!(self, Self::Stale | Self::Merged | Self::DirtyStale)
    }
}

/// An unexpired keep on a row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KeptDoc {
    pub until: String,
    pub note: Option<String>,
    pub by: Option<String>,
}

/// The leftover part of a row.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeftoverDoc {
    #[serde(rename = "sizeMB")]
    pub size_mb: Option<f64>,
    pub files: Option<u64>,
    pub newest_at: Option<String>,
    pub age_days: Option<i64>,
    pub has_nested_git: bool,
    pub nested_git: Option<String>,
    pub node_modules: bool,
    pub is_link: bool,
    pub partial: bool,
    pub pending: bool,
    pub top_entries: Vec<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub made: bool,
}

/// The item a worktree belongs to, and how Argus knows it.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemLink {
    /// The item it belongs to (`I-n`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    /// How Argus knows the item: a link, the owning agent, the branch name or an item's text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_source: Option<LinkSource>,
    /// No item an hour after Argus first saw it (the `no item` chip).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unlinked: bool,
    /// The agent record (`A-n`) that owns it: a running one, else the last to end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

/// The doc of one worktree.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeDoc {
    pub id: String,
    pub path: String,
    pub name: String,
    pub rule: String,
    pub thread: Option<String>,
    pub branch: Option<String>,
    pub detached: bool,
    pub head: Option<String>,
    pub subject: Option<String>,
    pub last_commit_at: Option<String>,
    pub last_touched_at: Option<String>,
    pub age_days: Option<i64>,
    pub stale_days: u32,
    pub dirty: u32,
    /// How many of the `dirty` paths are untracked files.
    pub untracked: u32,
    pub ahead: u32,
    pub behind: u32,
    pub upstream: Option<String>,
    pub merged: bool,
    pub merged_into: Option<String>,
    pub referenced: Option<bool>,
    /// What the folder is and what removing it would cost. Display only: no removal reads it.
    pub class: Class,
    /// Whose lock holds it, for a locked worktree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_kind: Option<LockKind>,
    pub unmerged: Option<u32>,
    pub unmerged_subjects: Vec<String>,
    pub prunable: Option<String>,
    pub locked: Option<String>,
    pub main: bool,
    pub repo: Option<String>,
    pub status: Status,
    pub kept: Option<KeptDoc>,
    pub request: Option<RequestDoc>,
    pub leftover: Option<LeftoverDoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job: Option<JobDoc>,
    /// Where the worktree stands: in use, released (and when it goes), closing, review or kept.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hygiene: Option<HygieneDoc>,
    #[serde(flatten)]
    pub link: ItemLink,
    pub error: Option<String>,
}

/// One rule as the summary shows it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSummary {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub invalid: Option<String>,
    pub thread: Option<String>,
    pub note: Option<String>,
    pub stale_days: Option<u32>,
    pub added_by: Option<String>,
    pub added_at: Option<String>,
    pub exists: Option<bool>,
    pub matched: Option<u32>,
    pub candidates: Option<u32>,
    pub error: Option<String>,
}

/// The leftovers a thread may delete: older than the age limit, no nested repository, no link, measured.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeftoverOld {
    pub days: f64,
    pub count: usize,
    #[serde(rename = "sizeMB")]
    pub size_mb: f64,
}

/// The project's summary: the doc `meta/worktrees`.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub id: String,
    pub name: String,
    pub scanned_at: Option<String>,
    pub scan_ms: i64,
    pub live_ms: i64,
    pub took_ms: Option<i64>,
    pub count: usize,
    /// The worktrees listed that are not main checkouts or being deleted: the left side of the counter.
    pub listed: usize,
    /// The running agent records of the board (working, waiting, stop-requested): the right side of the counter.
    pub running_agents: usize,
    pub git: usize,
    pub leftovers: usize,
    #[serde(rename = "leftoverMB")]
    pub leftover_mb: f64,
    pub leftover_pending: usize,
    pub leftover_old: LeftoverOld,
    pub deleting: usize,
    pub jobs: Vec<JobDoc>,
    /// What threads asked Argus to remove, newest first (the Worktrees tab lists them read-only).
    pub requests: Vec<Brief>,
    pub gone: usize,
    pub many_at: usize,
    pub too_many: bool,
    pub by_status: BTreeMap<String, usize>,
    pub stale_days: u32,
    pub rules: Vec<RuleSummary>,
    /// The hygiene: on, the debounce, the report delay, the last removals.
    pub hygiene: HygieneMeta,
    pub error: Option<String>,
}

/// What a project shows: its rows and its summary.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Derived {
    pub docs: Vec<WorktreeDoc>,
    pub meta: Meta,
}
