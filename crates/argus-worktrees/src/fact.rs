//! What a scan found about one folder.

use crate::measure::Leftover;
use crate::parse::ListedWorktree;
use crate::paths::worktree_id;
use crate::rule::Rule;

/// The facts of one candidate folder, as the last scan read them.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, PartialEq)]
pub struct Fact {
    pub path: String,
    pub id: String,
    pub rule: String,
    pub thread: Option<String>,
    pub exists: bool,
    pub is_git: bool,
    pub main: bool,
    pub branch: Option<String>,
    pub detached: bool,
    pub head: Option<String>,
    pub subject: Option<String>,
    pub commit_ms: Option<i64>,
    pub index_ms: Option<i64>,
    pub dirty: u32,
    pub ahead: u32,
    pub behind: u32,
    pub upstream: Option<String>,
    pub merged: bool,
    pub merged_into: Option<String>,
    pub referenced: Option<bool>,
    pub default_names: Vec<String>,
    pub unmerged: Option<u32>,
    pub unmerged_subjects: Vec<String>,
    pub prunable: Option<String>,
    pub locked: Option<String>,
    pub repo: Option<String>,
    pub registered: bool,
    pub leftover: Option<Leftover>,
    pub error: Option<String>,
}

/// The facts of a folder nothing was read from yet (not a git work tree).
pub fn blank_fact(path: &str, rule: &Rule, case_insensitive: bool) -> Fact {
    Fact {
        path: path.to_string(),
        id: worktree_id(path, case_insensitive),
        rule: rule.id.clone(),
        thread: rule.thread.clone(),
        exists: true,
        is_git: false,
        main: false,
        branch: None,
        detached: false,
        head: None,
        subject: None,
        commit_ms: None,
        index_ms: None,
        dirty: 0,
        ahead: 0,
        behind: 0,
        upstream: None,
        merged: false,
        merged_into: None,
        referenced: None,
        default_names: Vec::new(),
        unmerged: None,
        unmerged_subjects: Vec::new(),
        prunable: None,
        locked: None,
        repo: None,
        registered: false,
        leftover: None,
        error: None,
    }
}

/// A worktree git lists whose folder is gone. Never a doc: it only counts as `gone` in the summary.
pub fn missing_fact(
    item: &ListedWorktree,
    rule: &Rule,
    repo: Option<&str>,
    case_insensitive: bool,
) -> Fact {
    Fact {
        exists: false,
        is_git: true,
        branch: item.branch.clone(),
        detached: item.detached,
        head: item.head.clone(),
        prunable: Some(
            item.prunable
                .clone()
                .unwrap_or_else(|| "folder is gone".to_string()),
        ),
        locked: item.locked.clone(),
        repo: repo.map(ToString::to_string),
        registered: true,
        ..blank_fact(&item.path, rule, case_insensitive)
    }
}
