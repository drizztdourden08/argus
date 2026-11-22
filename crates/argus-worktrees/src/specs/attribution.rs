//! A repository shared by two projects: each worktree is listed under one project only.

use crate::attribution::{Entry, listed_here, own_facts};
use crate::board::{AgentFact, BoardFacts};
use crate::fact::{Fact, blank_fact};
use crate::measure::Leftover;
use crate::paths::join_path;
use crate::rule::Rule;
use crate::testkit::{Fixture, Harness, board_with, put};

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

/// The shared worktree is listed by `here` and not by `not_here`.
fn listed_once(boards: &[Entry], here: &str, not_here: &str) {
    let fact = worktree("eb8e7e24f6");
    assert!(listed_here(here, &fact, boards, &[], true));
    assert!(!listed_here(not_here, &fact, boards, &[], true));
}

#[test]
fn the_board_with_an_agent_in_the_worktree_owns_it() {
    let boards = vec![
        board_with_repo("rotp-web"),
        with_agent(board_with_repo("rotp"), "eb8e7e24f6"),
    ];
    listed_once(&boards, "rotp", "rotp-web");
}

#[test]
fn a_worktree_nobody_owns_stays_with_the_first_registered_project() {
    let boards = vec![board_with_repo("rotp-web"), board_with_repo("rotp")];
    listed_once(&boards, "rotp-web", "rotp");
}

#[test]
fn a_repository_used_by_one_project_is_listed_unchanged() {
    let boards = vec![board_with_repo("rotp"), board_without_repo("other")];
    let facts = vec![worktree("a"), worktree("b")];
    assert_eq!(own_facts("rotp", facts, &boards, &[], true).len(), 2);
}

fn board_with_folder_rule(name: &str) -> Entry {
    let rule = Rule::folder("r2", &format!("{REPO}/.claude/worktrees"), "lead");
    let board = BoardFacts {
        rules: vec![rule],
        ..BoardFacts::default()
    };
    (name.to_string(), board)
}

fn leftover(folder: &str) -> Fact {
    let rule = Rule::folder("r2", &format!("{REPO}/.claude/worktrees"), "lead");
    let mut fact = blank_fact(&format!("{REPO}/.claude/worktrees/{folder}"), &rule, true);
    fact.leftover = Some(Leftover::pending());
    fact
}

#[test]
fn a_project_reaching_the_repository_through_a_folder_rule_shares_it() {
    let boards = vec![board_with_repo("rotp"), board_with_folder_rule("rotp-web")];
    listed_once(&boards, "rotp", "rotp-web");
}

#[test]
fn the_auto_rule_owner_and_a_hand_rule_project_list_a_worktree_once() {
    let auto = Rule::folder("auto", REPO, "lead").repo();
    let hand = Rule::folder("r3", &format!("{REPO}/.claude/worktrees"), "lead");
    let one = BoardFacts {
        rules: vec![auto],
        ..BoardFacts::default()
    };
    let two = BoardFacts {
        rules: vec![hand],
        ..BoardFacts::default()
    };
    let boards = vec![("a".to_string(), one), ("b".to_string(), two)];
    let fact = worktree("x1");
    let count = ["a", "b"]
        .iter()
        .filter(|p| listed_here(p, &fact, &boards, &[], true))
        .count();
    assert_eq!(count, 1);
}

#[test]
fn a_leftover_folder_is_listed_under_one_project() {
    let boards = vec![board_with_repo("rotp"), board_with_folder_rule("rotp-web")];
    let repos = vec![REPO.to_string()];
    let kept = |p: &str| own_facts(p, vec![leftover("web-shots")], &boards, &repos, true).len();
    assert_eq!(kept("rotp"), 1);
    assert_eq!(kept("rotp-web"), 0);
}

#[test]
fn a_leftover_goes_to_the_board_that_works_in_it() {
    let boards = vec![
        board_with_repo("rotp"),
        with_agent(board_with_folder_rule("rotp-web"), "web-shots"),
    ];
    let repos = vec![REPO.to_string()];
    let fact = leftover("web-shots");
    assert!(!listed_here("rotp", &fact, &boards, &repos, true));
    assert!(listed_here("rotp-web", &fact, &boards, &repos, true));
}

fn stale_listed_under(h: &Harness, project: &str) -> bool {
    h.svc.list(project).iter().any(|d| d.name == "stale")
}

#[test]
fn a_leftover_the_live_check_meets_again_stays_with_one_project() {
    let fx = Fixture::fake();
    let dir = join_path(&fx.repo, ".claude/worktrees");
    put(&dir, "stale/readme.txt", "old\n");
    let auto = Rule::folder("auto", &fx.repo, "lead").repo();
    let hand = Rule::folder("r1", &dir, "lead");
    let h = Harness::new(&fx.root, vec![auto, hand.clone()], |_| {});
    let others = &h.store.others;
    others
        .lock()
        .unwrap()
        .insert("beta".to_string(), board_with(vec![hand]));
    h.store.projects.lock().unwrap().push("beta".to_string());
    for project in ["alpha", "beta"] {
        h.svc.scan_now(project);
        h.svc.live_step(project);
    }
    h.svc.live_step("beta");
    assert!(stale_listed_under(&h, "alpha"));
    assert!(!stale_listed_under(&h, "beta"));
}
