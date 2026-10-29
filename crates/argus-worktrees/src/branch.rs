//! The branch decision of a removal. Removing the worktree FOLDER loses no commit (they stay in the branch);
//! deleting the BRANCH is its own decision.
//!
//! A branch is MERGED, and may go without anyone's say, when (1) its tip is reachable from a default branch, the
//! remote's or the local one, (2) none of its commits lacks a patch-equivalent in a default branch (`git cherry`: a
//! squash of one commit, a rebase merge), or (3) GitHub says its pull request is merged (read-only, never required,
//! and never asked in local mode).
//! Whatever cannot be established is "not merged": the branch is kept unless the owner's say discards it.

use crate::exec::first_line;
use crate::fact::Fact;
use crate::inspect::{Defaults, find_defaults};
use crate::parse::{ListedWorktree, parse_worktree_list};
use crate::paths::path_key;
use crate::service::{GhCached, Service};

const GH_CACHE_MS: i64 = 2 * 60_000;

/// A commit id as GitHub names it: 7 to 64 hex digits. Anything else is no proof of a merge.
fn is_object_id(text: &str) -> bool {
    (7..=64).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// What is true of a branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchState {
    /// No branch to delete.
    None(String),
    /// The default branch itself: never deleted.
    Default(String),
    /// Checked out in another worktree: kept.
    Elsewhere(String),
    /// Merged, and why; the tip the decision was made for.
    Merged(String, String),
    /// Not merged; the tip the decision was made for.
    Unmerged(String),
}

/// What a removal does about the branch.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BranchPlan {
    pub delete: bool,
    pub force: bool,
    pub why: Option<String>,
    pub note: Option<String>,
    /// The tip the plan was made for: the branch is deleted only while it is still this commit.
    pub tip: Option<String>,
}

/// What happened to the branch.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BranchOutcome {
    pub deleted: bool,
    pub note: Option<String>,
    pub why: Option<String>,
}

struct RepoView {
    defaults: Defaults,
    items: Vec<ListedWorktree>,
}

impl Service {
    fn repo_view(&self, repo: &str) -> RepoView {
        let listing = self
            .deps
            .git
            .read(repo, &["worktree", "list", "--porcelain"]);
        let items = if listing.ok() {
            parse_worktree_list(&listing.stdout)
        } else {
            Vec::new()
        };
        let main_branch = items.first().and_then(|i| i.branch.clone());
        RepoView {
            defaults: find_defaults(&self.deps.git, repo, main_branch.as_deref()),
            items,
        }
    }

    fn gh_says_merged(&self, repo: &str, name: &str, tip: &str) -> bool {
        let url = self.deps.git.read(repo, &["remote", "get-url", "origin"]);
        if !(url.ok() && url.stdout.to_lowercase().contains("github.com")) {
            return false;
        }
        let args: Vec<String> = [
            "pr",
            "list",
            "--state",
            "merged",
            "--head",
            name,
            "--json",
            "number,headRefOid",
            "--limit",
            "1",
        ]
        .map(String::from)
        .to_vec();
        let out = self.deps.gh.run(repo, &args);
        if out.code != Some(0) {
            return false;
        }
        let Ok(rows) = serde_json::from_str::<serde_json::Value>(if out.stdout.trim().is_empty() {
            "[]"
        } else {
            &out.stdout
        }) else {
            return false;
        };
        let Some(pr) = rows
            .as_array()
            .and_then(|a| a.first())
            .filter(|pr| pr.get("number").is_some_and(serde_json::Value::is_u64))
        else {
            return false;
        };
        let oid = pr
            .get("headRefOid")
            .and_then(serde_json::Value::as_str)
            .filter(|oid| is_object_id(oid));
        oid.is_some_and(|oid| {
            oid == tip
                || self
                    .deps
                    .git
                    .read(repo, &["merge-base", "--is-ancestor", tip, oid])
                    .ok()
        })
    }

    /// Does GitHub say the pull request of this branch is merged, and is the branch still what was merged?
    /// Unknown is false. One question per branch and tip every two minutes. In local mode GitHub is not asked at all:
    /// the branch counts as merged only by ancestry or by `git cherry`.
    fn pr_merged(&self, repo: &str, name: &str, tip: &str) -> bool {
        if self.deps.remote.is_local() {
            return false;
        }
        let key = format!("{}\0{name}", path_key(repo, self.ci()));
        let now = self.now();
        if let Some(hit) = self
            .st()
            .gh_cache
            .get(&key)
            .filter(|h| h.tip == tip && now - h.at < GH_CACHE_MS)
        {
            return hit.merged;
        }
        let merged = self.gh_says_merged(repo, name, tip);
        self.st().gh_cache.insert(
            key,
            GhCached {
                tip: tip.to_string(),
                at: now,
                merged,
            },
        );
        merged
    }

    fn where_text(defaults: &Defaults, base: &str) -> String {
        if defaults.remote.as_deref() == Some(base) {
            base.to_string()
        } else {
            format!("local {base} (not pushed)")
        }
    }

