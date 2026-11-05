//! I-99: the old `coord build` verbs are aliases of `coord test run` for one more release.

// test-tag: critical

use crate::agent_common::{As, Rig};

#[test]
fn build_list_says_the_new_verb_and_lists_the_runs() {
    let rig = Rig::new();
    let ran = rig.ok(&["build", "list"], As::Nobody);
    assert!(
        ran.out.contains("coord build list is now coord test run list"),
        "{}",
        ran.out
    );
}

#[test]
fn build_add_and_set_are_gone_and_name_their_replacement() {
    let rig = Rig::new();
    rig.with_master();
    rig.refused(
        &["build", "add", "bundle-4", "--for", "I-1"],
        As::Lead,
        "coord build add is gone",
    );
    rig.refused(
        &["build", "set", "bundle-4", "--title", "x"],
        As::Lead,
        "coord build set is gone",
    );
}

#[test]
fn build_hand_needs_a_run_and_a_master_launcher() {
    let rig = Rig::new();
    rig.fails(&["build", "hand", "I-404"], As::Lead);
    rig.fails(&["test", "run", "hand", "I-404"], As::Lead);
}

#[test]
fn an_unknown_build_verb_is_refused() {
    let rig = Rig::new();
    rig.fails(&["build", "frobnicate"], As::Lead);
}
