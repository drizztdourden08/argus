//! KEEP: K4 (the walk that looks for a git of its own inside a folder before it is deleted).
//!
//! The depth, the entry cap and the unreadable folder of the walk, on a temporary folder: a folder the walk could
//! not search is never read as empty.

// test-tag: critical

use crate::fs::RealFs;
use crate::leftover_remove::{Within, git_within, git_within_capped};
use crate::testkit::{HookFs, put};
use std::fs;

#[test]
fn a_git_within_three_levels_is_found_one_deeper_than_three_is_not_asked_for() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::testkit::root_of(&dir);
    fs::create_dir_all(format!("{root}/junk/a/b/c/.git")).unwrap();
    let Within::Git(found) = git_within(&RealFs, &format!("{root}/junk")) else {
        panic!("the .git three levels down is found");
    };
    assert!(found.ends_with("junk/a/b/c/.git"), "{found}");
    fs::create_dir_all(format!("{root}/deep/a/b/c/d/.git")).unwrap();
    assert_eq!(
        git_within(&RealFs, &format!("{root}/deep")),
        Within::Nothing
    );
    put(&root, "node/node_modules/x/.git/HEAD", "x");
    assert_eq!(
        git_within(&RealFs, &format!("{root}/node")),
        Within::Nothing,
        "node_modules is not entered"
    );
}

#[test]
fn a_walk_that_hits_its_cap_is_too_big_never_nothing_found() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::testkit::root_of(&dir);
    for n in 0..5 {
        fs::create_dir_all(format!("{root}/big/d{n}")).unwrap();
    }
    let big = format!("{root}/big");
    assert_eq!(git_within_capped(&RealFs, &big, 6), Within::Nothing);
    assert_eq!(git_within_capped(&RealFs, &big, 5), Within::TooBig);
}

#[test]
fn a_folder_the_walk_cannot_read_is_unreadable_never_nothing_found() {
    let dir = tempfile::tempdir().unwrap();
    let root = crate::testkit::root_of(&dir);
    fs::create_dir_all(format!("{root}/junk/a/locked")).unwrap();
    let hook = HookFs::default();
    assert_eq!(git_within(&hook, &format!("{root}/junk")), Within::Nothing);
    *hook.unreadable_containing.lock().unwrap() = Some("/a/locked".to_string());
    let Within::Unreadable(why) = git_within(&hook, &format!("{root}/junk")) else {
        panic!("the folder that cannot be read stops the walk");
    };
    assert!(why.ends_with("junk/a/locked: EPERM"), "{why}");
}
