//! The service: statuses, the docs and the summary it mirrors, the owner's requests, keeps, and the stale signal.

use crate::board::{Answer, EntryWorktree, Keep, OwnerEntry};
use crate::clock::{DAY_MS, HOUR_MS};
use crate::doc::{Status, WorktreeDoc};
use crate::fact::{Fact, blank_fact};
use crate::paths::worktree_id;
use crate::rule::Rule;
use crate::service::Options;
use crate::status::{Judging, by_worst, classify, doc_of, wake_text};
use crate::testkit::{Fixture, Harness, RemoveMode, board_with, rule_for};
use crate::waking::{REASON, sent_key};
use serde_json::Value;

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

fn started(fx: &Fixture, options: impl FnOnce(&mut Options)) -> Harness {
    let h = Harness::new(&fx.root, vec![rule_for(fx)], options);
    h.scan();
    h
}

fn names_of(h: &Harness) -> Vec<String> {
    let mut names: Vec<String> = h.docs().keys().cloned().collect();
    names.sort();
    names
}

fn two_rules_started(fx: &Fixture) -> Harness {
    let nowhere = Rule {
        stale_days: Some(7),
        ..Rule::folder("W-2", &format!("{}/nowhere", fx.root), "lead")
    };
    let first = Rule {
        note: Some("x".into()),
        ..rule_for(fx)
    };
    let h = Harness::new(&fx.root, vec![first, nowhere], |_| {});
    h.scan();
    h
}

#[test]
fn a_scan_mirrors_one_doc_per_worktree_and_the_gone_folder_is_not_a_row() {
    let fx = Fixture::new();
    let h = two_rules_started(&fx);
    assert_eq!(
        names_of(&h),
        ["dirty-old", "fresh", "old-merged", "old-unmerged", "plain"]
    );
    let d = h.docs();
    let status = |n: &str| d[n].status.as_str();
    assert_eq!(status("dirty-old"), "dirty-stale");
    assert_eq!((status("fresh"), status("plain")), ("active", "leftover"));
    assert_eq!(
        (status("old-merged"), status("old-unmerged")),
        ("merged", "stale")
    );
}

#[test]
fn a_row_carries_the_branch_the_commit_the_counts_and_what_the_default_branch_lacks() {
    let fx = Fixture::new();
    let h = two_rules_started(&fx);
    let d = h.docs();
    let fresh = &d["fresh"];
    assert_eq!(
        (fresh.rule.as_str(), fresh.thread.as_deref()),
        ("W-1", Some("worker"))
    );
    assert_eq!(
        (fresh.branch.as_deref(), fresh.subject.as_deref()),
        (Some("fresh"), Some("add fresh.txt"))
    );
    assert_eq!((fresh.dirty, fresh.age_days), (0, Some(0)));
    assert_eq!(fresh.id, worktree_id(&fx.path("fresh"), cfg!(windows)));
    assert_eq!(fresh.head.as_deref().map(str::len), Some(7));
    let old = &d["old-unmerged"];
    assert_eq!((old.age_days, old.unmerged), (Some(10), Some(1)));
    assert_eq!(old.unmerged_subjects, ["add old.txt"]);
    assert_eq!(
        (d["old-merged"].merged, d["old-merged"].unmerged),
        (true, Some(0))
    );
    assert_eq!(d["dirty-old"].dirty, 1);
}

#[test]
fn the_summary_counts_the_rows_by_status_and_reports_each_rule() {
    let fx = Fixture::new();
    let h = two_rules_started(&fx);
    let meta = h.meta();
    assert_eq!((meta.count, meta.gone), (5, 1));
    assert_eq!(
        (meta.many_at, meta.too_many, meta.scan_ms),
        (12, false, 1_800_000)
    );
    let by: Vec<(&str, usize)> = meta
        .by_status
        .iter()
        .map(|(k, v)| (k.as_str(), *v))
        .collect();
    assert_eq!(
        by,
        [
            ("active", 1),
            ("dirty-stale", 1),
            ("leftover", 1),
            ("merged", 1),
            ("stale", 1)
        ]
    );
    let first = &meta.rules[0];
    assert_eq!(
        (
            first.exists,
            first.matched,
            first.candidates,
            first.stale_days
        ),
        (Some(true), Some(1), Some(5), None)
    );
    let second = &meta.rules[1];
    assert_eq!(
        (
            second.exists,
            second.matched,
            second.candidates,
            second.stale_days
        ),
        (Some(false), Some(0), Some(0), Some(7))
    );
    assert!(meta.scanned_at.is_some() && meta.error.is_none());
}

