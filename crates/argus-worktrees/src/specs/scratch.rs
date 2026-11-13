//! I-156 G: scratch roots. An untouched folder that matches an include glob goes in mode remove and is only listed in
//! mode report; a young one, a non-matching sibling, a folder with a `.git` inside and the root itself stay; a junction
//! inside goes as a link and its target survives.

// test-tag: standard

use crate::paths::join_path;
use crate::scratch::{ScratchMode, ScratchRoot};
use crate::specs::hygiene_kit::hygiene_on;
use crate::testkit::{Harness, backdate, exists_itself, make_link, put, root_of};
use argus_core::fixture::links::{assert_sibling_intact, fill_sibling};
use std::path::Path;
use tempfile::tempdir;

fn scratch_harness(root: &str, mode: ScratchMode) -> Harness {
    let entry = ScratchRoot {
        root: join_path(root, "scr"),
        include: vec!["gate-*".to_string()],
        min_age_days: 7.0,
        mode,
    };
    Harness::new(root, Vec::new(), |o| {
        hygiene_on(o);
        o.scratch = vec![entry];
    })
}

fn old(root: &str, name: &str) -> String {
    let path = join_path(root, &format!("scr/{name}"));
    put(&path, "a/file.txt", "x");
    backdate(&path, 30.0);
    path
}

fn planned(h: &Harness) -> Vec<String> {
    h.svc.scratch_plan().into_iter().map(|i| i.path).collect()
}

#[test]
fn an_old_matching_folder_is_removed_in_remove_mode_with_one_activity_line() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    let path = old(&root, "gate-1");
    let h = scratch_harness(&root, ScratchMode::Remove);
    assert!(h.svc.scratch_step());
    assert!(!exists_itself(&path));
    assert_eq!(h.notes.removals("removed scratch folder").len(), 1);
    assert_eq!(planned(&h).len(), 0);
}

#[test]
fn an_old_matching_folder_is_only_listed_in_report_mode() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    let path = old(&root, "gate-1");
    let h = scratch_harness(&root, ScratchMode::Report);
    assert!(h.svc.scratch_step());
    assert!(exists_itself(&path));
    assert_eq!(planned(&h), vec![path]);
    assert_eq!(h.notes.removals("scratch").len(), 0);
}

#[test]
fn a_young_folder_a_non_matching_sibling_and_a_folder_with_git_stay() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    let young = put(&root, "scr/gate-young/a.txt", "x");
    let other = old(&root, "other-1");
    let repo = old(&root, "gate-repo");
    put(&repo, "deep/er/.git", "gitdir: x");
    backdate(&repo, 30.0);
    let h = scratch_harness(&root, ScratchMode::Remove);
    assert!(h.svc.scratch_step());
    assert!(exists_itself(&young));
    assert!(exists_itself(&other));
    assert!(exists_itself(&join_path(&repo, "deep/er/.git")));
    assert_eq!(h.notes.removals("scratch").len(), 0);
}

#[test]
fn the_root_itself_is_never_removed() {
    let dir = tempdir().unwrap();
    let root = root_of(&dir);
    let scr = join_path(&root, "scr");
    let path = old(&root, "gate-1");
    backdate(&scr, 30.0);
    let h = scratch_harness(&root, ScratchMode::Remove);
    assert!(h.svc.scratch_step());
    assert!(!exists_itself(&path));
    assert!(exists_itself(&scr), "the root stays");
}

#[test]
fn a_junction_inside_goes_as_a_link_and_its_target_survives() {
    let (dir, outside) = (tempdir().unwrap(), tempdir().unwrap());
    let root = root_of(&dir);
    let target = join_path(&root_of(&outside), "outside");
    fill_sibling(Path::new(&target));
    let path = join_path(&root, "scr/gate-1");
    put(&path, "node_modules/pkg/index.js", "x");
    make_link(&join_path(&path, "node_modules/ext"), &target);
    backdate(&path, 30.0);
    let h = scratch_harness(&root, ScratchMode::Remove);
    assert!(h.svc.scratch_step());
    assert!(!exists_itself(&path));
    assert_sibling_intact(Path::new(&target));
}
