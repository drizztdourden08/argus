//! KEEP: K3, K4.
//! A folder rule's name filters sort only the folders git does not register (D-90): a worktree the repository
//! registers under the rule folder is listed and judged whatever its name, and a folder that is no work tree is judged
//! and removed through the safe path, links unlinked first.

// test-tag: standard

use crate::board::Caller;
use crate::hygiene::{ApproveInput, HygieneState};
use crate::paths::join_path;
use crate::specs::hygiene_kit::{DEBOUNCE, hygienic, inbox, kind, left_why, state, step};
use crate::testkit::{Fixture, Harness, exists_itself, listed, make_link, put, rule_for};
use argus_core::fixture::links::{assert_sibling_intact, fill_sibling};
use std::path::Path;

fn agents_only(fx: &Fixture) -> Harness {
    hygienic(fx, rule_for(fx).filtered(&["agent-*"], &[]))
}

#[test]
fn a_registered_worktree_is_listed_and_judged_whatever_its_name() {
    let fx = Fixture::fake_bare();
    fx.add("wave-a");
    put(&fx.wts, "wave-x/notes.txt", "not registered\n");
    let h = agents_only(&fx);
    h.scan();
    h.svc.start();
    let docs = h.docs();
    assert!(docs.contains_key("wave-a"), "{:?}", docs.keys());
    assert!(
        !docs.contains_key("wave-x"),
        "an unregistered folder stays filtered"
    );
    assert_eq!(state(&h, "wave-a"), Some((HygieneState::Removing, None)));
}

#[test]
fn a_folder_that_is_no_work_tree_is_left_for_the_lead_and_goes_without_touching_a_junction_target()
{
    let fx = Fixture::fake_bare();
    fx.add("agent-ok");
    let path = fx.add("agent-ng");
    std::fs::remove_file(join_path(&path, ".git")).unwrap();
    put(&path, "notes.txt", "left behind\n");
    let sibling = join_path(&fx.root, "sibling");
    fill_sibling(Path::new(&sibling));
    make_link(&join_path(&path, "lnk"), &sibling);
    let h = agents_only(&fx);
    h.scan();
    h.svc.start();
    assert_eq!(
        left_why(&h, "agent-ng"),
        "not a git worktree: 3 entries left"
    );
    step(&h, DEBOUNCE);
    let input = ApproveInput {
        project: "alpha".into(),
        ids: vec![h.id_of(&path)],
        caller: Caller::Thread("worker".into()),
        reason: None,
    };
    let done = h.svc.approve_many(&input).unwrap();
    assert_eq!(done.refused.len(), 0, "{:?}", done.refused);
    assert_eq!(kind(&h, "agent-ng"), Some(HygieneState::Removing));
    h.svc.requests_idle();
    assert!(
        !exists_itself(&path),
        "{:?} {:?}",
        inbox(&h),
        h.docs().get("agent-ng").map(|d| d.hygiene.clone())
    );
    assert!(!listed(&fx.repo).contains(&"agent-ng".to_string()));
    assert_sibling_intact(Path::new(&sibling));
}
