//! KEEP: K3 (a delete never follows a link), standard because a junction is made with `mklink /J` on Windows.
//!
//! Every delete path that needs no git meets a directory junction and, where the machine allows one, a file symlink,
//! both pointing at a SIBLING folder full of files. The sibling must come out untouched and the link itself must be
//! gone; a folder to remove that is itself a junction never reaches its target. The scan of a leftover folder reads a
//! link as a leaf.

// test-tag: standard

use crate::fact::Fact;
use crate::fs::{Fs, RealFs};
use crate::git::Git;
use crate::measure::{Bounds, measure_folder};
use crate::paths::{base_name, join_path};
use crate::rule::Rule;
use crate::scan::{ScanEnv, scan_rules};
use crate::testkit::{
    Harness, NotRepo, backdate, exists_itself, make_link, put, real_measure, root_of,
};
use crate::tree::{drop_links, remove_entry, walk_remove};
use argus_core::fixture::links::{assert_sibling_intact, file_symlink, fill_sibling};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;

/// A sibling folder full of files, beside `dir`'s other content.
pub(super) fn sibling(root: &str, name: &str) -> String {
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

pub(super) fn intact(dir: &str) {
    assert_sibling_intact(Path::new(dir));
}

/// A file symlink at `link` to the sibling's `keep.txt`: false (and a skip line) when the machine does not allow one.
pub(super) fn file_link(link: &str, sibling: &str) -> bool {
    file_symlink(Path::new(link), Path::new(&join_path(sibling, "keep.txt")))
}

/// `victim` with files, a deep junction and a top junction to `sibling`, and a file symlink when allowed. Returns the
/// links made.
pub(super) fn links_inside(victim: &str, sibling: &str) -> Vec<String> {
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

pub(super) fn links_gone(links: &[String]) {
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

fn facts_by_name(facts: &[Fact]) -> HashMap<String, &Fact> {
    facts.iter().map(|f| (base_name(&f.path), f)).collect()
}

fn scan_no_repo(wts: &str, ci: bool) -> crate::scan::ScanResult {
    let git = Git::new(Arc::new(NotRepo));
    let env = ScanEnv {
        git: &git,
        fs: &RealFs,
        case_insensitive: ci,
        measure: &real_measure,
        known_repos: &[],
    };
    scan_rules(&[Rule::folder("W-1", wts, "lead")], &env)
}

#[test]
fn with_no_repository_only_the_git_check_decides_a_link_is_a_leaf() {
    let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
    let (root, out_root) = (root_of(&dir), root_of(&outside));
    let wts = format!("{root}/proj/.claude/worktrees");
    put(&wts, "plain/a.txt", "x\n");
    put(&wts, "has-git-file/.git", "gitdir: X:/nowhere\n");
    fs::create_dir_all(format!("{wts}/has-git-dir/.git")).unwrap();
    put(&wts, "nested/inner/.git/HEAD", "x");
    put(&out_root, "keep.txt", "x");
    make_link(&format!("{wts}/link"), &out_root);
    put(&wts, "half.argus-deleting-1730000000000/x.txt", "x");
    let out = scan_no_repo(&wts, false);
    let f = facts_by_name(&out.facts);
    let mut names: Vec<&String> = f.keys().collect();
    names.sort();
    assert_eq!(
        names,
        ["has-git-dir", "has-git-file", "link", "nested", "plain"],
        "the half-deleted folder is not a candidate"
    );
    let plain = f["plain"].leftover.as_ref().unwrap();
    assert_eq!(
        (plain.files, plain.has_nested_git, f["plain"].is_git),
        (Some(1), false, false)
    );
    assert!(f["has-git-file"].leftover.is_none() && f["has-git-dir"].leftover.is_none());
    let nested = f["nested"].leftover.as_ref().unwrap();
    assert_eq!(
        (nested.has_nested_git, nested.nested_git.as_deref()),
        (true, Some("inner/.git"))
    );
    assert!(f["link"].leftover.as_ref().unwrap().is_link);
    let deleting: Vec<String> = out.deleting.iter().map(|d| base_name(&d.path)).collect();
    assert_eq!(deleting, ["half.argus-deleting-1730000000000"]);
    assert!(exists_itself(&format!("{out_root}/keep.txt")));
}

/// Now, read from the real clock.
fn real_now() -> i64 {
    crate::clock::Clock::now_ms(&crate::clock::SystemClock)
}

#[test]
fn a_link_is_a_leaf_to_the_measure_and_never_followed() {
    let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
    let (root, target) = (root_of(&dir), root_of(&outside));
    put(&target, "huge.bin", &"x".repeat(3 * 1024 * 1024));
    put(&target, "deeper/.git/HEAD", "x");
    put(&root, "own.txt", "own");
    make_link(&format!("{root}/lnk"), &target);
    backdate(&root, 9.0);
    backdate(&target, 0.0);
    let m = measure_folder(&RealFs, &root, Bounds::SCAN, &real_now);
    assert_eq!(
        (m.files, m.has_nested_git, m.size_mb),
        (Some(2), false, Some(0.0))
    );
    let age = real_now() - m.newest_ms.unwrap();
    assert!(
        age > 8 * 86_400_000,
        "the target's young files do not make the folder young"
    );
    let as_root = measure_folder(&RealFs, &format!("{root}/lnk"), Bounds::SCAN, &real_now);
    assert_eq!((as_root.is_link, as_root.files), (true, Some(0)));
}
