//! KEEP: K3.
//! Removing a worktree that holds submodules, on the fake git. git refuses such a worktree without `--force`, so Argus
//! checks every initialized submodule itself and forces only when each is clean and on its recorded commit.

// test-tag: standard

use crate::board::Caller;
use crate::error::ErrorCode;
use crate::fakegit::SubState;
use crate::testkit::{Fixture, Harness, exists_itself, listed, rule_for};

/// A started harness over the fake fixture, with the submodules `prep` gives `old-merged`.
fn with_submodules(prep: impl FnOnce(&Fixture)) -> (Fixture, Harness, String) {
    let fx = Fixture::fake();
    prep(&fx);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    let id = h.id_of(&fx.path("old-merged"));
    (fx, h, id)
}

fn still_there(fx: &Fixture) {
    assert!(exists_itself(&fx.path("old-merged")), "nothing was removed");
    assert!(listed(&fx.repo).contains(&"old-merged".to_string()));
}

#[test]
fn a_clean_merged_worktree_with_clean_initialized_submodules_is_removed() {
    let (fx, h, id) = with_submodules(|fx| {
        fx.submodule("old-merged", "libultraship", SubState::Clean);
        fx.submodule("old-merged", "torch", SubState::Clean);
    });
    assert_eq!(
        h.docs()["old-merged"].dirty,
        0,
        "a submodule on its commit is not a change of the superproject"
    );
    let out = h.rm(&id).unwrap();
    assert!(out.removed, "{}", out.text);
    assert!(
        out.backup.is_none(),
        "a submodule's .git is not a nested repository"
    );
    assert!(!exists_itself(&fx.path("old-merged")), "the folder is gone");
    assert!(!listed(&fx.repo).contains(&"old-merged".to_string()));
}

#[test]
fn the_fake_git_refuses_a_worktree_with_submodules_without_force_as_git_does() {
    let (fx, _h, _id) = with_submodules(|fx| {
        fx.submodule("old-merged", "libultraship", SubState::Clean);
    });
    let door = fx.door();
    let plain = door.write(&fx.repo, &["worktree", "remove", &fx.path("old-merged")]);
    assert!(
        plain
            .stderr
            .contains("working trees containing submodules cannot be moved or removed"),
        "{}",
        plain.stderr
    );
    assert!(exists_itself(&fx.path("old-merged")));
}

#[test]
fn a_submodule_moved_off_its_commit_is_refused_by_name_and_the_backup_stays() {
    let (fx, h, id) = with_submodules(|fx| {
        fx.submodule("old-merged", "libultraship", SubState::Moved);
        fx.submodule("old-merged", "torch", SubState::Clean);
    });
    let plain = h.refused(&id, |_| {});
    assert!(
        plain.message.contains("1 uncommitted change"),
        "git sees the moved submodule as a change: {}",
        plain.message
    );
    let err = h.refused(&id, |i| (i.discard, i.caller) = (true, Caller::Page));
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message
            .contains("libultraship moved off its recorded commit"),
        "{}",
        err.message
    );
    assert!(!err.message.contains("torch"), "{}", err.message);
    assert!(err.message.contains("nothing was removed"));
    assert!(err.backup.is_some(), "the backup written before stays");
    still_there(&fx);
}

#[test]
fn a_submodule_with_a_conflict_is_refused_by_name() {
    let (fx, h, id) = with_submodules(|fx| {
        fx.submodule("old-merged", "torch", SubState::Conflict);
    });
    let err = h.refused(&id, |i| (i.discard, i.caller) = (true, Caller::Page));
    assert!(
        err.message.contains("torch has a merge conflict"),
        "{}",
        err.message
    );
    still_there(&fx);
}

#[test]
fn a_submodule_with_uncommitted_changes_is_refused_by_name_even_when_the_superproject_looks_clean()
{
    let (fx, h, id) = with_submodules(|fx| {
        fx.submodule("old-merged", "libultraship", SubState::Clean);
        fx.submodule("old-merged", "torch", SubState::Edited);
    });
    assert_eq!(
        h.docs()["old-merged"].dirty,
        0,
        "the superproject says nothing"
    );
    let err = h.refused(&id, |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(
        err.message.contains("torch has 1 uncommitted change"),
        "{}",
        err.message
    );
    assert!(!err.message.contains("libultraship"), "{}", err.message);
    assert!(err.message.contains("nothing was removed"));
    assert!(err.backup.is_none(), "a safe worktree wrote no backup");
    still_there(&fx);
}

#[test]
fn a_submodule_with_changes_the_superproject_shows_is_refused_by_name_after_the_discard() {
    let (fx, h, id) = with_submodules(|fx| {
        fx.submodule("old-merged", "torch", SubState::Changed);
    });
    let err = h.refused(&id, |i| (i.discard, i.caller) = (true, Caller::Page));
    assert!(
        err.message.contains("torch has 1 uncommitted change"),
        "{}",
        err.message
    );
    assert!(err.backup.is_some());
    still_there(&fx);
}

#[test]
fn an_uninitialized_submodule_does_not_block_the_removal() {
    let (fx, h, id) = with_submodules(|fx| {
        fx.submodule("old-merged", "libultraship", SubState::Uninitialized);
    });
    let out = h.rm(&id).unwrap();
    assert!(out.removed, "{}", out.text);
    assert!(out.backup.is_none());
    assert!(!exists_itself(&fx.path("old-merged")));
}