#[test]
fn a_project_with_no_rule_is_not_scanned_no_git_empty_docs() {
    let fx = Fixture::new();
    let h = Harness::new(&fx.root, Vec::new(), |_| {});
    h.scan();
    assert_eq!(h.svc.deps.git.stats().started, 0);
    assert!(h.docs().is_empty());
    let meta = h.meta();
    assert_eq!((meta.count, meta.rules.len(), meta.error), (0, 0, None));
}

#[test]
fn the_default_stale_days_are_configurable() {
    let fx = Fixture::new();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |o| {
        (o.scan_ms, o.stale_days) = (90_000, 30);
    });
    h.scan();
    assert_eq!(h.meta().scan_ms, 90_000);
    let d = h.docs();
    assert_eq!(
        (d["old-unmerged"].status, d["old-unmerged"].stale_days),
        (Status::Active, 30),
        "10 days is inside a 30 day default"
    );
}

fn request(id: &str, kind: &str, at: i64, text: &str, wt_id: &str, message: &str) -> OwnerEntry {
    OwnerEntry {
        id: id.into(),
        author: "human".into(),
        kind: "followup".into(),
        created_ms: Some(at),
        text: text.into(),
        worktree: Some(EntryWorktree {
            id: wt_id.into(),
            path: "p".into(),
            kind: kind.into(),
            message: Some(message.into()),
        }),
        ..OwnerEntry::default()
    }
}

fn plain_note(id: &str, at: i64) -> OwnerEntry {
    OwnerEntry {
        id: id.into(),
        author: "human".into(),
        kind: "followup".into(),
        created_ms: Some(at),
        text: "a plain note".into(),
        ..OwnerEntry::default()
    }
}

fn with_requests(fx: &Fixture) -> (Harness, String) {
    let h = Harness::new(&fx.root, vec![rule_for(fx)], |_| {});
    let id = h.id_of(&fx.path("old-unmerged"));
    let old = request(
        "O-1",
        "delete",
        1000,
        "[worktree delete] p: old",
        &id,
        "old one",
    );
    let newest = request(
        "O-2",
        "check",
        2000,
        "[worktree check] p: still needed?",
        &id,
        "still needed?",
    );
    for e in [old, newest, plain_note("O-3", 3000)] {
        h.board().entries.insert(e.id.clone(), e);
    }
    h.scan();
    (h, id)
}

#[test]
fn a_request_is_the_newest_owner_entry_about_the_worktree() {
    let fx = Fixture::new();
    let (h, _) = with_requests(&fx);
    let asked = h.docs()["old-unmerged"].request.clone().unwrap();
    assert_eq!((asked.id.as_str(), asked.kind.as_str()), ("O-2", "check"));
    assert_eq!(
        (asked.text.as_str(), asked.by.as_str()),
        ("still needed?", "owner")
    );
    assert!(asked.answer.is_none());
    assert!(h.docs()["fresh"].request.is_none());
}

#[test]
fn the_threads_answer_and_a_keep_show_on_the_row_without_git() {
    let fx = Fixture::new();
    let (h, id) = with_requests(&fx);
    let before = h.svc.deps.git.stats().started;
    let answer = Answer {
        note: "kept for the login work".into(),
        at_ms: 4000,
        by: Some("worker".into()),
    };
    h.board().answers.insert("O-2".into(), answer);
    let keep = Keep {
        until_ms: h.clock_now() + 3 * DAY_MS,
        note: Some("login work".into()),
        by: Some("worker".into()),
    };
    h.board().keeps.insert(id, keep);
    assert_eq!(h.svc.deps.git.stats().started, before, "no git ran");
    let row = &h.docs()["old-unmerged"];
    assert_eq!(
        row.request.as_ref().unwrap().answer.as_ref().unwrap().note,
        "kept for the login work"
    );
    assert_eq!(row.status, Status::Kept);
    let kept = row.kept.as_ref().unwrap();
    assert_eq!(
        (kept.note.as_deref(), kept.by.as_deref()),
        (Some("login work"), Some("worker"))
    );
    assert_eq!(h.meta().by_status.get("kept"), Some(&1));
}

