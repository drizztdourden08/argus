//! Worktree rules: where a project's worktrees live.

use crate::paths::{has_wildcard, name_allowed};
use serde::{Deserialize, Serialize};

/// The default number of days without a touch before a worktree is stale.
pub const DEFAULT_STALE_DAYS: u32 = 3;

/// What a rule's path is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleKind {
    /// The path is the parent folder; every allowed sub-folder is a candidate.
    Folder,
    /// The path is one git repository; its `git worktree list` is the source.
    Repo,
}

/// One rule of a project's board. The path is always the literal parent folder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub path: String,
    pub kind: RuleKind,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub thread: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub stale_days: Option<u32>,
    #[serde(default)]
    pub added_by: Option<String>,
    #[serde(default)]
    pub added_at: Option<String>,
}

impl Rule {
    /// A folder rule with no name filters.
    pub fn folder(id: &str, path: &str, thread: &str) -> Self {
        Self {
            id: id.to_string(),
            path: path.to_string(),
            kind: RuleKind::Folder,
            include: Vec::new(),
            exclude: Vec::new(),
            thread: Some(thread.to_string()),
            note: None,
            stale_days: None,
            added_by: None,
            added_at: None,
        }
    }

    /// The same rule as a repo rule.
    #[must_use]
    pub fn repo(mut self) -> Self {
        self.kind = RuleKind::Repo;
        self
    }

    /// The same rule with name filters.
    #[must_use]
    pub fn filtered(mut self, include: &[&str], exclude: &[&str]) -> Self {
        self.include = include.iter().map(ToString::to_string).collect();
        self.exclude = exclude.iter().map(ToString::to_string).collect();
        self
    }

    /// Why a stored rule cannot be used, or none. The scan and every delete guard ignore a rule that has one.
    pub fn problem(&self) -> Option<&'static str> {
        has_wildcard(&self.path).then_some("wildcard path")
    }

    /// Does the rule let this direct-child folder name through?
    pub fn allows(&self, name: &str, case_insensitive: bool) -> bool {
        name_allowed(&self.include, &self.exclude, name, case_insensitive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wildcard_path_is_invalid() {
        assert_eq!(
            Rule::folder("W-1", "X:/a/web-*", "t").problem(),
            Some("wildcard path")
        );
        assert_eq!(
            Rule::folder("W-1", "X:/a/b?", "t").problem(),
            Some("wildcard path")
        );
        assert_eq!(Rule::folder("W-1", "X:/a/b", "t").problem(), None);
    }

    #[test]
    fn filters_apply_to_names() {
        let rule = Rule::folder("W-1", "X:/a", "t").filtered(&["web-*"], &["web-2*"]);
        assert!(rule.allows("web-1", false));
        assert!(!rule.allows("web-22", false));
        assert!(!rule.allows("agent", false));
    }

    #[test]
    fn a_rule_reads_from_json_with_missing_lists() {
        let rule: Rule =
            serde_json::from_str(r#"{"id":"W-1","path":"X:/a","kind":"repo","staleDays":7}"#)
                .unwrap();
        assert_eq!(
            (rule.kind, rule.stale_days, rule.include.len()),
            (RuleKind::Repo, Some(7), 0)
        );
    }
}
