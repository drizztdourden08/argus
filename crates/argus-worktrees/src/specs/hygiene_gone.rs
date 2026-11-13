//! KEEP: K3, K4.
//! The hygiene step cleans what git still lists but nothing holds: it forgets the records of deleted folders that lose
//! no commit, and it lifts a Claude Code lock whose pid is gone, so the worktree is then judged like any other.

// test-tag: standard

use crate::specs::hygiene_kit::{DEBOUNCE, left_why, step, through_debounce, world, world_alive};
use crate::testkit::{Fixture, Harness, records};
use std::fs;

const DEAD: &str = "claude agent agent-a1 (pid 4242)";

fn delete_folder(fx: &Fixture, name: &str) {
    fs::remove_dir_all(fx.path(name)).unwrap();
}

fn pruned_lines(h: &Harness) -> usize {
    h.notes.removals("pruned").len()
}

#[test]
fn a_worktree_whose_folder_was_deleted_is_pruned_by_the_step_the_scan_runs() {
    let (fx, h) = world(|fx| {
        fx.add("agent-a9");
        delete_folder(fx, "agent-a9");
    });
    assert_eq!(records(&fx), vec!["agent-a1".to_string()]);
    assert_eq!(pruned_lines(&h), 1);
    step(&h, DEBOUNCE);
    assert_eq!(pruned_lines(&h), 1, "nothing is pruned twice");
}

#[test]
fn a_gone_worktree_with_commits_no_ref_keeps_is_left() {
    let (fx, h) = world(|fx| {
        fx.add_detached_work("agent-a8");
        delete_folder(fx, "agent-a8");
    });
    step(&h, DEBOUNCE);
    assert!(records(&fx).contains(&"agent-a8".to_string()));
    assert_eq!(pruned_lines(&h), 0);
}

#[test]
fn a_stray_record_outside_every_rule_does_not_block_the_rest() {
    let (fx, h) = world(|fx| {
        fx.add("agent-a9");
        delete_folder(fx, "agent-a9");
        crate::testkit::unplugged(false)(fx);
    });
    step(&h, DEBOUNCE);
    let left = records(&fx);
    assert!(left.contains(&"drive-x".to_string()), "the stray stays");
    assert!(!left.contains(&"agent-a9".to_string()));
    assert_eq!(
        fx.git(&["worktree", "list", "--porcelain"])
            .matches("locked")
            .count(),
        0
    );
}

fn lock_agent(fx: &Fixture, reason: &str) {
    fx.git(&["worktree", "lock", "--reason", reason, &fx.path("agent-a1")]);
}

#[test]
fn a_claude_lock_with_a_dead_pid_is_unlocked_and_the_clean_worktree_removed() {
    let (fx, h) = world(|fx| lock_agent(fx, DEAD));
    step(&h, 1000);
    assert_eq!(h.notes.removals("unlocked").len(), 1);
    assert!(through_debounce(&h, &fx.path("agent-a1")));
}

#[test]
fn a_claude_lock_with_a_live_pid_stays_and_asks_the_lead() {
    let (fx, h) = world_alive(&[4242], |fx| lock_agent(fx, DEAD));
    step(&h, 1000);
    assert_eq!(h.notes.removals("unlocked").len(), 0);
    assert_eq!(left_why(&h, "agent-a1"), format!("it is locked ({DEAD})"));
    assert!(!through_debounce(&h, &fx.path("agent-a1")));
}

#[test]
fn a_lock_that_is_not_claudes_or_names_no_pid_is_unchanged() {
    for reason in ["owner: hands off", "claude agent agent-a1"] {
        let (fx, h) = world(|fx| lock_agent(fx, reason));
        step(&h, 1000);
        assert_eq!(h.notes.removals("unlocked").len(), 0);
        assert_eq!(left_why(&h, "agent-a1"), format!("it is locked ({reason})"));
        assert!(!through_debounce(&h, &fx.path("agent-a1")));
    }
}
