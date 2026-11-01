//! KEEP: K1 (the status of a worktree and the wake sentence built from the rows).
//!
//! The status judge and the wake text on facts the tests build in memory: no repository, no folder, no process.

// test-tag: critical

use crate::board::Keep;
use crate::clock::{DAY_MS, HOUR_MS};
use crate::doc::{Status, WorktreeDoc};
use crate::fact::{Fact, blank_fact};
use crate::rule::Rule;
use crate::status::{Judging, by_worst, classify, doc_of, many_text, wake_text};
use crate::waking::REASON;

const NOW: i64 = 1_792_929_600_000;

fn fact(change: impl FnOnce(&mut Fact)) -> Fact {
    let rule = Rule::folder("W-1", "X:/p/wt", "lead");
    let mut fact = blank_fact("X:/p/wt/a", &rule, true);
    (fact.is_git, fact.branch, fact.commit_ms) = (true, Some("a".into()), Some(0));
    change(&mut fact);
    fact
}

fn status_of(change: impl FnOnce(&mut Fact), keep: Option<&Keep>) -> Status {
    let rule = Rule::folder("W-1", "X:/p/wt", "lead");
    let judging = Judging {
        rule: Some(&rule),
        keep,
        now_ms: NOW,
        default_stale_days: None,
    };
    classify(&fact(change), &judging).status
}

#[test]
fn active_inside_stale_days_stale_from_then_on_the_newest_of_commit_and_index_counts() {
    let commit = |ago: i64| move |f: &mut Fact| f.commit_ms = Some(NOW - ago);
    assert_eq!(status_of(commit(DAY_MS * 29 / 10), None), Status::Active);
    assert_eq!(status_of(commit(3 * DAY_MS), None), Status::Stale);
    assert_eq!(
        status_of(
            |f| (f.commit_ms, f.index_ms) = (Some(NOW - 9 * DAY_MS), Some(NOW - HOUR_MS)),
            None
        ),
        Status::Active,
        "a recent index keeps an old commit active"
    );
    assert_eq!(
        status_of(
            |f| (f.commit_ms, f.index_ms) = (Some(NOW - HOUR_MS), Some(NOW - 9 * DAY_MS)),
            None
        ),
        Status::Active
    );
    let judging = Judging {
        now_ms: NOW,
        ..Judging::default()
    };
    let aged =
        fact(|f| (f.commit_ms, f.index_ms) = (Some(NOW - 9 * DAY_MS), Some(NOW - 5 * DAY_MS)));
    assert_eq!(classify(&aged, &judging).age_days, Some(5));
}

#[test]
fn the_rule_overrides_the_default_of_three_days_and_the_service_default_is_configurable() {
    let seven = Rule {
        stale_days: Some(7),
        ..Rule::folder("W-1", "X:/p/wt", "lead")
    };
    let at = |ago: i64, rule: Option<&Rule>, default: Option<u32>| {
        let judging = Judging {
            rule,
            now_ms: NOW,
            default_stale_days: default,
            keep: None,
        };
        classify(&fact(|f| f.commit_ms = Some(NOW - ago)), &judging).status
    };
    assert_eq!(at(5 * DAY_MS, Some(&seven), None), Status::Active);
    assert_eq!(at(8 * DAY_MS, Some(&seven), None), Status::Stale);
    assert_eq!(at(5 * DAY_MS, None, Some(10)), Status::Active);
}

