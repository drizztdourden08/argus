//! I-85 P1: the letters of a type name its item. `coord item show B-12` is `coord item show I-12`,
//! a wrong letter is refused with the right one, and every stored id still works.

// test-tag: critical

use crate::work_board::Work;
use serde_json::json;

/// A board with a bug (I-2), a story (I-3) and the seeded untyped item (I-1).
fn seeded() -> Work {
    let work = Work::new();
    work.worker_ok(&[
        "item",
        "add",
        "Crash",
        "--type",
        "bug",
        "--field",
        "severity=S2",
    ]);
    work.worker_ok(&["item", "add", "Plan", "--type", "story"]);
    work
}

#[test]
fn show_by_code_equals_show_by_stored_id() {
    let work = seeded();
    let stored = work.worker_ok(&["item", "show", "I-2"]);
    assert_eq!(work.worker_ok(&["item", "show", "B-2"]), stored);
    let story = work.worker_ok(&["item", "show", "I-3"]);
    assert_eq!(work.worker_ok(&["item", "show", "S-3"]), story);
    let plain = work.worker_ok(&["item", "show", "I-1"]);
    assert_eq!(work.worker_ok(&["item", "show", "K-1"]), plain);
}

#[test]
fn a_wrong_letter_is_refused_with_the_right_code() {
    let work = seeded();
    let ran = work.worker(&["item", "show", "S-2"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains("S-2 is not a story: I-2 is a bug, B-2."),
        "{}",
        ran.err
    );
    let ran = work.worker(&["item", "set", "K-2", "--title", "Nope"]);
    assert_eq!(ran.code, 1);
    assert!(ran.err.contains("I-2 is a bug, B-2."), "{}", ran.err);
    assert_eq!(work.board()["items"]["I-2"]["title"], "Crash");
}

#[test]
fn a_code_that_names_nothing_is_the_old_not_found() {
    let work = seeded();
    let ran = work.worker(&["item", "show", "B-99"]);
    assert_eq!(ran.code, 1);
    assert!(ran.err.contains("no item \"I-99\"."), "{}", ran.err);
}

#[test]
fn the_verbs_write_the_stored_id_whatever_was_typed() {
    let work = seeded();
    let said = work.worker_ok(&["item", "set", "B-2", "--title", "Crash again"]);
    assert!(said.starts_with("I-2: "), "{said}");
    work.worker_ok(&["item", "ac", "B-2", "it opens"]);
    work.worker_ok(&["item", "block", "B-2", "--on", "K-1"]);
    let bug = work.board()["items"]["I-2"].clone();
    assert_eq!(bug["title"], "Crash again");
    assert_eq!(bug["ac"].as_array().map(Vec::len), Some(1));
    assert_eq!(bug["blockedBy"], json!(["I-1"]));
    work.worker_ok(&["item", "unblock", "B-2", "--from", "K-1"]);
    let left = work.board()["items"]["I-2"]["blockedBy"].clone();
    assert_eq!(left.as_array().map_or(0, Vec::len), 0);
}

#[test]
fn a_parent_and_a_to_do_item_may_be_written_with_the_letters_too() {
    let work = seeded();
    work.worker_ok(&["item", "add", "Sub", "--type", "task", "--parent", "S-3"]);
    assert_eq!(work.board()["items"]["I-4"]["parent"], "I-3");
    work.worker_ok(&["action", "add", "Look at it", "--item", "B-2"]);
    let board = work.board();
    let todo = board["actions"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    assert_eq!(todo["item"], "I-2");
}

#[test]
fn a_plain_stored_id_and_a_soft_verb_by_code_still_work() {
    let work = seeded();
    work.worker_ok(&["item", "set", "I-2", "--title", "By stored id"]);
    assert_eq!(work.board()["items"]["I-2"]["title"], "By stored id");
    let said = work.worker_ok(&["item", "archive", "B-2"]);
    assert!(said.starts_with("I-2: archived"), "{said}");
}
