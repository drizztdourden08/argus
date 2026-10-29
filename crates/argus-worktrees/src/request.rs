//! What a removal is asked with, and what it answers.

use crate::backup::Backup;
use crate::board::Caller;
use serde::Serialize;

/// A request to remove a worktree, prune a gone one, or delete a leftover folder.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct RemoveInput {
    pub project: String,
    /// A worktree id (ten hex characters) or its path.
    pub id: String,
    pub caller: Caller,
    /// Also delete the branch (only a merged one goes without the owner's say).
    pub branch: bool,
    /// Discard uncommitted work, or an unmerged branch: needs the owner's say and a backup first.
    pub discard: bool,
    /// The board entry of the owner that asks for the discard.
    pub owner_entry: Option<String>,
    /// Why, for the activity line.
    pub reason: Option<String>,
    /// The HEAD the caller saw; a different one is a conflict.
    pub head: Option<String>,
    /// The dirty count the caller saw; a different one is a conflict.
    pub dirty: Option<u32>,
    /// A leftover: the newest change (ms) the caller saw.
    pub newest_at_ms: Option<i64>,
    /// A leftover: the page confirms a folder younger than the age limit.
    pub confirm_young: bool,
    /// A leftover: delete only folders untouched for this many days.
    pub older_than_days: Option<f64>,
    pub(crate) prune: bool,
    pub(crate) job: Option<String>,
}

impl RemoveInput {
    pub fn new(project: &str, id: &str, caller: Caller) -> Self {
        Self {
            project: project.to_string(),
            id: id.to_string(),
            caller,
            branch: false,
            discard: false,
            owner_entry: None,
            reason: None,
            head: None,
            dirty: None,
            newest_at_ms: None,
            confirm_young: false,
            older_than_days: None,
            prune: false,
            job: None,
        }
    }
}

/// What was discarded: the uncommitted changes and the commits no ref keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Discarded {
    pub dirty: u32,
    pub unmerged: Option<u32>,
}

/// The answer of a removal, a prune, or the start of a leftover deletion.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveOutcome {
    pub removed: bool,
    pub pruned_only: bool,
    pub started: bool,
    pub leftover: bool,
    pub finished_by_argus: bool,
    pub folder_left: bool,
    pub job_id: Option<String>,
    pub folder: Option<String>,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub deleted_branch: bool,
    pub branch_note: Option<String>,
    pub branch_why: Option<String>,
    pub backup: Option<Backup>,
    pub discarded: Option<Discarded>,
    pub git_error: Option<String>,
    /// An ignored, untracked `node_modules` was deleted by Argus before git ran. Absent from the JSON when false.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub node_modules_cleared: bool,
    #[serde(rename = "sizeMB")]
    pub size_mb: Option<f64>,
    pub files: Option<u64>,
    pub newest_at: Option<String>,
    pub pruned: Vec<String>,
    pub text: String,
}

impl RemoveOutcome {
    /// The sentence of what was done, as the activity line and the CLI say it.
    pub fn sentence(&self) -> String {
        let path = self.path.as_deref().unwrap_or_default();
        let verb = if self.pruned_only {
            "pruned"
        } else {
            "removed"
        };
        let finished = if self.finished_by_argus {
            " (folder finished by Argus: git could not delete it)"
        } else {
            ""
        };
        let mut parts = vec![format!("{verb} worktree {path}{finished}")];
        if self.finished_by_argus {
            parts.push(format!(
                "job {} deletes the files in the background",
                self.job_id.as_deref().unwrap_or("?")
            ));
        }
        if self.folder_left {
            parts.push("git left the folder behind: it is a leftover now, which coord worktree remove --all-leftovers takes without the age limit".to_string());
        }
        if let Some(branch) = &self.branch {
            parts.push(self.branch_part(branch));
        }
        if let Some(backup) = &self.backup {
            let files: Vec<&str> = [backup.bundle.as_deref(), backup.patch.as_deref()]
                .into_iter()
                .flatten()
                .collect();
            parts.push(format!(
                "backup {} (restore: {})",
                files.join(" and "),
                backup.restore.join("; ")
            ));
        }
        if self.node_modules_cleared {
            parts.push("node_modules cleared by Argus first".to_string());
        }
        parts.join(", ")
    }

    fn branch_part(&self, branch: &str) -> String {
        if self.deleted_branch {
            return match &self.branch_why {
                Some(why) => format!("branch {branch} deleted: {why}"),
                None => format!("branch {branch} deleted"),
            };
        }
        match &self.branch_note {
            Some(note) => format!("branch {branch} kept ({note})"),
            None => format!("branch {branch} kept"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sentence_says_what_happened_to_the_branch_and_the_backup() {
        let mut out = RemoveOutcome {
            removed: true,
            path: Some("X:/a/w".into()),
            branch: Some("w".into()),
            ..RemoveOutcome::default()
        };
        assert_eq!(out.sentence(), "removed worktree X:/a/w, branch w kept");
        out.branch_note = Some("not merged".into());
        assert_eq!(
            out.sentence(),
            "removed worktree X:/a/w, branch w kept (not merged)"
        );
        out.deleted_branch = true;
        out.branch_why = Some("merged into local main (not pushed)".into());
        assert_eq!(
            out.sentence(),
            "removed worktree X:/a/w, branch w deleted: merged into local main (not pushed)"
        );
        out.backup = Some(Backup {
            bundle: Some("b.bundle".into()),
            patch: Some("b.patch".into()),
            restore: vec!["one".into(), "two".into()],
            ..Backup::default()
        });
        assert!(
            out.sentence()
                .ends_with(", backup b.bundle and b.patch (restore: one; two)")
        );
        let pruned = RemoveOutcome {
            pruned_only: true,
            path: Some("X:/a/g".into()),
            ..RemoveOutcome::default()
        };
        assert_eq!(pruned.sentence(), "pruned worktree X:/a/g");
    }
}
