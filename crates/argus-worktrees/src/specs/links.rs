//! Delete safety with real links: every delete path of the crate meets a directory junction (`mklink /J`) and, where the
//! machine allows one, a file symlink, both pointing at a SIBLING folder full of files. The sibling must come out
//! untouched and the link itself must be gone; a folder to remove that is itself a junction never reaches its target.

use crate::fs::{Fs, RealFs};
use crate::paths::join_path;
use crate::testkit::{
    Fixture, Harness, RemoveMode, backdate, board_with, exists_itself, make_link, put, root_of,
    rule_for,
};
use crate::tree::{drop_links, remove_entry, walk_remove};
use argus_core::fixture::links::{assert_sibling_intact, file_symlink, fill_sibling};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// A sibling folder full of files, beside `dir`'s other content.
fn sibling(root: &str, name: &str) -> String {
    let path = join_path(root, name);
    fill_sibling(Path::new(&path));
    path
}

/// A fresh temp folder with a sibling folder full of files and the path of a `victim` to build beside it.
fn arena(victim: &str) -> (tempfile::TempDir, String, String) {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    let side = sibling(&root, "sibling");
    let victim = join_path(&root, victim);
    (dir, side, victim)
}

fn intact(dir: &str) {
    assert_sibling_intact(Path::new(dir));
}

/// A file symlink at `link` to the sibling's `keep.txt`: false (and a skip line) when the machine does not allow one.
fn file_link(link: &str, sibling: &str) -> bool {
    file_symlink(Path::new(link), Path::new(&join_path(sibling, "keep.txt")))
}

/// `victim` with files, a deep junction and a top junction to `sibling`, and a file symlink when allowed. Returns the
/// links made.
fn links_inside(victim: &str, sibling: &str) -> Vec<String> {
    put(victim, "own.txt", "mine");
    put(victim, "node_modules/.pnpm/pkg@1/index.js", "x");
    let mut links = vec![
        join_path(victim, "node_modules/ext"),
        join_path(victim, "lnk-top"),
    ];
    for link in &links {
        make_link(link, sibling);
    }
    let file = join_path(victim, "lnk-file.txt");
    if file_link(&file, sibling) {
        links.push(file);
    }
    links
}

fn links_gone(links: &[String]) {
    for link in links {
        assert!(!exists_itself(link), "the link {link} itself is gone");
    }
}

type Delete = fn(&str) -> std::io::Result<()>;

/// Every primitive, by name: the native delete of `RealFs`, the walk, and the top-level entry with and without the
/// pre-pass that drops the links first.
fn primitives() -> [(&'static str, Delete); 4] {
    [
        ("RealFs::remove_tree", |p| RealFs.remove_tree(p)),
        ("walk_remove", |p| walk_remove(&RealFs, p)),
        ("remove_entry with the pre-pass", |p| {
            remove_entry(&RealFs, p, true)
        }),
        ("remove_entry without the pre-pass", |p| {
            remove_entry(&RealFs, p, false)
        }),
    ]
}

#[test]
fn every_primitive_deletes_a_folder_and_unlinks_the_links_inside_never_their_targets() {
    for (name, delete) in primitives() {
        let (_dir, side, victim) = arena("victim");
        let links = links_inside(&victim, &side);
        delete(&victim).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!exists_itself(&victim), "{name}: the folder is gone");
        links_gone(&links);
        intact(&side);
    }
}

#[test]
fn no_primitive_reaches_the_target_of_a_folder_that_is_itself_a_junction() {
    let mut reached = Vec::new();
    for (name, delete) in primitives() {
        let (_dir, side, victim) = arena("victim");
        make_link(&victim, &side);
        delete(&victim).unwrap_or_else(|e| panic!("{name}: {e}"));
        if !exists_itself(&join_path(&side, "keep.txt")) {
            reached.push(name);
        }
        assert!(!exists_itself(&victim), "{name}: the junction itself went");
    }
    assert!(
        reached.is_empty(),
        "these followed the junction: {reached:?}"
    );
}

#[test]
fn the_pre_pass_unlinks_a_file_symlink_and_a_junction_and_keeps_every_real_file() {
    let (_dir, side, victim) = arena("wt");
    let links = links_inside(&victim, &side);
    drop_links(&RealFs, &victim).unwrap();
    links_gone(&links);
    assert!(exists_itself(&join_path(&victim, "own.txt")));
    assert!(exists_itself(&join_path(
        &victim,
        "node_modules/.pnpm/pkg@1/index.js"
    )));
    intact(&side);
}

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

#[test]
fn the_backup_prune_unlinks_a_link_among_the_backups_and_never_follows_it() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    let side = sibling(&root, "sibling");
    let h = Harness::new(&root, Vec::new(), |o| {
        (o.keep_backups, o.keep_backup_days) = (0, 0);
    });
    let backups = h.backups.clone();
    let mut links = vec![join_path(&backups, "alpha-old-20260101.untracked")];
    make_link(&links[0], &side);
    let file = join_path(&backups, "alpha-old-20260101.patch");
    if file_link(&file, &side) {
        links.push(file);
    }
    put(&backups, "alpha-old-20260101.bundle", "bundle");
    h.clock.advance(crate::clock::DAY_MS);
    h.svc.prune_backups();
    intact(&side);
    links_gone(&links);
    assert!(!exists_itself(&join_path(
        &backups,
        "alpha-old-20260101.bundle"
    )));
}
