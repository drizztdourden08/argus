//! KEEP: K1.
//! The event-driven scan: a cheap look at file times decides whether a scan runs, and a scan reads with git only the
//! worktrees whose fingerprint changed. A forced scan and the safety period read everything.

// test-tag: e2e

use crate::probe::SAFETY_SCAN_MS;
use crate::sync::locked;
use crate::testkit::{Harness, days_ago, index_of, put, scanned_real, set_mtime};
use std::path::Path;

const READ_WORKTREES: usize = 4;

fn git_calls(h: &Harness) -> usize {
    locked(&h.exec.calls).len()
}

/// How many worktrees git read (`git status`) since `since` calls.
fn statuses_since(h: &Harness, since: usize) -> usize {
    locked(&h.exec.calls)[since..]
        .iter()
        .filter(|args| args.iter().any(|a| a == "status"))
        .count()
}

/// A changed disk is seen, and the scan after it reads exactly one worktree with git.
fn reads_one_changed(h: &Harness, said: &str) {
    assert!(h.svc.changed("alpha"));
    let before = git_calls(h);
    assert!(h.svc.scan_if_changed("alpha").is_some());
    assert_eq!(statuses_since(h, before), 1, "{said}");
}

#[test]
#[ignore = "e2e"]
fn an_unchanged_disk_means_no_scan_and_no_git_process() {
    let (_fx, h) = scanned_real();
    let before = git_calls(&h);
    assert!(!h.svc.changed("alpha"));
    assert!(h.svc.scan_if_changed("alpha").is_none());
    assert_eq!(git_calls(&h), before, "the look runs no git");
}

#[test]
#[ignore = "e2e"]
fn a_new_index_time_scans_that_worktree_only_and_the_others_keep_their_facts() {
    let (fx, h) = scanned_real();
    let rows = h.docs().len();
    set_mtime(Path::new(&index_of(&fx.path("fresh"))), days_ago(0.5));
    reads_one_changed(&h, "only the changed worktree");
    assert_eq!(h.docs().len(), rows, "the others keep their rows");
    assert!(
        !h.svc.changed("alpha"),
        "the new fingerprint is the one kept"
    );
}

#[test]
#[ignore = "e2e"]
fn a_new_head_time_scans_that_worktree() {
    let (fx, h) = scanned_real();
    let index = index_of(&fx.path("old-merged"));
    set_mtime(&Path::new(&index).with_file_name("HEAD"), days_ago(1.0));
    reads_one_changed(&h, "the one that changed");
}

#[test]
#[ignore = "e2e"]
fn a_worktree_that_comes_is_read_and_the_others_are_not() {
    let (fx, h) = scanned_real();
    fx.add("brand-new");
    reads_one_changed(&h, "the one that changed");
    assert_eq!(h.fact("brand-new").branch.as_deref(), Some("brand-new"));
}

#[test]
#[ignore = "e2e"]
fn a_worktree_that_goes_is_noticed_without_git() {
    let (fx, h) = scanned_real();
    std::fs::remove_dir_all(fx.path("old-merged")).unwrap();
    let before = git_calls(&h);
    assert!(h.svc.changed("alpha"));
    assert_eq!(git_calls(&h), before, "the look runs no git");
}

#[test]
#[ignore = "e2e"]
fn the_safety_period_reads_everything_and_finds_what_no_fingerprint_shows() {
    let (fx, h) = scanned_real();
    put(&fx.path("fresh"), "wip.txt", "work\n");
    assert!(
        !h.svc.changed("alpha"),
        "an untracked file changes none of the files the look reads"
    );
    assert_eq!(h.fact("fresh").dirty, 0, "the row is the last scan's");
    h.clock.advance(SAFETY_SCAN_MS);
    assert!(h.svc.changed("alpha"));
    let before = git_calls(&h);
    assert!(h.svc.scan_if_changed("alpha").is_some());
    assert_eq!(statuses_since(&h, before), READ_WORKTREES);
    assert_eq!(h.fact("fresh").dirty, 1);
    assert!(!h.svc.changed("alpha"), "the period starts again");
}

#[test]
#[ignore = "e2e"]
fn a_forced_scan_reads_everything_whatever_the_fingerprints_say() {
    let (_fx, h) = scanned_real();
    assert!(!h.svc.changed("alpha"));
    let before = git_calls(&h);
    h.scan();
    assert_eq!(statuses_since(&h, before), READ_WORKTREES);
}
