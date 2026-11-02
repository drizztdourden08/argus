//! I-61: `coord item block` and `unblock` set and clear the blocker text of an item (the Rust twin
//! of `item.blocker.test.mjs`).

// test-tag: critical

use crate::work_board::Work;
use serde_json::Value;

fn ok(work: &Work, words: &[&str]) -> String {
    let ran = work.worker(words);
    assert_eq!(ran.code, 0, "{words:?}\n{}\n{}", ran.out, ran.err);
    ran.out
}

#[test]
fn block_sets_the_blocker_and_unblock_clears_it() {
    let work = Work::new();
    let said = ok(&work, &["item", "block", "I-1", "--on", "D-11"]);
    assert!(said.contains("I-1: blocked on D-11."), "{said}");
    assert_eq!(work.board()["items"]["I-1"]["blocker"], "D-11");
    let said = ok(&work, &["item", "unblock", "I-1"]);
    assert!(said.contains("I-1: blocker cleared."), "{said}");
    assert_eq!(work.board()["items"]["I-1"]["blocker"], Value::Null);
}

#[test]
fn block_without_a_reason_is_refused_with_the_usage() {
    let work = Work::new();
    for words in [
        vec!["item", "block", "I-1"],
        vec!["item", "block", "I-1", "--on", "  "],
    ] {
        let ran = work.worker(&words);
        assert_eq!(ran.code, 1, "{words:?}");
        assert!(
            ran.err.contains(
                "usage: coord item block <I-n> --on \"<D-n | Q-n | external: who, what>\"."
            ),
            "{}",
            ran.err
        );
    }
    assert!(work.board()["items"]["I-1"].get("blocker").is_none());
}

#[test]
fn the_item_verb_list_names_block_and_unblock() {
    let work = Work::new();
    let ran = work.worker(&["item", "nonsense"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains("status, block, unblock, move"),
        "{}",
        ran.err
    );
}
