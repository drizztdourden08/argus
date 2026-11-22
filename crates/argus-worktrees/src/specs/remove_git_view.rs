//! KEEP: K3, K4.
//! I-177: a worktree `git worktree remove` would refuse without `--force` is refused by Argus before it touches anything
//! (the ignored `node_modules`, the links, a Brock lock), with the usual "uncommitted changes" refusal. That covers the
//! untracked files and the tracked paths a link holds: once Argus drops the links, those read as deleted to git. With
//! the owner's say the removal goes as before, forced. When git still fails after the prep steps, the refusal says what
//! Argus had already done to the folder.

// test-tag: standard

use super::fallback::{Shape, shape};
use crate::error::ErrorCode;
use crate::testkit::{
    Fixture, RemoveMode, commit, exists_itself, make_link, owner_now, put, root_of,
};
use std::fs;
use tempfile::{TempDir, tempdir};

const REFUSED: &str = "has 1 uncommitted change: removing it discards them";

fn pnpm_file(s: &Shape) -> String {
    format!(
        "{}/node_modules/.pnpm/pkg@1/node_modules/pkg/index.js",
        s.fx.path("old-merged")
    )
}

/// Did the removal send git this command (its first words)?
fn asked(s: &Shape, command: &[&str]) -> bool {
    s.h.exec
        .calls
        .lock()
        .unwrap()
        .iter()
        .any(|a| a.len() >= command.len() && a.iter().zip(command).all(|(x, y)| x == y))
}

/// Nothing was touched: `node_modules` and every link are still there, and git was never asked to check or remove.
fn refused_and_untouched(s: &Shape) {
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::Forbidden);
    assert!(err.message.contains(REFUSED), "{}", err.message);
    assert_untouched(s);
}

fn assert_untouched(s: &Shape) {
    let merged = s.fx.path("old-merged");
    assert!(exists_itself(&pnpm_file(s)), "node_modules is still there");
    for link in ["node_modules/pkg", "node_modules/ext", "lnk-top"] {
        assert!(
            exists_itself(&format!("{merged}/{link}")),
            "{link} was not dropped"
        );
    }
    assert!(
        !asked(s, &["check-ignore"]),
        "node_modules was not looked at"
    );
    assert!(
        !asked(s, &["worktree", "remove"]),
        "git was never asked to remove it"
    );
    assert!(!asked(s, &["worktree", "unlock"]));
}

fn untracked(fx: &Fixture) {
    put(&fx.path("old-merged"), "notes.txt", "not committed\n");
}

/// `old-merged` commits `pkg/a.txt`, then its `pkg` folder is swapped for a junction to a copy outside: git reads the
/// file through the junction, so the worktree looks clean until the link goes (R-10 on the `RotP` Web board).
fn tracked_behind_a_link(outside: &str) -> impl FnOnce(&Fixture) + '_ {
    move |fx| {
        let merged = fx.path("old-merged");
        commit(&merged, "pkg/a.txt", "tracked\n", 15.0);
        fs::remove_dir_all(format!("{merged}/pkg")).unwrap();
        put(outside, "a.txt", "tracked\n");
        make_link(&format!("{merged}/pkg"), outside);
    }
}

fn linked_shape() -> (Shape, TempDir, String) {
    let dir = tempdir().unwrap();
    let outside = root_of(&dir);
    let s = shape(RemoveMode::Real, tracked_behind_a_link(&outside));
    (s, dir, outside)
}

/// The owner's note naming `old-merged`, and the removal with discard and that entry.
fn discard_with_the_owners_say(s: &Shape) -> crate::request::RemoveOutcome {
    let entry = owner_now("O-n", "you can remove **old-merged** now");
    s.h.board().entries.insert(entry.id.clone(), entry);
    s.h.rm_with(&s.id("old-merged"), |i| {
        (i.discard, i.owner_entry) = (true, Some("O-n".into()));
    })
    .unwrap()
}

#[test]
fn only_untracked_files_and_no_discard_is_refused_before_node_modules_or_a_link_is_touched() {
    let s = shape(RemoveMode::Real, untracked);
    refused_and_untouched(&s);
    assert!(exists_itself(&format!(
        "{}/notes.txt",
        s.fx.path("old-merged")
    )));
}

#[test]
fn only_untracked_files_with_discard_and_the_owners_say_goes_as_before_forced() {
    let s = shape(RemoveMode::Real, untracked);
    let out = discard_with_the_owners_say(&s);
    assert!(out.removed, "{}", out.text);
    assert_eq!(out.discarded.as_ref().map(|d| d.dirty), Some(1));
    assert!(out.backup.is_some(), "the backup is written first");
    assert!(asked(&s, &["worktree", "remove", "--force"]));
    assert!(!exists_itself(&s.fx.path("old-merged")));
    s.h.idle();
    s.keeps_survive();
}

#[test]
fn a_tracked_folder_behind_a_link_looks_clean_but_is_refused_before_anything_is_touched() {
    let (s, _dir, outside) = linked_shape();
    assert_eq!(
        s.h.docs()["old-merged"].dirty,
        0,
        "git reads the tracked file through the junction"
    );
    refused_and_untouched(&s);
    assert!(exists_itself(&format!("{}/pkg", s.fx.path("old-merged"))));
    assert!(exists_itself(&format!("{outside}/a.txt")));
}

#[test]
fn a_tracked_folder_behind_a_link_with_the_owners_say_is_forced_and_the_target_survives() {
    let (s, _dir, outside) = linked_shape();
    let out = discard_with_the_owners_say(&s);
    assert!(out.removed, "{}", out.text);
    assert_eq!(out.discarded.as_ref().map(|d| d.dirty), Some(1));
    assert!(asked(&s, &["worktree", "remove", "--force"]));
    assert!(!exists_itself(&s.fx.path("old-merged")));
    assert_eq!(
        fs::read_to_string(format!("{outside}/a.txt")).unwrap(),
        "tracked\n",
        "the link went as a link"
    );
    s.h.idle();
    s.keeps_survive();
}

#[test]
fn git_failing_after_the_prep_steps_says_what_argus_had_already_done() {
    let s = shape(RemoveMode::Untouched, |_| {});
    let err = s.h.refused(&s.id("old-merged"), |_| {});
    assert_eq!(err.code, ErrorCode::GitFailed);
    assert!(
        err.message.starts_with("git worktree remove failed: ")
            && err.message.contains(
                "; before git ran, Argus had cleared the ignored node_modules and dropped 1 link inside the worktree (as links: what they point to is untouched): reinstall the dependencies before working in the worktree again"
            ),
        "{}",
        err.message
    );
    assert!(!exists_itself(&pnpm_file(&s)));
    s.keeps_survive();
}
