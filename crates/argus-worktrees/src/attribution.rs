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

fn uses_repo(board: &BoardFacts, repo: &str, ci: bool) -> bool {
    board
        .valid_rules()
        .iter()
        .any(|r| r.kind == RuleKind::Repo && path_key(&r.path, ci) == repo)
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

/// Does this project list the worktree? True unless another project owns it in a repository both use.
pub(crate) fn listed_here(project: &str, fact: &Fact, boards: &[Entry], ci: bool) -> bool {
    let Some(repo) = fact.repo.as_deref().map(|r| path_key(r, ci)) else {
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

/// The facts a project keeps: the ones it owns.
pub(crate) fn own_facts(project: &str, facts: Vec<Fact>, boards: &[Entry], ci: bool) -> Vec<Fact> {
    facts
        .into_iter()
        .filter(|f| listed_here(project, f, boards, ci))
        .collect()
}
