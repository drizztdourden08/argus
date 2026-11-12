//! I-61 and I-70: `coord item block` and `unblock` link the item to ids of its board, make a
//! to-do for whoever must act, and refuse free text with what to do instead.

// test-tag: critical

use crate::work_board::Work;
use serde_json::{Value, json};

/// A board with the item I-1 and a second item I-2.
fn two_items() -> Work {
    let work = Work::new();
    work.seed_item("Second");
    work
}

fn links(work: &Work, item: &str) -> Value {
    work.board()["items"][item]["blockedBy"].clone()
}

#[test]
fn block_links_an_id_and_unblock_drops_one_link_or_all() {
    let work = two_items();
    let said = work.worker_ok(&["item", "block", "I-1", "--on", "I-2"]);
    assert!(said.contains("I-1: blocked on I-2."), "{said}");
    assert_eq!(links(&work, "I-1"), json!(["I-2"]));
    let again = work.worker_ok(&["item", "block", "I-1", "--on", "I-2"]);
    assert!(again.contains("I-1: blocked on I-2."), "{again}");
    assert_eq!(links(&work, "I-1"), json!(["I-2"]));
    let said = work.worker_ok(&["item", "unblock", "I-1", "--from", "I-2"]);
    assert!(said.contains("I-1: no longer blocked on I-2."), "{said}");
    assert_eq!(links(&work, "I-1"), json!([]));
    work.worker_ok(&["item", "block", "I-1", "--on", "I-2"]);
    let said = work.worker_ok(&["item", "unblock", "I-1"]);
    assert!(said.contains("I-1: blocker cleared."), "{said}");
    assert_eq!(links(&work, "I-1"), json!([]));
}

#[test]
fn unblock_clears_the_legacy_text_and_refuses_a_link_it_does_not_have() {
    let work = two_items();
    let mut board = work.board();
    board["items"]["I-1"]["blocker"] = json!("external: owner, a go");
    work.save(&board);
    let said = work.worker_ok(&["item", "unblock", "I-1"]);
    assert!(said.contains("I-1: blocker cleared."), "{said}");
    assert_eq!(work.board()["items"]["I-1"]["blocker"], Value::Null);
    let ran = work.worker(&["item", "unblock", "I-1", "--from", "I-2"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains("I-1 is not blocked on I-2."),
        "{}",
        ran.err
    );
}

#[test]
fn a_plain_text_blocker_is_refused_with_the_hint_and_nothing_is_written() {
    let work = two_items();
    let before = work.board();
    let ran = work.worker(&["item", "block", "I-1", "--on", "needs the schema"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains("a blocker is an id, not text"),
        "{}",
        ran.err
    );
    assert!(
        ran.err
            .contains("coord item block I-1 --todo \"<what>\" --who owner|<thread>|lead:<key>"),
        "{}",
        ran.err
    );
    assert_eq!(work.board(), before);
}

#[test]
fn an_external_blocker_is_stored_as_text_and_unblock_clears_it() {
    let work = two_items();
    let said = work.worker_ok(&["item", "block", "I-1", "--on", "External: owner, a go"]);
    assert!(said.contains("I-1: blocked on External: owner, a go."), "{said}");
    let item = work.board()["items"]["I-1"].clone();
    assert_eq!(item["blocker"], "External: owner, a go");
    assert_eq!(item["blockedBy"], Value::Null);
    let said = work.worker_ok(&["item", "unblock", "I-1"]);
    assert!(said.contains("I-1: blocker cleared."), "{said}");
    assert_eq!(work.board()["items"]["I-1"]["blocker"], Value::Null);
    let ran = work.worker(&["item", "block", "I-1", "--on", "external:  "]);
    assert_eq!(ran.code, 1);
}

#[test]
fn an_id_of_another_board_points_to_dep_ask() {
    let work = two_items();
    let ran = work.worker(&["item", "block", "I-1", "--on", "brock/I-9"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err
            .contains("brock/I-9 is on another board: coord dep ask brock \"<what>\" --item I-1."),
        "{}",
        ran.err
    );
}

#[test]
fn an_unknown_id_a_note_and_a_cycle_are_refused_by_the_board() {
    let work = two_items();
    let missing = work.worker(&["item", "block", "I-1", "--on", "I-99"]);
    assert_eq!(missing.code, 1);
    assert!(
        missing.err.contains("I-99 is not on this board"),
        "{}",
        missing.err
    );
    work.worker_ok(&["item", "block", "I-1", "--on", "I-2"]);
    let cycle = work.worker(&["item", "block", "I-2", "--on", "I-1"]);
    assert_eq!(cycle.code, 1);
    assert!(cycle.err.contains("cycle"), "{}", cycle.err);
    let itself = work.worker(&["item", "block", "I-1", "--on", "I-1"]);
    assert!(itself.err.contains("cannot block itself"), "{}", itself.err);
}

#[test]
fn todo_makes_a_to_do_for_who_and_links_it() {
    let work = two_items();
    let said = work.worker_ok(&[
        "item",
        "block",
        "I-1",
        "--todo",
        "Run the login test",
        "--who",
        "owner",
    ]);
    assert!(said.contains("I-1: blocked on H-"), "{said}");
    assert!(said.contains("(to-do for owner)."), "{said}");
    let board = work.board();
    let actions = board["actions"].as_object().cloned().unwrap_or_default();
    assert_eq!(actions.len(), 1);
    let (id, todo) = actions.iter().next().unwrap();
    assert_eq!(todo["text"], "Run the login test");
    assert_eq!(todo["item"], "I-1");
    assert_eq!(todo["assignee"], "owner");
    assert_eq!(board["items"]["I-1"]["blockedBy"], json!([id]));
}

#[test]
fn todo_needs_who_and_never_comes_with_on() {
    let work = two_items();
    let no_who = work.worker(&["item", "block", "I-1", "--todo", "A go"]);
    assert_eq!(no_who.code, 1);
    assert!(no_who.err.contains("--todo needs --who"), "{}", no_who.err);
    let both = work.worker(&[
        "item", "block", "I-1", "--on", "I-2", "--todo", "A go", "--who", "owner",
    ]);
    assert_eq!(both.code, 1);
    assert!(both.err.contains("usage: coord item block"), "{}", both.err);
    let board = work.board();
    assert_eq!(
        board["actions"].as_object().map_or(0, serde_json::Map::len),
        0
    );
}

#[test]
fn block_without_a_target_is_refused_with_the_usage() {
    let work = two_items();
    for words in [
        vec!["item", "block", "I-1"],
        vec!["item", "block", "I-1", "--on", "  "],
    ] {
        let ran = work.worker(&words);
        assert_eq!(ran.code, 1, "{words:?}");
        assert!(
            ran.err.contains(
                "usage: coord item block <I-n> --on <I-n|T-n|H-n|Q-n|D-n> | --todo \"<what>\" --who <owner|thread|lead:key>."
            ),
            "{}",
            ran.err
        );
    }
    assert!(work.board()["items"]["I-1"].get("blockedBy").is_none());
}

#[test]
fn the_item_verb_list_names_the_new_verbs() {
    let work = Work::new();
    let ran = work.worker(&["item", "nonsense"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err.contains("status, block, unblock, move"),
        "{}",
        ran.err
    );
    assert!(
        ran.err.contains("list, tree, migrate-blockers"),
        "{}",
        ran.err
    );
}
