//! KEEP: K3 (a removal never follows a link out of the worktree).
//! A worktree holds a pnpm-style junction deep in `node_modules`, a top junction and, where the machine allows one, a
//! file symlink, all pointing at a folder in another temporary folder that no rule covers. After the removal the
//! worktree is gone and every file of the outside folder is still there. A junction is `mklink /J` on Windows and a
//! symlink elsewhere; the file symlink is skipped (with a line) where the machine does not allow one.

// test-tag: standard

use crate::paths::join_path;
use crate::testkit::{Fixture, Harness, exists_itself, make_link, put, root_of, rule_for};
use argus_core::fixture::links::{assert_sibling_intact, file_symlink, fill_sibling};
use std::path::Path;

#[test]
fn a_removal_leaves_the_folder_a_junction_inside_points_to_outside_the_worktree() {
    let fx = Fixture::fake();
    put(&fx.repo, ".git/info/exclude", "node_modules\nlnk-*\n");
    let outside_dir = tempfile::tempdir().unwrap();
    let outside = join_path(&root_of(&outside_dir), "outside");
    fill_sibling(Path::new(&outside));
    let path = fx.path("old-merged");
    put(&path, "node_modules/.pnpm/pkg@1/index.js", "x");
    let links = [
        join_path(&path, "node_modules/ext"),
        join_path(&path, "lnk-top"),
    ];
    for link in &links {
        make_link(link, &outside);
    }
    let file_link = join_path(&path, "lnk-file.txt");
    let file_made = file_symlink(
        Path::new(&file_link),
        Path::new(&join_path(&outside, "keep.txt")),
    );
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |o| o.drop_links_first = true);
    h.scan();
    let out = h.rm(&h.id_of(&path)).unwrap();
    assert!(out.removed, "{}", out.text);
    h.idle();
    assert!(!exists_itself(&path), "the worktree is gone");
    for link in links.iter().chain(file_made.then_some(&file_link)) {
        assert!(!exists_itself(link), "the link {link} went with it");
    }
    assert_sibling_intact(Path::new(&outside));
}
