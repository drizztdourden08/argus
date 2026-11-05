//! Builds closing with their item and coming back with it.

// test-tag: critical

use serde_json::{Value, json};

use super::{
    applied, board, build, has_play, is_closed, item, owner, retire_by_hand, to_lane, trash,
};

/// A story with three builds, one of them already retired by hand.
fn story_with_builds() -> Value {
    let mut b = board();
    item(&mut b, "I-1", "story");
    for name in ["web", "api", "old"] {
        build(&mut b, name, Some("I-1"));
    }
    retire_by_hand(&mut b, "old");
    b
}

/// A build closed with its item: marked by it, no longer current, and without its Play.
fn assert_closed_with_item(b: &Value, name: &str) {
    let rec = &b["launchers"][name];
    assert_eq!(rec["retiredByItem"], "I-1");
    assert_eq!(rec["current"], false);
    assert!(!has_play(b, name), "{name} still has Play");
}

/// A build brought back with its item: live again, unmarked, and with its Play.
fn assert_brought_back(b: &Value, name: &str) {
    assert!(!is_closed(b, name), "{name} stayed closed");
    assert_eq!(b["launchers"][name].get("retiredByItem"), None);
    assert!(has_play(b, name), "{name} lost Play");
}

#[test]
fn a_story_closes_and_its_two_builds_close_with_it_and_a_reopen_brings_back_those_two() {
    let mut b = story_with_builds();
    let done = to_lane(&mut b, "I-1", "done");
    applied(&done);
    assert_eq!(done.extra["closedWith"], json!(["api", "web"]));
    assert_closed_with_item(&b, "web");
    assert_closed_with_item(&b, "api");
    assert_eq!(b["launchers"]["old"]["retiredByItem"], json!(null));
    let reopened = to_lane(&mut b, "I-1", "doing");
    applied(&reopened);
    assert_eq!(reopened.extra["reopenedWith"], json!(["api", "web"]));
    assert_brought_back(&b, "web");
    assert_brought_back(&b, "api");
    assert!(is_closed(&b, "old"));
    assert!(!has_play(&b, "old"));
}

#[test]
fn a_dropped_item_closes_its_builds_too() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    build(&mut b, "web", Some("I-1"));
    applied(&to_lane(&mut b, "I-1", "dropped"));
    assert!(is_closed(&b, "web"));
    assert_eq!(b["launchers"]["web"]["retiredByItem"], "I-1");
}

#[test]
fn the_trash_closes_the_builds_and_a_restore_brings_them_back() {
    let mut b = board();
    item(&mut b, "I-1", "bug");
    build(&mut b, "web", Some("I-1"));
    let removed = trash(&mut b, "delete", "I-1");
    applied(&removed);
    assert_eq!(removed.extra["closedWith"], json!(["web"]));
    assert!(is_closed(&b, "web"));
    assert!(!has_play(&b, "web"));
    let back = trash(&mut b, "restore", "I-1");
    applied(&back);
    assert_eq!(back.extra["reopenedWith"], json!(["web"]));
    assert!(!is_closed(&b, "web"));
    assert!(has_play(&b, "web"));
}

#[test]
fn a_bulk_move_of_three_items_closes_the_builds_of_all_three() {
    let mut b = board();
    let mut ops = Vec::new();
    for n in 1..=3 {
        let id = format!("I-{n}");
        item(&mut b, &id, "task");
        build(&mut b, &format!("b{n}"), Some(&id));
        ops.push(
            json!({"kind": "move", "target": {"c": "items", "id": id}, "to": {"status": "done"}}),
        );
    }
    let res = owner(&mut b, &json!({"kind": "bulk", "ops": ops}));
    applied(&res);
    for n in 1..=3 {
        assert!(is_closed(&b, &format!("b{n}")), "b{n} stayed live");
    }
    assert_eq!(res.extra["closedWith"], json!(["b1", "b2", "b3"]));
}

#[test]
fn a_build_serving_another_item_is_not_touched() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    item(&mut b, "I-2", "task");
    build(&mut b, "one", Some("I-1"));
    build(&mut b, "two", Some("I-2"));
    applied(&to_lane(&mut b, "I-1", "done"));
    assert!(is_closed(&b, "one"));
    assert!(!is_closed(&b, "two"));
    assert!(has_play(&b, "two"));
}

#[test]
fn a_board_with_no_link_anywhere_behaves_as_before() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    build(&mut b, "tool", None);
    let done = to_lane(&mut b, "I-1", "done");
    applied(&done);
    assert!(!is_closed(&b, "tool"));
    assert_eq!(done.extra.len(), 0);
}

#[test]
fn the_master_build_never_closes_with_an_item() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    build(&mut b, "main", Some("I-1"));
    b["launchers"]["main"]["master"] = json!(true);
    build(&mut b, "web", Some("I-1"));
    let done = to_lane(&mut b, "I-1", "done");
    applied(&done);
    assert!(!is_closed(&b, "main"));
    assert!(has_play(&b, "main"));
    assert!(is_closed(&b, "web"));
    assert_eq!(done.extra["closedWith"], json!(["web"]));
}

#[test]
fn a_build_without_a_one_line_command_comes_back_without_a_play() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    build(&mut b, "multi", Some("I-1"));
    if let Some(defs) = b["runDefs"].as_object_mut() {
        defs.shift_remove("launcher-multi");
    }
    applied(&to_lane(&mut b, "I-1", "done"));
    applied(&to_lane(&mut b, "I-1", "doing"));
    assert!(!is_closed(&b, "multi"));
    assert!(!has_play(&b, "multi"));
}

#[test]
fn a_run_definition_made_by_hand_under_the_name_is_left_alone() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    build(&mut b, "web", Some("I-1"));
    b["runDefs"]["launcher-web"]["launcher"] = json!("other");
    applied(&to_lane(&mut b, "I-1", "done"));
    assert_eq!(b["runDefs"]["launcher-web"]["launcher"], "other");
}
