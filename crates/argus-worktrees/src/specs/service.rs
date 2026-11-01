//! KEEP: K2.
//! The service: statuses, the docs and the summary it mirrors, the owner's requests, keeps, and the stale signal.

// test-tag: e2e

use crate::board::{Answer, EntryWorktree, Keep, OwnerEntry};
use crate::clock::{DAY_MS, HOUR_MS};
use crate::doc::Status;
use crate::paths::worktree_id;
use crate::rule::Rule;
use crate::service::Options;
use crate::testkit::{Fixture, Harness, RemoveMode, board_with, rule_for};
use crate::waking::sent_key;
use serde_json::Value;

const TAIL: &str =
    "Keep it (coord worktree keep), remove it (coord worktree remove), or drop its rule.";

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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
#[ignore = "e2e"]
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
