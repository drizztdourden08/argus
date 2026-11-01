//! Delete safety of the prune with real links: a directory junction (`mklink /J`) and, where the machine allows one, a
//! file symlink, pointing at a SIBLING folder of the cache root full of files. The sibling must come out untouched; a
//! link inside a deleted entry goes with it, and a link in the cache's own folders is never followed.
//! Keep rule: E2.
// test-tag: e2e

mod rig;

use argus_core::fixture::links::{assert_sibling_intact, exists_itself, file_symlink, junction};
use rig::{DAY, Rig};

const NAME: &str = "@drizztdourden08+brock-core";

#[test]
#[ignore = "e2e"]
fn a_pruned_entry_takes_its_links_with_it_and_never_their_targets() {
    let rig = Rig::new();
    let sibling = rig.sibling();
    rig.entry("package", NAME, "0.100.0", "node-aa", 10, 40 * DAY);
    let entry = rig
        .version_folder("package", NAME, "0.100.0")
        .join("node-aa");
    junction(&entry.join("files/linked"), &sibling);
    let _ = file_symlink(&entry.join("files/linked.txt"), &sibling.join("keep.txt"));
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted.len(), 1, "{outcome:?}");
    assert!(!exists_itself(&entry), "the entry and its links are gone");
    assert!(outcome.failed.is_empty(), "{outcome:?}");
    assert_sibling_intact(&sibling);
}

#[test]
#[ignore = "e2e"]
fn an_entry_or_a_version_that_is_itself_a_junction_is_never_entered() {
    let rig = Rig::new();
    let sibling = rig.sibling();
    let outside_entry = sibling.join("node-aa");
    std::fs::create_dir_all(&outside_entry).expect("folder");
    rig.age(&outside_entry, 40 * DAY);
    junction(
        &rig.version_folder("package", NAME, "0.100.0")
            .join("node-aa"),
        &sibling,
    );
    junction(&rig.version_folder("package", NAME, "0.101.0"), &sibling);
    let outcome = rig.prune(14).expect("prune");
    assert!(outcome.deleted.is_empty(), "{outcome:?}");
    assert_sibling_intact(&sibling);
    assert!(outside_entry.exists());
}

#[test]
#[ignore = "e2e"]
fn a_tmp_folder_that_is_a_junction_is_never_entered() {
    let rig = Rig::new();
    let sibling = rig.sibling();
    let old_build = sibling.join("old-build");
    std::fs::create_dir_all(&old_build).expect("folder");
    std::fs::write(old_build.join("half.js"), "x").expect("file");
    rig.age(&old_build, 2 * DAY);
    junction(&rig.root_path().join(".tmp"), &sibling);
    rig.entry("package", NAME, "0.100.0", "node-aa", 10, 40 * DAY);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.abandoned_deleted, 0, "{outcome:?}");
    assert!(
        old_build.join("half.js").exists(),
        "the old folder behind the junction stays"
    );
    assert_sibling_intact(&sibling);
    assert!(
        rig.exists("package", NAME, "0.100.0", "node-aa"),
        "an entry is never parked through a .tmp junction"
    );
}