#[test]
fn a_removed_rule_drops_its_rows_at_once_and_a_changed_rule_asks_for_a_scan() {
    let fx = Fixture::new();
    let h = started(&fx, |_| {});
    assert_eq!(h.docs().len(), 5);
    assert!(
        !h.svc.rules_changed("alpha"),
        "the scan knows the rules it read"
    );
    h.board().rules = Vec::new();
    assert!(
        h.docs().is_empty(),
        "the rows leave with their rule, no scan needed"
    );
    assert_eq!(h.meta().rules.len(), 0);
    assert!(
        h.svc.rules_changed("alpha") && !h.svc.rules_changed("alpha"),
        "a change is told once"
    );
    h.board().rules = vec![Rule::folder("W-3", &fx.wts, "worker")];
    assert!(h.svc.rules_changed("alpha"));
    let before = h.svc.deps.git.stats().started;
    h.scan();
    assert_eq!(h.docs().len(), 5, "the new rule was scanned");
    assert!(h.svc.deps.git.stats().started > before);
    assert_eq!(h.docs()["fresh"].rule, "W-3");
}

fn sent(h: &Harness) -> serde_json::Map<String, Value> {
    let text = h
        .store
        .kv
        .lock()
        .unwrap()
        .get(&sent_key("alpha"))
        .cloned()
        .unwrap_or_default();
    serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

#[test]
fn one_wake_goes_to_the_rules_thread_listing_what_needs_a_decision_worst_first() {
    let fx = Fixture::new();
    let h = started(&fx, |_| {});
    let wakes = h.notes.wakes.lock().unwrap().clone();
    assert_eq!(wakes.len(), 1);
    let w = &wakes[0];
    assert_eq!(
        (
            w.scope.as_str(),
            w.thread.as_str(),
            w.reason,
            w.subject.as_str()
        ),
        ("alpha", "worker", "worktree-stale", "worker")
    );
    assert!(w.text.chars().count() <= 300);
    assert!(w.text.starts_with("3 worktrees need a decision: "));
    assert!(
        w.text
            .contains("proj-worktrees/dirty-old stale 15d with 1 uncommitted (branch dirty-old)"),
        "{}",
        w.text
    );
    for left_out in ["fresh", "plain", "gone", "missing"] {
        assert!(!w.text.contains(left_out), "{left_out}");
    }
    assert!(w.text.ends_with(TAIL));
    let lines = h
        .notes
        .lines
        .lock()
        .unwrap()
        .iter()
        .filter(|l| l.kind == "worktree-stale")
        .count();
    assert_eq!(lines, 1);
}

#[test]
fn the_same_state_is_not_told_again_before_repeat_ms_then_once_and_the_state_is_saved() {
    let fx = Fixture::new();
    let h = started(&fx, |o| o.repeat_ms = 24 * HOUR_MS);
    assert_eq!(h.notes.wake_count(), 1);
    h.scan();
    h.clock.advance(23 * HOUR_MS);
    h.scan();
    assert_eq!(h.notes.wake_count(), 1, "inside 24 hours: nothing");
    h.clock.advance(2 * HOUR_MS);
    h.scan();
    assert_eq!(h.notes.wake_count(), 2, "after 24 hours: again");
    assert_eq!(h.notes.wakes.lock().unwrap()[1].thread, "worker");
    let saved = sent(&h);
    let mut keys: Vec<&String> = saved.keys().collect();
    keys.sort();
    let mut want: Vec<String> = ["dirty-old", "old-merged", "old-unmerged"]
        .iter()
        .map(|n| h.id_of(&fx.path(n)))
        .collect();
    want.sort();
    assert_eq!(keys, want.iter().collect::<Vec<_>>());
    assert_eq!(saved[&h.id_of(&fx.path("old-merged"))]["s"], "merged");
}

#[test]
fn a_restart_on_the_same_database_does_not_tell_the_thread_again() {
    let fx = Fixture::new();
    let first = started(&fx, |_| {});
    assert_eq!(first.notes.wake_count(), 1);
    first.svc.stop();
    let again = Harness::on_store(first.store.clone(), &fx.root, RemoveMode::Real, |_| {});
    again.scan();
    assert_eq!(again.notes.wake_count(), 0, "remembered");
}

#[test]
fn a_status_change_is_told_alone_a_keep_silences_it_and_it_is_told_again_when_the_keep_ends() {
    let fx = Fixture::new();
    let h = started(&fx, |o| o.repeat_ms = 400 * 24 * HOUR_MS);
    h.notes.wakes.lock().unwrap().clear();
    fs_write(&fx.path("old-unmerged"), "scratch.txt");
    h.scan();
    let texts = h.wake_texts();
    assert_eq!(texts.len(), 1);
    assert!(
        texts[0].starts_with("1 worktree needs a decision: ")
            && texts[0].contains("old-unmerged stale 10d with 1 uncommitted"),
        "{}",
        texts[0]
    );
    let id = h.id_of(&fx.path("old-unmerged"));
    h.board().keeps.insert(
        id.clone(),
        Keep {
            until_ms: h.clock_now() + 7 * DAY_MS,
            note: Some("login".into()),
            by: Some("worker".into()),
        },
    );
    h.notes.wakes.lock().unwrap().clear();
    h.clock.advance(30 * HOUR_MS);
    h.scan();
    assert_eq!(h.docs()["old-unmerged"].status, Status::Kept);
    assert!(
        !h.wake_texts().iter().any(|t| t.contains("old-unmerged")),
        "a kept worktree is never in the signal"
    );
    assert!(
        !sent(&h).contains_key(&id),
        "and it is forgotten, so it is told again later"
    );
    h.clock.advance(8 * DAY_MS);
    h.notes.wakes.lock().unwrap().clear();
    h.scan();
    assert_eq!(h.docs()["old-unmerged"].status, Status::DirtyStale);
    assert!(h.wake_texts().len() == 1 && h.wake_texts()[0].contains("old-unmerged"));
}

fn fs_write(dir: &str, name: &str) {
    crate::testkit::put(dir, name, "x");
}

#[test]
fn an_ended_thread_is_replaced_by_the_lead_and_with_no_lead_nobody_is_told() {
    let fx = Fixture::new();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.board().threads.get_mut("worker").unwrap().live = false;
    h.scan();
    let wakes = h.notes.wakes.lock().unwrap().clone();
    assert_eq!((wakes.len(), wakes[0].thread.as_str()), (1, "lead"));
    let none = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    for id in ["worker", "lead"] {
        none.board().threads.get_mut(id).unwrap().live = false;
    }
    none.scan();
    assert_eq!(none.notes.wake_count(), 0);
    assert!(sent(&none).is_empty());
}

#[test]
fn a_path_two_rules_reach_is_listed_once_and_a_refused_wake_is_tried_again_at_the_next_scan() {
    let fx = Fixture::new();
    let rules = vec![rule_for(&fx), Rule::folder("W-2", &fx.repo, "lead").repo()];
    let h = Harness::with_mode(&fx.root, board_with(rules), RemoveMode::Real, |_| {});
    h.scan();
    let threads: Vec<String> = h
        .notes
        .wakes
        .lock()
        .unwrap()
        .iter()
        .map(|w| w.thread.clone())
        .collect();
    assert_eq!(
        threads,
        ["worker"],
        "W-1 holds the five folders (first rule wins); W-2 sees the same worktrees: listed once"
    );
    let t = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    t.notes
        .refuse_wakes
        .store(true, std::sync::atomic::Ordering::SeqCst);
    t.scan();
    assert_eq!(t.notes.wake_count(), 0);
    t.notes
        .refuse_wakes
        .store(false, std::sync::atomic::Ordering::SeqCst);
    t.scan();
    assert_eq!(t.notes.wake_count(), 1, "tried again");
}

#[test]
fn a_stopped_service_scans_nothing() {
    let fx = Fixture::new();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.svc.stop();
    assert!(h.svc.scan_now("alpha").is_none());
    assert_eq!(h.svc.deps.git.stats().started, 0);
    h.svc.start();
    assert!(h.svc.scan_now("alpha").is_some());
}

#[test]
fn a_rule_whose_scan_fails_says_so_once_for_the_servers_log() {
    let fx = Fixture::new();
    let plain = format!("{}/plain", fx.wts);
    let h = Harness::new(
        &fx.root,
        vec![rule_for(&fx), Rule::folder("W-2", &plain, "worker").repo()],
        |_| {},
    );
    h.scan();
    h.scan();
    let problems = h.notes.problems.lock().unwrap().clone();
    assert_eq!(
        problems,
        ["alpha: the worktree scan of rule W-2 failed: not a git repository"],
        "one line, not one per scan"
    );
}
