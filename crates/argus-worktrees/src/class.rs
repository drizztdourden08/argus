//! The class of a worktree: what it is and what removing it would cost, from the facts the hygiene judges with. It
//! only classifies and shows; no verdict reads it.

use crate::fact::Fact;
use crate::guards::{is_brock_lock, is_claude_lock};
use crate::remove::Safety;
use serde::Serialize;

/// Whose lock holds a worktree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockKind {
    /// Brock's own lock on a clone with `node_modules`.
    Brock,
    /// Claude Code's lock on an agent worktree.
    Claude,
    Other,
}

impl LockKind {
    /// The kind of lock the worktree holds, or none when it is not locked.
    pub fn of(fact: &Fact) -> Option<Self> {
        let reason = fact.locked.as_deref()?;
        Some(if is_brock_lock(reason) {
            Self::Brock
        } else if is_claude_lock(reason) {
            Self::Claude
        } else {
            Self::Other
        })
    }
}

/// What a folder the scan found is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Class {
    /// Clean, and its HEAD is kept by a branch or another ref.
    Clean,
    /// Merged into the default branch and clean.
    Merged,
    /// Merged, and the only changes are untracked files.
    MergedUntracked,
    /// Tracked changes, untracked files on an unmerged tree, a nested repository, or commits no ref keeps.
    WorkLeft,
    /// Git lists it but its folder or its gitdir is gone.
    Prunable,
    /// Locked: the kind is in `lockKind`.
    Locked,
    /// A folder that is not a git work tree (a leftover, or one git cannot read).
    NotWorktree,
    /// The main checkout of a repository.
    Main,
}

impl Class {
    pub fn of(fact: &Fact) -> Self {
        if !fact.is_git {
            return Self::NotWorktree;
        }
        if fact.main {
            return Self::Main;
        }
        if !fact.exists || fact.prunable.is_some() {
            return Self::Prunable;
        }
        if fact.locked.is_some() {
            return Self::Locked;
        }
        Self::of_content(fact)
    }

    /// An unlocked, existing, linked worktree, by what it holds.
    fn of_content(fact: &Fact) -> Self {
        let tracked = fact.dirty.saturating_sub(fact.untracked);
        let lost = fact.error.is_some()
            || tracked > 0
            || fact.nested_git.is_some()
            || !Safety::of(fact).kept;
        match (lost, fact.merged, fact.untracked > 0) {
            (true, _, _) | (false, false, true) => Self::WorkLeft,
            (false, true, true) => Self::MergedUntracked,
            (false, true, false) => Self::Merged,
            (false, false, false) => Self::Clean,
        }
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::blank_fact;
    use crate::rule::Rule;

    fn tree(change: impl FnOnce(&mut Fact)) -> Fact {
        let rule = Rule::folder("W-1", "X:/p/.claude/worktrees", "t");
        let mut fact = blank_fact("X:/p/.claude/worktrees/a", &rule, true);
        fact.is_git = true;
        fact.branch = Some("a".into());
        change(&mut fact);
        fact
    }

    fn class(change: impl FnOnce(&mut Fact)) -> Class {
        Class::of(&tree(change))
    }

    #[test]
    fn a_clean_kept_tree_is_clean_and_a_merged_one_is_merged() {
        assert_eq!(class(|_| {}), Class::Clean);
        assert_eq!(class(|f| f.merged = true), Class::Merged);
    }

    #[test]
    fn untracked_files_alone_tell_merged_untracked_from_work_left() {
        let untracked = |f: &mut Fact| (f.dirty, f.untracked) = (2, 2);
        assert_eq!(
            class(|f| {
                untracked(f);
                f.merged = true;
            }),
            Class::MergedUntracked
        );
        assert_eq!(class(untracked), Class::WorkLeft);
        assert_eq!(
            class(|f| (f.dirty, f.untracked, f.merged) = (3, 2, true)),
            Class::WorkLeft,
            "a tracked change is work left, merged or not"
        );
    }

    #[test]
    fn commits_no_ref_keeps_a_nested_repository_and_an_error_are_work_left() {
        let detached = |f: &mut Fact| (f.branch, f.detached) = (None, true);
        assert_eq!(class(detached), Class::WorkLeft);
        assert_eq!(
            class(|f| {
                detached(f);
                f.merged = true;
            }),
            Class::Merged,
            "a detached HEAD merged into main loses nothing"
        );
        assert_eq!(
            class(|f| f.nested_git = Some("X:/p/a/vendor/.git".into())),
            Class::WorkLeft
        );
        assert_eq!(
            class(|f| f.error = Some("git failed".into())),
            Class::WorkLeft
        );
    }

    #[test]
    fn prunable_locked_not_a_tree_and_main_come_before_the_content() {
        assert_eq!(class(|f| f.prunable = Some("gone".into())), Class::Prunable);
        assert_eq!(class(|f| f.exists = false), Class::Prunable);
        assert_eq!(
            class(|f| f.locked = Some("hands off".into())),
            Class::Locked
        );
        assert_eq!(class(|f| f.is_git = false), Class::NotWorktree);
        assert_eq!(class(|f| (f.main, f.dirty) = (true, 4)), Class::Main);
    }

    #[test]
    fn a_lock_is_brocks_claudes_or_another() {
        let kind =
            |reason: Option<&str>| LockKind::of(&tree(|f| f.locked = reason.map(String::from)));
        assert_eq!(kind(None), None);
        assert_eq!(
            kind(Some("cloned node_modules from X:/brock")),
            Some(LockKind::Brock)
        );
        assert_eq!(
            kind(Some("claude agent agent-a1 (pid 4242)")),
            Some(LockKind::Claude)
        );
        assert_eq!(kind(Some("owner: hands off")), Some(LockKind::Other));
    }
}
