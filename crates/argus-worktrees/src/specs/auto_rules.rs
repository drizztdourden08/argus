//! I-156 A: the automatic repo rules. A board with no repo rule gets a derived one, a board with one does not, two
//! boards on one root share one, and the listed extra repos are covered. In report mode the hygiene leaves the
//! worktrees under an automatic rule alone and the plan lists what it would remove; in on mode it removes them after
//! the debounce.

// test-tag: critical

use crate::auto::{AutoBoard, AutoMode, AutoRule, ExtraRepo, auto_rules};
use crate::rule::{Rule, RuleKind};
use crate::specs::hygiene_kit::{DEBOUNCE, hygiene_on, step, through_debounce};
use crate::testkit::{Fixture, Harness, exists_itself, records};

fn owned(made: &[AutoRule]) -> Vec<(String, String)> {
    made.iter()
        .map(|a| (a.owner.clone(), a.rule.path.clone()))
        .collect()
}

#[test]
fn a_board_with_no_repo_rule_gets_one_and_a_board_with_one_does_not() {
    let folder = vec![Rule::folder("W-1", "X:/a/.claude/worktrees", "lead")];
    let repo = vec![Rule::folder("W-1", "X:/b", "lead").repo()];
    let boards = [
        AutoBoard {
            key: "a",
            root: Some("X:/a"),
            rules: &folder,
        },
        AutoBoard {
            key: "b",
            root: Some("X:/b"),
            rules: &repo,
        },
        AutoBoard {
            key: "c",
            root: None,
            rules: &[],
        },
    ];
    let made = auto_rules(&boards, &[], AutoMode::Report, false);
    assert_eq!(owned(&made), [("a".to_string(), "X:/a".to_string())]);
    assert!(made[0].rule.auto);
    assert_eq!(made[0].rule.kind, RuleKind::Repo);
    assert_eq!(auto_rules(&boards, &[], AutoMode::Off, false).len(), 0);
}

#[test]
fn two_boards_on_one_root_get_one_rule_owned_by_the_first() {
    let boards = [
        AutoBoard {
            key: "rotp",
            root: Some("X:/rotp"),
            rules: &[],
        },
        AutoBoard {
            key: "rotp-web",
            root: Some("x:/rotp/"),
            rules: &[],
        },
    ];
    let made = auto_rules(&boards, &[], AutoMode::On, true);
    assert_eq!(owned(&made), [("rotp".to_string(), "X:/rotp".to_string())]);
}

#[test]
fn an_extra_repo_is_covered_by_the_owner_board() {
    let extras = [ExtraRepo {
        path: "X:/td-music-studio".to_string(),
        owner: "portfolio".to_string(),
    }];
    let made = auto_rules(&[], &extras, AutoMode::Report, false);
    assert_eq!(
        owned(&made),
        [("portfolio".to_string(), "X:/td-music-studio".to_string())]
    );
}

/// A clean sibling nothing is attached to, under an automatic repo rule on the fixture's repository.
fn sibling_world(mode: AutoMode) -> (Fixture, Harness) {
    let fx = Fixture::fake_bare();
    fx.add("sib");
    let h = auto_harness(&fx, mode, Vec::new());
    (fx, h)
}

fn auto_harness(fx: &Fixture, mode: AutoMode, keep: Vec<String>) -> Harness {
    let rule = Rule {
        auto: true,
        ..Rule::folder("auto-alpha", &fx.repo, "lead").repo()
    };
    let h = Harness::new(&fx.root, vec![rule], |o| {
        hygiene_on(o);
        o.auto_rules = mode;
        o.keep = keep;
    });
    h.scan();
    h.svc.start();
    h
}

#[test]
fn in_report_mode_a_clean_sibling_stays_and_the_plan_lists_it() {
    let (fx, h) = sibling_world(AutoMode::Report);
    let path = fx.path("sib");
    assert!(!through_debounce(&h, &path), "report mode removes nothing");
    let plan = h.svc.auto_plan();
    let row = plan.iter().find(|r| r.path.ends_with("/sib")).unwrap();
    assert!(row.would_remove);
    assert_eq!(row.board, "alpha");
    assert_eq!(h.svc.hygiene_step("alpha").told, 0);
}

#[test]
fn a_keep_pattern_holds_a_sibling_even_in_on_mode_and_the_plan_says_why() {
    let fx = Fixture::fake_bare();
    fx.add("sib");
    fx.add("other");
    let pattern = fx.path("sib*");
    let h = auto_harness(&fx, AutoMode::On, vec![pattern.clone()]);
    assert!(through_debounce(&h, &fx.path("other")));
    assert!(exists_itself(&fx.path("sib")), "the pattern holds it");
    let plan = h.svc.auto_plan();
    let row = plan.iter().find(|r| r.path.ends_with("/sib")).unwrap();
    assert!(!row.would_remove);
    assert_eq!(row.reason, format!("kept by the keep pattern {pattern}"));
}

#[test]
fn in_on_mode_the_sibling_is_removed_after_the_debounce() {
    let (fx, h) = sibling_world(AutoMode::On);
    let path = fx.path("sib");
    step(&h, DEBOUNCE / 2);
    assert!(exists_itself(&path), "not before the debounce");
    assert!(through_debounce(&h, &path));
}

#[test]
fn in_report_mode_a_gone_record_is_not_pruned_and_a_dead_pid_lock_is_not_unlocked() {
    let fx = Fixture::fake_bare();
    fx.add("gone");
    fx.add("held");
    fx.git(&[
        "worktree",
        "lock",
        "--reason",
        "claude agent agent-a1 (pid 4242)",
        &fx.path("held"),
    ]);
    std::fs::remove_dir_all(fx.path("gone")).unwrap();
    let h = auto_harness(&fx, AutoMode::Report, Vec::new());
    step(&h, 1000);
    assert!(records(&fx).contains(&"gone".to_string()));
    assert_eq!(h.notes.removals("pruned").len(), 0);
    assert_eq!(h.notes.removals("unlocked").len(), 0);
    assert!(
        fx.git(&["worktree", "list", "--porcelain"])
            .contains("locked claude agent")
    );
}
