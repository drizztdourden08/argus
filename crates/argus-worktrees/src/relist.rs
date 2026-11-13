//! A folder rule learns a repository's `git worktree list` from a live worktree under it. When the last one is gone, a
//! folder that lost its `.git` is matched against the listings of the repositories an earlier scan knew instead, so it
//! stays `not-git` for its lead and never turns into a leftover that goes by age. Read-only git.

use crate::fact::Fact;
use crate::inspect::{RepoList, ScanCtx};
use crate::parse::parse_worktree_list;
use crate::paths::{join_path, path_key, same_path};
use crate::scan::{ScanEnv, contains_listed};
use std::collections::HashSet;

/// A folder no listing named yet and with no `.git`: the one a missing listing could turn into a leftover.
pub fn undecided(fact: &Fact, listed: &HashSet<String>, env: &ScanEnv<'_>) -> bool {
    let ci = env.case_insensitive;
    fact.exists
        && !fact.is_git
        && fact.error.is_none()
        && fact.leftover.is_none()
        && !env.fs.exists(&join_path(&fact.path, ".git"))
        && !listed.contains(&path_key(&fact.path, ci))
        && !contains_listed(&fact.path, listed, ci)
}

fn has_list(lists: &[RepoList], repo: &str, ci: bool) -> bool {
    lists
        .iter()
        .any(|l| l.items.first().is_some_and(|i| same_path(&i.path, repo, ci)))
}

/// Ask the known repositories not listed yet. False when a repository that still exists did not answer: what it lists
/// is then unknown, and no folder may be taken for a leftover on that.
pub fn relist_known(ctx: &mut ScanCtx<'_>, repos: &[String]) -> bool {
    let mut complete = true;
    for repo in repos {
        if has_list(&ctx.lists, repo, ctx.case_insensitive) || !ctx.fs.is_dir(repo) {
            continue;
        }
        let out = ctx.git.read(repo, &["worktree", "list", "--porcelain"]);
        if out.ok() {
            ctx.lists.push(RepoList {
                items: parse_worktree_list(&out.stdout),
                defaults: None,
            });
        } else {
            complete = false;
        }
    }
    complete
}

/// A folder git lists without a work tree in it keeps the repository that lists it, so the next scan still knows it.
pub fn note_repo(fact: &mut Fact, lists: &[RepoList], ci: bool) {
    if fact.is_git || fact.repo.is_some() {
        return;
    }
    let key = path_key(&fact.path, ci);
    let list = lists
        .iter()
        .find(|l| l.items.iter().any(|i| path_key(&i.path, ci) == key));
    fact.repo = list.and_then(|l| l.items.first().map(|i| i.path.clone()));
}
