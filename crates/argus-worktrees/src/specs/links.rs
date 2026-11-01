//! KEEP: K3 (a removal through git never follows a link).
//! Delete safety with real links: every delete path of the crate meets a directory junction (`mklink /J`) and, where the
//! machine allows one, a file symlink, both pointing at a SIBLING folder full of files. The sibling must come out
//! untouched and the link itself must be gone; a folder to remove that is itself a junction never reaches its target.

// test-tag: e2e

use super::links_fs::{file_link, intact, links_gone, links_inside, sibling};
use crate::paths::join_path;
use crate::testkit::{
    Fixture, Harness, RemoveMode, backdate, board_with, exists_itself, make_link, put, rule_for,
};
use std::fs;

/// A fixture whose `old-merged` worktree holds ignored links to two siblings (and a file symlink when allowed), and a
/// service that drops the links first, as on Windows.
fn worktree_with_links(mode: RemoveMode, modules_link: bool) -> (Fixture, Harness, [String; 2]) {
    let fx = Fixture::new();
    fs::write(
        format!("{}/.git/info/exclude", fx.repo),
        "node_modules\nlnk-*\n",
    )
    .unwrap();
    let sides = [sibling(&fx.wts, "sibling-a"), sibling(&fx.wts, "sibling-b")];
    let path = fx.path("old-merged");
    if modules_link {
        make_link(&join_path(&path, "node_modules"), &sides[0]);
    } else {
        put(&path, "node_modules/.pnpm/pkg@1/index.js", "x");
        make_link(&join_path(&path, "node_modules/ext"), &sides[0]);
    }
    make_link(&join_path(&path, "lnk-top"), &sides[1]);
    file_link(&join_path(&path, "lnk-file.txt"), &sides[1]);
    let board = board_with(vec![rule_for(&fx)]);
    let h = Harness::with_mode(&fx.root, board, mode, |o| o.drop_links_first = true);
    h.scan();
    (fx, h, sides)
}

#[test]
#[ignore = "e2e"]
fn a_worktree_removal_by_git_never_reaches_a_sibling_through_a_link() {
    for modules_link in [false, true] {
        let (fx, h, sides) = worktree_with_links(RemoveMode::Real, modules_link);
        let out = h.rm(&h.id_of(&fx.path("old-merged"))).unwrap();
        assert!(out.removed, "{}", out.text);
        h.idle();
        assert!(!exists_itself(&fx.path("old-merged")));
        for side in &sides {
            intact(side);
        }
    }
}

#[test]
#[ignore = "e2e"]
fn the_folder_fallback_job_unlinks_the_links_git_left_and_never_follows_them() {
    let (fx, h, sides) = worktree_with_links(RemoveMode::Stuck, false);
    let path = fx.path("old-merged");
    let (left, side) = (path.clone(), sides[1].clone());
    *h.exec.after_failure.lock().unwrap() = Some(Box::new(move || {
        make_link(&join_path(&left, "lnk-late"), &side);
        file_link(&join_path(&left, "lnk-late.txt"), &side);
    }));
    let out = h.rm(&h.id_of(&path)).unwrap();
    assert!(out.finished_by_argus, "{}", out.text);
    h.idle();
    assert!(
        !exists_itself(&out.folder.unwrap()),
        "the renamed folder is gone"
    );
    for side in &sides {
        intact(side);
    }
}

/// A leftover folder `junk` (git does not know it) made by `prep`, a sibling, and the service.
fn leftover(prep: impl FnOnce(&str, &str)) -> (Fixture, Harness, String) {
    let fx = Fixture::new();
    let side = sibling(&fx.root, "sibling");
    prep(&fx.path("junk"), &side);
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |o| o.drop_links_first = true);
    h.scan();
    (fx, h, side)
}

fn remove_leftover(h: &Harness, path: &str) -> crate::error::Result<crate::request::RemoveOutcome> {
    let newest = h.fact("junk").leftover.and_then(|l| l.newest_ms);
    h.rm_with(&h.id_of(path), |i| {
        (i.reason, i.newest_at_ms) = (Some("old agent folder".into()), newest);
    })
}

#[test]
#[ignore = "e2e"]
fn a_leftover_job_unlinks_the_links_inside_and_never_follows_them() {
    let mut links = Vec::new();
    let (fx, h, side) = leftover(|junk, side| {
        links = links_inside(junk, side);
        backdate(junk, 10.0);
    });
    let out = remove_leftover(&h, &fx.path("junk")).unwrap();
    h.idle();
    assert!(
        !exists_itself(&out.folder.unwrap()),
        "the renamed folder is gone"
    );
    links_gone(&links);
    intact(&side);
}

#[test]
#[ignore = "e2e"]
fn a_leftover_that_is_itself_a_junction_is_refused_and_its_target_untouched() {
    let (fx, h, side) = leftover(make_link);
    let err = remove_leftover(&h, &fx.path("junk")).unwrap_err();
    assert!(
        err.message.contains("is a link or junction"),
        "{}",
        err.message
    );
    h.idle();
    intact(&side);
}