    /// What is true of the branch `name` of the worktree at `path`.
    pub(crate) fn branch_state(
        &self,
        repo: &str,
        path: &str,
        name: Option<&str>,
        detached: bool,
    ) -> BranchState {
        let Some(name) = name.filter(|_| !detached) else {
            return BranchState::None("no branch (detached HEAD)".to_string());
        };
        let view = self.repo_view(repo);
        let bases = view.defaults.bases();
        if bases
            .iter()
            .any(|b| b == name || *b == format!("origin/{name}"))
        {
            return BranchState::Default(format!("{name} is the default branch: never deleted"));
        }
        let ci = self.ci();
        if let Some(other) = view.items.iter().find(|x| {
            x.branch.as_deref() == Some(name) && path_key(&x.path, ci) != path_key(path, ci)
        }) {
            return BranchState::Elsewhere(format!(
                "{name} is checked out in {}: kept",
                other.path
            ));
        }
        let reference = format!("refs/heads/{name}");
        let tip = self
            .deps
            .git
            .read(repo, &["rev-parse", "--verify", "-q", &reference]);
        if !tip.ok() {
            return BranchState::None(format!("no branch {name} any more"));
        }
        self.merged_state(repo, &view.defaults, name, tip.stdout.trim())
    }

    /// Is the branch merged: by ancestry into a default branch, by content (`git cherry`), or by its pull request?
    fn merged_state(&self, repo: &str, defaults: &Defaults, name: &str, tip: &str) -> BranchState {
        let bases = defaults.bases();
        let git = &self.deps.git;
        if let Some(base) = bases.iter().find(|b| {
            git.read(repo, &["merge-base", "--is-ancestor", tip, b])
                .ok()
        }) {
            let why = format!("merged into {}", Self::where_text(defaults, base));
            return BranchState::Merged(why, tip.to_string());
        }
        let reference = format!("refs/heads/{name}");
        let cherried = bases.iter().find(|b| {
            let run = git.read(repo, &["cherry", b, &reference]);
            run.ok() && !run.stdout.lines().any(|l| l.starts_with('+'))
        });
        if let Some(base) = cherried {
            let why = format!(
                "its changes are already in {}",
                Self::where_text(defaults, base)
            );
            return BranchState::Merged(why, tip.to_string());
        }
        if self.pr_merged(repo, name, tip) {
            return BranchState::Merged("its pull request is merged".to_string(), tip.to_string());
        }
        BranchState::Unmerged(tip.to_string())
    }

    /// The branch part of a removal. A merged branch goes; an unmerged one only with `discard` (the caller then needs
    /// the owner's say: `force` tells it); otherwise it is kept and the answer says "not merged".
    pub(crate) fn plan_branch(&self, asked: bool, discard: bool, fact: &Fact) -> BranchPlan {
        if !asked {
            return BranchPlan::default();
        }
        let repo = fact.repo.as_deref().unwrap_or(&fact.path);
        match self.branch_state(repo, &fact.path, fact.branch.as_deref(), fact.detached) {
            BranchState::Merged(why, tip) => BranchPlan {
                delete: true,
                why: Some(why),
                tip: Some(tip),
                ..BranchPlan::default()
            },
            BranchState::Unmerged(tip) if discard => BranchPlan {
                delete: true,
                force: true,
                why: Some("discarded with the owner's say (it was not merged)".to_string()),
                tip: Some(tip),
                note: None,
            },
            BranchState::Unmerged(_) => BranchPlan {
                note: Some("not merged".to_string()),
                ..BranchPlan::default()
            },
            BranchState::None(note) | BranchState::Default(note) | BranchState::Elsewhere(note) => {
                BranchPlan {
                    note: Some(note),
                    ..BranchPlan::default()
                }
            }
        }
    }

    /// The worktree that has the branch checked out now, if any.
    fn checked_out_at(&self, repo: &str, branch: &str) -> Option<String> {
        let listing = self
            .deps
            .git
            .read(repo, &["worktree", "list", "--porcelain"]);
        parse_worktree_list(&listing.stdout)
            .into_iter()
            .find(|i| i.branch.as_deref() == Some(branch))
            .map(|i| i.path)
    }

    /// Delete the branch when the plan says so, and only while it still is the commit the plan was made for
    /// (`git update-ref -d` checks the old value): work added after the decision is never lost.
    pub(crate) fn delete_branch(&self, fact: &Fact, plan: &BranchPlan) -> BranchOutcome {
        let kept = |note: Option<String>| BranchOutcome {
            deleted: false,
            note,
            why: None,
        };
        let (true, Some(branch), Some(repo), Some(tip)) = (
            plan.delete,
            fact.branch.as_deref(),
            fact.repo.as_deref(),
            plan.tip.as_deref(),
        ) else {
            return kept(plan.note.clone());
        };
        if let Some(path) = self.checked_out_at(repo, branch) {
            return kept(Some(format!("{branch} is checked out in {path}: kept")));
        }
        let run = self.deps.git.write(
            repo,
            &["update-ref", "-d", &format!("refs/heads/{branch}"), tip],
        );
        if !run.ok() {
            let said = if run.stderr.trim().is_empty() {
                run.error.clone().unwrap_or_else(|| "refused".to_string())
            } else {
                first_line(&run.stderr)
            };
            return kept(Some(format!(
                "git kept {branch}: {said} (it is no longer the commit that was judged)"
            )));
        }
        BranchOutcome {
            deleted: true,
            note: None,
            why: plan.why.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_object_id;

    #[test]
    fn a_commit_id_is_seven_to_sixty_four_hex_digits_and_nothing_else() {
        for ok in ["abcdef1", "ABCDEF1234", &"a".repeat(64)] {
            assert!(is_object_id(ok), "{ok}");
        }
        for bad in ["", "abcde", "--oops", "abcdefg", &"a".repeat(65)] {
            assert!(!is_object_id(bad), "{bad}");
        }
    }
}
