//! The master build: one per board, never taken away, never serving the rule of the others.

use serde_json::json;

use super::{board, build, retire_by_hand};
use crate::launchers::{has_master, is_standalone};
use crate::served::{
    check_make_master, check_not_last_master, hand_over_master, is_master, master_name,
};

fn flagged(name: &str) -> serde_json::Value {
    let mut b = board();
    build(&mut b, name, None);
    b["launchers"][name]["master"] = json!(true);
    b
}

#[test]
fn the_flagged_build_is_the_master_and_a_board_without_one_has_none() {
    let b = flagged("main");
    assert_eq!(master_name(&b).as_deref(), Some("main"));
    assert!(has_master(&b));
    assert!(is_master(&b, "main"));
    assert_eq!(master_name(&board()), None);
    assert!(!has_master(&board()));
}

#[test]
fn a_board_not_migrated_yet_reads_the_launcher_named_master() {
    let mut b = board();
    build(&mut b, "master", None);
    assert_eq!(master_name(&b).as_deref(), Some("master"));
    build(&mut b, "dev", None);
    b["launchers"]["dev"]["master"] = json!(true);
    assert_eq!(master_name(&b).as_deref(), Some("dev"));
    assert!(!is_master(&b, "master"));
}

#[test]
fn a_retired_master_leaves_the_board_without_one() {
    let mut b = flagged("main");
    retire_by_hand(&mut b, "main");
    assert_eq!(master_name(&b), None);
    assert!(!has_master(&b));
}

#[test]
fn a_second_master_is_refused_with_the_name_of_the_first() {
    let mut b = flagged("main");
    build(&mut b, "other", None);
    let text = check_make_master(&b, "other").unwrap_err().to_string();
    assert!(text.contains("main is already the master build"), "{text}");
    assert!(check_make_master(&b, "main").is_ok());
    assert!(check_make_master(&board(), "other").is_ok());
}

#[test]
fn the_last_master_cannot_be_retired_or_removed_but_another_build_can() {
    let mut b = flagged("main");
    build(&mut b, "other", None);
    let text = check_not_last_master(&b, "main").unwrap_err().to_string();
    assert!(text.contains("a board always has one"), "{text}");
    assert!(check_not_last_master(&b, "other").is_ok());
}

#[test]
fn the_role_is_handed_over_to_a_live_build_and_the_old_master_loses_it() {
    let mut b = flagged("main");
    build(&mut b, "other", None);
    let before = hand_over_master(&mut b, "other", "ui").unwrap();
    assert_eq!(before.as_deref(), Some("main"));
    assert_eq!(master_name(&b).as_deref(), Some("other"));
    assert_eq!(b["launchers"]["main"].get("master"), None);
    assert!(check_not_last_master(&b, "main").is_ok());
    assert_eq!(hand_over_master(&mut b, "other", "ui").unwrap(), None);
}

#[test]
fn the_role_cannot_go_to_a_retired_or_missing_build() {
    let mut b = flagged("main");
    build(&mut b, "gone", None);
    retire_by_hand(&mut b, "gone");
    assert!(hand_over_master(&mut b, "gone", "ui").is_err());
    assert!(hand_over_master(&mut b, "nobody", "ui").is_err());
    assert_eq!(master_name(&b).as_deref(), Some("main"));
}

#[test]
fn only_an_unlinked_build_that_is_not_the_master_has_a_usage_window() {
    let mut b = flagged("main");
    build(&mut b, "tool", None);
    build(&mut b, "web", Some("I-1"));
    assert!(!is_standalone(&b, &b["launchers"]["main"]));
    assert!(is_standalone(&b, &b["launchers"]["tool"]));
    assert!(!is_standalone(&b, &b["launchers"]["web"]));
}
