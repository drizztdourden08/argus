//! K3, K4: an old copy that is a junction or a symlink goes as a link and its target stays.
//! The junction is made by `cmd /C mklink`, so this is a real tool run.
// test-tag: e2e

use super::remove_old_copies;

/// A set-aside name that is a junction (or a file symlink) to a sibling folder goes as a link:
/// the files it points at stay.
#[test]
#[ignore = "e2e"]
fn an_old_copy_that_is_a_link_goes_as_a_link_and_its_target_stays() {
    use argus_core::fixture::links::{
        assert_sibling_intact, exists_itself, file_symlink, fill_sibling, junction,
    };
    let dir = tempfile::tempdir().unwrap();
    let sibling = dir.path().join("sibling");
    fill_sibling(&sibling);
    let stable = dir.path().join("stable");
    let mut links = vec![stable.join("argus.exe.old-dir")];
    junction(&links[0], &sibling);
    let file = stable.join("argus.exe.old-file");
    if file_symlink(&file, &sibling.join("keep.txt")) {
        links.push(file);
    }
    remove_old_copies(&stable, "argus.exe");
    assert_sibling_intact(&sibling);
    for link in &links {
        assert!(!exists_itself(link), "{} is gone", link.display());
    }
}
