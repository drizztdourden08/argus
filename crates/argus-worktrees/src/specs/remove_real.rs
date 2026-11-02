//! KEEP: K3, K4.
//! The removal journeys on the real git, one smoke each: a merged worktree goes, a dirty one is refused, and the bundle
//! a discard wrote gives the commit back to a real `git fetch`, and a git that times out hands the folder to Argus. Everything else about removing is in `remove.rs` on the
//! fake git; these prove the fake and git agree where it matters.

// test-tag: e2e

use super::fallback::shape_on;
use super::remove::discard_by_decision_on;
use crate::error::ErrorCode;
use crate::testkit::{Fixture, RemoveMode, branches, exists_itself, git_in, listed, scanned_real};

#[test]
#[ignore = "e2e"]
fn a_merged_clean_worktree_goes_with_its_branch_and_git_no_longer_lists_it() {
    let (fx, h) = scanned_real();
    let out = h
        .rm_with(&h.id_of(&fx.path("old-merged")), |i| i.branch = true)
        .unwrap();
    assert!(out.removed && out.deleted_branch, "{}", out.text);
    assert!(out.backup.is_none());
    assert!(!exists_itself(&fx.path("old-merged")));
    assert!(!listed(&fx.repo).contains(&"old-merged".to_string()));
    assert!(!branches(&fx.repo).contains(&"old-merged".to_string()));
}

#[test]
#[ignore = "e2e"]
fn a_dirty_worktree_is_refused_without_discard_and_stays() {
    let (fx, h) = scanned_real();
    let err = h.refused(&h.id_of(&fx.path("dirty-old")), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .contains("has 1 uncommitted change: removing it discards them"),
        "{}",
        err.message
    );
    assert!(exists_itself(&fx.path("dirty-old")));
    assert!(listed(&fx.repo).contains(&"dirty-old".to_string()));
}

#[test]
#[ignore = "e2e"]
fn the_bundle_gives_the_commit_back_to_a_real_fetch() {
    let (fx, _h, out, _head) = discard_by_decision_on(Fixture::new());
    let b = out.backup.unwrap();
    git_in(
        &fx.repo,
        &[
            "fetch",
            "-q",
            b.bundle.as_deref().unwrap(),
            "old-unmerged:old-unmerged",
        ],
        &[],
    );
    let subject = git_in(&fx.repo, &["log", "-1", "--format=%s", "old-unmerged"], &[]);
    assert_eq!(
        subject.trim(),
        "add old.txt",
        "the bundle gives the commit back"
    );
}

#[test]
#[ignore = "e2e"]
fn a_git_that_times_out_after_deleting_the_files_hands_the_folder_to_argus_and_no_link_target_is_lost()
 {
    let (s, out) = shape_on(Fixture::new(), RemoveMode::Timeout, |_| {}).remove_old_merged();
    assert!(out.removed && out.finished_by_argus, "{}", out.text);
    s.h.idle();
    assert!(!exists_itself(&s.fx.path("old-merged")));
    assert!(!listed(&s.fx.repo).contains(&"old-merged".to_string()));
    s.keeps_survive();
}