#[test]
fn merged_needs_stale_and_merged_and_clean_dirty_wins_and_a_main_checkout_is_never_merged() {
    let old = |f: &mut Fact| f.commit_ms = Some(NOW - 9 * DAY_MS);
    assert_eq!(
        status_of(
            |f| {
                old(f);
                f.merged = true;
            },
            None
        ),
        Status::Merged
    );
    assert_eq!(
        status_of(
            |f| {
                old(f);
                f.merged = true;
                f.dirty = 2;
            },
            None
        ),
        Status::DirtyStale
    );
    assert_eq!(
        status_of(
            |f| {
                old(f);
                f.dirty = 1;
            },
            None
        ),
        Status::DirtyStale
    );
    assert_eq!(
        status_of(
            |f| {
                old(f);
                f.merged = true;
                f.main = true;
            },
            None
        ),
        Status::Stale
    );
    assert_eq!(
        status_of(
            |f| {
                f.commit_ms = Some(NOW - HOUR_MS);
                f.merged = true;
            },
            None
        ),
        Status::Active,
        "a fresh worktree on a merged branch is not \"can go\""
    );
}

fn day_keep() -> Keep {
    Keep {
        until_ms: NOW + DAY_MS,
        note: Some("n".into()),
        by: Some("lead".into()),
    }
}

fn old_commit(f: &mut Fact) {
    f.commit_ms = Some(NOW - 9 * DAY_MS);
}

#[test]
fn missing_not_git_and_error_are_judged_from_the_facts() {
    assert_eq!(status_of(|f| f.exists = false, None), Status::Missing);
    assert_eq!(status_of(|f| f.is_git = false, None), Status::NotGit);
    let broken = |f: &mut Fact| {
        f.is_git = false;
        f.error = Some("dubious ownership".into());
    };
    assert_eq!(status_of(broken, None), Status::Error);
}

#[test]
fn a_keep_snoozes_only_the_ones_that_need_a_decision_and_an_expired_keep_is_no_keep() {
    let (keep, expired) = (
        day_keep(),
        Keep {
            until_ms: NOW - 1,
            ..day_keep()
        },
    );
    assert_eq!(status_of(old_commit, Some(&keep)), Status::Kept);
    assert_eq!(status_of(|f| f.exists = false, Some(&keep)), Status::Kept);
    assert_eq!(
        status_of(|f| f.commit_ms = Some(NOW - HOUR_MS), Some(&keep)),
        Status::Active,
        "a keep never hides an active worktree"
    );
    assert_eq!(status_of(|f| f.is_git = false, Some(&keep)), Status::NotGit);
    assert_eq!(
        status_of(old_commit, Some(&expired)),
        Status::Stale,
        "an expired keep is no keep"
    );
}

#[test]
fn stale_merged_and_dirty_stale_need_a_decision_and_the_wake_reason_is_known() {
    assert!(
        Status::Stale.needs_decision()
            && Status::Merged.needs_decision()
            && Status::DirtyStale.needs_decision()
    );
    assert!(!Status::Kept.needs_decision() && !Status::Leftover.needs_decision());
    assert_eq!(REASON, "worktree-stale");
}

fn doc(path: &str, status: Status, age: i64, ahead: u32) -> WorktreeDoc {
    let rule = Rule::folder("W-1", "X:/t", "lead");
    let name = path.rsplit('/').next().unwrap_or(path).to_string();
    let changed = fact(|f| {
        f.path = path.to_string();
        f.id = path.to_string();
        f.commit_ms = Some(NOW - age * DAY_MS);
        f.merged = status == Status::Merged;
        f.dirty = if status == Status::DirtyStale { 3 } else { 0 };
        f.exists = status != Status::Missing;
        f.ahead = ahead;
        f.branch = Some(name);
    });
    doc_of(
        &changed,
        &Judging {
            rule: Some(&rule),
            now_ms: NOW,
            ..Judging::default()
        },
        None,
    )
}

const TAIL: &str =
    "Keep it (coord worktree keep), remove it (coord worktree remove), or drop its rule.";

