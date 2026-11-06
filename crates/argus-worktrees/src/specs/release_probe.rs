//! KEEP: K3.
//! The release probe: a folder that cannot be renamed is refused before git or the backup runs; a free one goes on.

// test-tag: critical

use crate::error::ErrorCode;
use crate::testkit::{Fixture, Harness, exists_itself, listed, rule_for};

fn setup() -> (Fixture, Harness) {
    let fx = Fixture::fake();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    (fx, h)
}

#[test]
fn a_folder_that_cannot_be_renamed_is_refused_and_git_never_runs() {
    let (fx, h) = setup();
    let id = h.id_of(&fx.path("old-merged"));
    let before = h.exec.calls.lock().unwrap().len();
    *h.fs.probe_error.lock().unwrap() = Some(std::io::ErrorKind::PermissionDenied);
    let err = h.refused(&id, |_| {});
    assert_eq!((err.code, err.code.status()), (ErrorCode::InUse, 423));
    assert!(
        err.message
            .contains("is in use (open in a process or locked")
            && err.message.contains(
                "nothing was removed. Close what holds it and request the removal again."
            ),
        "{}",
        err.message
    );
    let calls = h.exec.calls.lock().unwrap();
    assert!(
        calls[before..].iter().all(|a| !a
            .iter()
            .any(|w| w == "remove" || w == "unlock" || w == "prune")),
        "{:?}",
        &calls[before..]
    );
    assert!(exists_itself(&format!("{}/.git", fx.path("old-merged"))));
    assert!(listed(&fx.repo).contains(&"old-merged".to_string()));
}

#[test]
fn a_folder_that_can_be_renamed_goes_on_and_is_removed_as_before() {
    let (fx, h) = setup();
    let out = h.rm(&h.id_of(&fx.path("old-merged"))).unwrap();
    assert!(out.removed);
    assert!(!exists_itself(&fx.path("old-merged")));
}
