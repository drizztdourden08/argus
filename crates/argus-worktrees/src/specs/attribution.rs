//! A repository shared by two projects: each worktree is listed under one project only.

use crate::attribution::{Entry, listed_here, own_facts};
use crate::board::{AgentFact, BoardFacts};
use crate::fact::{Fact, blank_fact};
use crate::rule::Rule;

const REPO: &str = "X:/relic";

fn board_with_repo(name: &str) -> Entry {
    let rule = Rule::folder("r1", REPO, "lead").repo();
    let board = BoardFacts {
        rules: vec![rule],
        ..BoardFacts::default()
    };
    (name.to_string(), board)
}

fn board_without_repo(name: &str) -> Entry {
    (name.to_string(), BoardFacts::default())
}

fn worktree(folder: &str) -> Fact {
    let rule = Rule::folder("r1", REPO, "lead").repo();
    let mut fact = blank_fact(&format!("{REPO}/.claude/worktrees/{folder}"), &rule, true);
    fact.repo = Some(REPO.to_string());
    fact
}

fn with_agent(mut entry: Entry, folder: &str) -> Entry {
    entry.1.agents.push(AgentFact {
        id: "A-1".to_string(),
        status: "running".to_string(),
        worktree: Some(folder.to_string()),
        ..AgentFact::default()
    });
    entry
}

#[test]
fn the_board_with_an_agent_in_the_worktree_owns_it() {
    let boards = vec![
        board_with_repo("rotp-web"),
        with_agent(board_with_repo("rotp"), "eb8e7e24f6"),
    ];
    let fact = worktree("eb8e7e24f6");
    assert!(listed_here("rotp", &fact, &boards, true));
    assert!(!listed_here("rotp-web", &fact, &boards, true));
}

#[test]
fn a_worktree_nobody_owns_stays_with_the_first_registered_project() {
    let boards = vec![board_with_repo("rotp-web"), board_with_repo("rotp")];
    let fact = worktree("eb8e7e24f6");
    assert!(listed_here("rotp-web", &fact, &boards, true));
    assert!(!listed_here("rotp", &fact, &boards, true));
}

#[test]
fn a_repository_used_by_one_project_is_listed_unchanged() {
    let boards = vec![board_with_repo("rotp"), board_without_repo("other")];
    let facts = vec![worktree("a"), worktree("b")];
    assert_eq!(own_facts("rotp", facts, &boards, true).len(), 2);
}
