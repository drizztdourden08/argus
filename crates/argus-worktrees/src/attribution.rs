//! Whose worktree is it when two projects share one repository? Each worktree is listed under exactly one project.
//!
//! The owner decides, in order: a board whose agent record or open work names the worktree (its folder or its branch);
//! else the first registered project that uses the repository. A repository used by one project is never touched.

use crate::board::BoardFacts;
use crate::fact::Fact;
use crate::hygiene::{links_to, owns};
use crate::paths::path_key;
use crate::rule::RuleKind;

/// A project and its board.
pub(crate) type Entry = (String, BoardFacts);

/// Is `path` the repository or a folder inside it?
fn within(path: &str, repo: &str, ci: bool) -> bool {
    let key = path_key(path, ci);
    key == repo || key.starts_with(&format!("{repo}/"))
}

/// Can the project's scan reach the repository? Any valid repo rule on it, or a folder rule inside it (the automatic
/// rules are already on the board).
fn uses_repo(board: &BoardFacts, repo: &str, ci: bool) -> bool {
    board.valid_rules().iter().any(|r| match r.kind {
        RuleKind::Repo => path_key(&r.path, ci) == repo,
        RuleKind::Folder => within(&r.path, repo, ci),
    })
}

/// The repository a fact belongs to: the one git lists it in, or for a leftover folder the deepest known repository
/// that holds it.
fn repo_of(fact: &Fact, repos: &[String], ci: bool) -> Option<String> {
    if let Some(repo) = &fact.repo {
        return Some(path_key(repo, ci));
    }
    fact.leftover.as_ref()?;
    repos
        .iter()
        .map(|r| path_key(r, ci))
        .filter(|r| within(&fact.path, r, ci) && path_key(&fact.path, ci) != *r)
        .max_by_key(String::len)
}

fn works_on(board: &BoardFacts, fact: &Fact, ci: bool) -> bool {
    board.agents.iter().any(|a| owns(a, fact, ci))
        || board.links.iter().any(|l| links_to(l, fact, ci))
}

/// The project that owns the worktree among the projects that share its repository.
fn owner_of<'a>(fact: &Fact, sharers: &[&'a Entry], ci: bool) -> &'a str {
    let worker = sharers.iter().find(|(_, b)| works_on(b, fact, ci));
    let chosen = worker.or_else(|| sharers.first());
    chosen.map_or("", |(name, _)| name.as_str())
}

/// Does this project list the worktree? True unless another project owns it in a repository both can reach.
pub(crate) fn listed_here(
    project: &str,
    fact: &Fact,
    boards: &[Entry],
    repos: &[String],
    ci: bool,
) -> bool {
    let Some(repo) = repo_of(fact, repos, ci) else {
        return true;
    };
    let sharers: Vec<&Entry> = boards
        .iter()
        .filter(|(_, b)| uses_repo(b, &repo, ci))
        .collect();
    if sharers.len() < 2 || !sharers.iter().any(|(n, _)| n == project) {
        return true;
    }
    owner_of(fact, &sharers, ci) == project
}

/// The facts and leftover folders a project keeps: the ones it owns.
pub(crate) fn own_facts(
    project: &str,
    facts: Vec<Fact>,
    boards: &[Entry],
    known: &[String],
    ci: bool,
) -> Vec<Fact> {
    let mut repos: Vec<String> = known.to_vec();
    repos.extend(facts.iter().filter_map(|f| f.repo.clone()));
    facts
        .into_iter()
        .filter(|f| listed_here(project, f, boards, &repos, ci))
        .collect()
}
