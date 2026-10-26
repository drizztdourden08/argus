//! The branch decision of a removal. Removing the worktree FOLDER loses no commit (they stay in the branch);
//! deleting the BRANCH is its own decision.
//!
//! A branch is MERGED, and may go without anyone's say, when (1) its tip is reachable from a default branch, the
//! remote's or the local one, (2) none of its commits lacks a patch-equivalent in a default branch (`git cherry`: a
//! squash of one commit, a rebase merge), or (3) GitHub says its pull request is merged (read-only, never required).
//! Whatever cannot be established is "not merged": the branch is kept unless the owner's say discards it.

use crate::exec::first_line;
use crate::fact::Fact;
use crate::inspect::{Defaults, find_defaults};
use crate::parse::{ListedWorktree, parse_worktree_list};
use crate::paths::path_key;
use crate::service::{GhCached, Service};

const GH_CACHE_MS: i64 = 2 * 60_000;

/// What is true of a branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BranchState {
    /// No branch to delete.
    None(String),
    /// The default branch itself: never deleted.
    Default(String),
    /// Checked out in another worktree: kept.
    Elsewhere(String),
    Merged(String),
    Unmerged,
}

/// What a removal does about the branch.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BranchPlan {
    pub delete: bool,
    pub force: bool,
    pub why: Option<String>,
    pub note: Option<String>,
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
        let oid = pr.get("headRefOid").and_then(serde_json::Value::as_str);
        oid.is_none_or(|oid| {
            oid == tip
                || self
                    .deps
                    .git
                    .read(repo, &["merge-base", "--is-ancestor", tip, oid])
                    .ok()
        })
    }

    /// Does GitHub say the pull request of this branch is merged, and is the branch still what was merged?
    /// Unknown is false. One question per branch and tip every two minutes.
    fn pr_merged(&self, repo: &str, name: &str, tip: &str) -> bool {
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
            return BranchState::Merged(format!(
                "merged into {}",
                Self::where_text(defaults, base)
            ));
        }
        let reference = format!("refs/heads/{name}");
        let cherried = bases.iter().find(|b| {
            let run = git.read(repo, &["cherry", b, &reference]);
            run.ok() && !run.stdout.lines().any(|l| l.starts_with('+'))
        });
        if let Some(base) = cherried {
            return BranchState::Merged(format!(
                "its changes are already in {}",
                Self::where_text(defaults, base)
            ));
        }
        if self.pr_merged(repo, name, tip) {
            return BranchState::Merged("its pull request is merged".to_string());
        }
        BranchState::Unmerged
    }

    /// The branch part of a removal. A merged branch goes; an unmerged one only with `discard` (the caller then needs
    /// the owner's say: `force` tells it); otherwise it is kept and the answer says "not merged".
    pub(crate) fn plan_branch(&self, asked: bool, discard: bool, fact: &Fact) -> BranchPlan {
        if !asked {
            return BranchPlan::default();
        }
        let repo = fact.repo.as_deref().unwrap_or(&fact.path);
        match self.branch_state(repo, &fact.path, fact.branch.as_deref(), fact.detached) {
            BranchState::Merged(why) => BranchPlan {
                delete: true,
                why: Some(why),
                ..BranchPlan::default()
            },
            BranchState::Unmerged if discard => BranchPlan {
                delete: true,
                force: true,
                why: Some("discarded with the owner's say (it was not merged)".to_string()),
                note: None,
            },
            BranchState::Unmerged => BranchPlan {
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

    /// Delete the branch when the plan says so. `-D`, because the merge was proven here (`-d` judges against the main
    /// checkout's HEAD, not the default branch).
    pub(crate) fn delete_branch(&self, fact: &Fact, plan: &BranchPlan) -> BranchOutcome {
        let (true, Some(branch), Some(repo)) =
            (plan.delete, fact.branch.as_deref(), fact.repo.as_deref())
        else {
            return BranchOutcome {
                deleted: false,
                note: plan.note.clone(),
                why: None,
            };
        };
        let run = self.deps.git.write(repo, &["branch", "-D", branch]);
        if !run.ok() {
            let said = if run.stderr.trim().is_empty() {
                run.error.clone().unwrap_or_else(|| "refused".to_string())
            } else {
                first_line(&run.stderr)
            };
            return BranchOutcome {
                deleted: false,
                note: Some(format!("git kept {branch}: {said}")),
                why: None,
            };
        }
        BranchOutcome {
            deleted: true,
            note: None,
            why: plan.why.clone(),
        }
    }
}