#[test]
fn the_wake_text_lists_worst_first_with_the_first_path_in_full_and_the_others_short() {
    let mut list = [
        doc("X:/t/.claude/worktrees/bar", Status::Merged, 4, 0),
        doc("X:/t/.claude/worktrees/foo", Status::Stale, 6, 2),
        doc("X:/t/.claude/worktrees/zed", Status::Missing, 6, 0),
    ];
    list.sort_by(by_worst);
    assert_eq!(
        list.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(),
        ["zed", "foo", "bar"]
    );
    let refs: Vec<&WorktreeDoc> = list.iter().collect();
    let want = format!(
        "3 worktrees need a decision: X:/t/.claude/worktrees/zed missing (git still lists it), .../foo stale 6d (branch foo, 2 ahead), .../bar merged and clean (can go). {TAIL}"
    );
    assert_eq!(wake_text(&refs, ""), want);
    let one = doc("X:/t/w/a", Status::DirtyStale, 6, 0);
    assert!(wake_text(&[&one], "").starts_with(
        "1 worktree needs a decision: X:/t/w/a stale 6d with 3 uncommitted (branch a). Keep it"
    ));
}

#[test]
fn many_long_paths_are_cut_to_300_characters_with_the_count_of_the_rest() {
    let mut list: Vec<WorktreeDoc> = (0..12)
        .map(|i| {
            doc(
                &format!("X:/some/long/project/folder/.claude/worktrees/feature-number-{i}"),
                Status::Stale,
                20 - i,
                0,
            )
        })
        .collect();
    list.sort_by(by_worst);
    let refs: Vec<&WorktreeDoc> = list.iter().collect();
    let text = wake_text(&refs, "");
    assert!(text.chars().count() <= 300, "{}", text.len());
    assert!(text.starts_with("12 worktrees need a decision: X:/some/long/project/folder/.claude/worktrees/feature-number-0 stale 20d"));
    assert!(text.contains("more).") && text.ends_with(TAIL));
    let huge = doc(&format!("X:/{}wt", "deep/".repeat(40)), Status::Stale, 6, 0);
    assert!(wake_text(&[&huge], "").chars().count() <= 300);
}

fn doc_with(path: &str, age: i64) -> WorktreeDoc {
    let rule = Rule::folder("W-1", "X:/Bxel/.claude/worktrees", "lead");
    let mut fact = crate::fact::blank_fact(path, &rule, true);
    (fact.is_git, fact.branch, fact.commit_ms, fact.ahead) = (
        true,
        Some(format!("feature/some-long-branch-name-{age}")),
        Some(1_792_929_600_000 - age * DAY_MS),
        2,
    );
    crate::status::doc_of(
        &fact,
        &crate::status::Judging {
            rule: Some(&rule),
            now_ms: 1_792_929_600_000,
            ..crate::status::Judging::default()
        },
        None,
    )
}

#[test]
fn many_text_counts_git_worktrees_and_leftovers_apart_and_the_wake_keeps_the_sentence_first_within_300_characters()
 {
    let sentence = many_text("Bxel", 100, 8, 92, 6144.0);
    assert_eq!(
        sentence,
        "Bxel has 100 folders: 8 git worktrees and 92 leftovers (about 6 GB): `coord worktree remove --all-leftovers` clears the old ones."
    );
    assert_eq!(
        many_text("Bxel", 13, 12, 1, 0.2),
        "Bxel has 13 folders: 12 git worktrees and 1 leftover: `coord worktree remove --all-leftovers` clears the old ones.",
        "under 1 MB says no size"
    );
    assert_eq!(
        many_text("Alpha", 14, 14, 0, 0.0),
        "Alpha has 14 worktrees: that is a lot, remove the ones you no longer need (coord worktree list, coord worktree remove)."
    );
    let docs: Vec<WorktreeDoc> = (5..10)
        .map(|i| {
            doc_with(
                &format!("X:/Bxel/.claude/worktrees/some-long-worktree-name-{i}"),
                i,
            )
        })
        .collect();
    let refs: Vec<&WorktreeDoc> = docs.iter().collect();
    let text = wake_text(&refs, &sentence);
    assert!(
        text.chars().count() <= 300,
        "{} characters",
        text.chars().count()
    );
    assert!(text.starts_with(&format!("{sentence} ")));
}
