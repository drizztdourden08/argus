//! Deletion jobs: a leftover folder, or the rest of a worktree git could not delete, removed in the background.

use crate::doc::WorktreeDoc;
use crate::paths::base_name;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

/// Finished jobs kept for the page and `coord worktree leftovers`.
pub const JOBS_KEPT: usize = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Queued,
    Checking,
    Running,
    Done,
    Failed,
    Refused,
}

impl JobState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Checking => "checking",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
            Self::Refused => "refused",
        }
    }

    /// Is the job still to run or running?
    pub fn active(self) -> bool {
        matches!(self, Self::Queued | Self::Checking | Self::Running)
    }
}

/// What a job deletes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    Leftover,
    Worktree,
}

/// A job as the page shows it.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDoc {
    pub id: String,
    pub row_id: String,
    pub path: String,
    pub name: String,
    pub folder: String,
    pub state: JobState,
    pub removed: u64,
    pub total: Option<u64>,
    #[serde(rename = "sizeMB")]
    pub size_mb: Option<f64>,
    pub files: Option<u64>,
    pub queued_at: String,
    pub finished_at: Option<String>,
    pub error: Option<String>,
    pub by: String,
}

impl JobDoc {
    /// A job that was just queued for this folder.
    pub fn queued(id: &str, row_id: &str, path: &str, by: &str, queued_at: String) -> Self {
        Self {
            id: id.to_string(),
            row_id: row_id.to_string(),
            path: path.to_string(),
            name: base_name(path),
            folder: path.to_string(),
            state: JobState::Queued,
            removed: 0,
            total: None,
            size_mb: None,
            files: None,
            queued_at,
            finished_at: None,
            error: None,
            by: by.to_string(),
        }
    }
}

/// A deletion job, as the service keeps it: what the page shows, and what only the service needs.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct Job {
    pub doc: JobDoc,
    pub project: String,
    pub row: Option<WorktreeDoc>,
    pub batch: bool,
    pub kind: JobKind,
    pub git_error: Option<String>,
}

impl Deref for Job {
    type Target = JobDoc;

    fn deref(&self) -> &JobDoc {
        &self.doc
    }
}

impl DerefMut for Job {
    fn deref_mut(&mut self) -> &mut JobDoc {
        &mut self.doc
    }
}

impl Job {
    pub fn doc(&self) -> JobDoc {
        self.doc.clone()
    }
}

/// Drop the oldest finished jobs beyond [`JOBS_KEPT`].
pub fn trim_jobs(jobs: &mut Vec<Job>) {
    let mut finished: Vec<(String, String)> = jobs
        .iter()
        .filter(|j| !j.state.active())
        .map(|j| (j.finished_at.clone().unwrap_or_default(), j.id.clone()))
        .collect();
    finished.sort();
    finished.reverse();
    let drop: Vec<String> = finished
        .into_iter()
        .skip(JOBS_KEPT)
        .map(|(_, id)| id)
        .collect();
    jobs.retain(|j| !drop.contains(&j.id));
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn job(id: usize, state: JobState, finished: &str) -> Job {
        let path = format!("X:/a/j{id}");
        let mut doc = JobDoc::queued(
            &format!("L-{id}"),
            &format!("r{id}"),
            &path,
            "t",
            String::new(),
        );
        (doc.state, doc.finished_at) = (state, Some(finished.to_string()));
        Job {
            doc,
            project: "alpha".into(),
            row: None,
            batch: false,
            kind: JobKind::Leftover,
            git_error: None,
        }
    }

    #[test]
    fn only_the_newest_finished_jobs_stay_and_active_ones_never_go() {
        let mut jobs: Vec<Job> = (0..40)
            .map(|i| job(i, JobState::Done, &format!("2026-10-{:02}", i + 1)))
            .collect();
        jobs.push(job(99, JobState::Running, ""));
        trim_jobs(&mut jobs);
        assert_eq!(jobs.len(), JOBS_KEPT + 1);
        assert!(jobs.iter().any(|j| j.id == "L-99"));
        assert!(jobs.iter().any(|j| j.id == "L-39"));
        assert!(!jobs.iter().any(|j| j.id == "L-0"));
    }
}
