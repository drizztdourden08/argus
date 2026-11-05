//! The link rules of `for`, and what serves an item.

// test-tag: critical

use serde_json::json;

use super::{applied, board, build, item, retire_by_hand, to_lane};
use crate::served::{check_for, served_by};

fn refusal(b: &serde_json::Value, master: bool, target: Option<&str>) -> String {
    check_for(b, master, target).unwrap_err().to_string()
}

#[test]
fn a_build_without_a_link_is_refused_and_the_master_build_goes_without() {
    let b = board();
    let text = refusal(&b, false, None);
    assert!(text.contains("needs the item it serves"), "{text}");
    assert!(text.contains("Only the master build"), "{text}");
    assert!(check_for(&b, true, None).is_ok());
    assert!(check_for(&b, false, Some("  ")).is_err());
}

#[test]
fn a_build_may_serve_each_open_type_of_the_registry() {
    let mut b = board();
    for (n, kind) in ["epic", "story", "task", "bug", "debt", "test-plan"]
        .iter()
        .enumerate()
    {
        let id = format!("I-{}", n + 1);
        item(&mut b, &id, kind);
        assert!(check_for(&b, false, Some(&id)).is_ok(), "{kind}");
    }
}

#[test]
fn an_item_without_a_type_reads_as_a_task() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    if let Some(rec) = b["items"]["I-1"].as_object_mut() {
        rec.shift_remove("type");
    }
    assert!(check_for(&b, false, Some("I-1")).is_ok());
}

#[test]
fn a_closed_item_is_refused_with_its_state_and_the_master_build_may_still_name_it() {
    let mut b = board();
    item(&mut b, "I-1", "task");
    item(&mut b, "I-2", "task");
    item(&mut b, "I-3", "task");
    applied(&to_lane(&mut b, "I-1", "done"));
    applied(&to_lane(&mut b, "I-2", "dropped"));
    b["items"]["I-3"]["deletedAt"] = json!("2026-01-02T00:00:00.000Z");
    assert!(refusal(&b, false, Some("I-1")).contains("I-1 is done"));
    assert!(refusal(&b, false, Some("I-2")).contains("I-2 is dropped"));
    assert!(refusal(&b, false, Some("I-3")).contains("I-3 is in the trash"));
    assert!(check_for(&b, true, Some("I-1")).is_ok());
}

#[test]
fn a_test_case_or_a_sub_item_is_refused() {
    let b = board();
    for id in ["T-1", "Q-2", "D-3", "H-4", "N-5"] {
        let text = refusal(&b, false, Some(id));
        assert!(text.contains(&format!("{id} is not a work item")), "{text}");
    }
}

#[test]
fn another_boards_item_or_a_made_up_id_is_refused() {
    let b = board();
    assert!(refusal(&b, false, Some("rotp/I-3")).contains("its own board"));
    assert!(refusal(&b, false, Some("rotp:I-3")).contains("its own board"));
    assert!(refusal(&b, false, Some("the lobby")).contains("takes an item id"));
}

#[test]
fn an_item_that_is_not_on_this_board_is_refused() {
    let b = board();
    let text = refusal(&b, false, Some("I-9"));
    assert!(text.contains("no item I-9 on this board"), "{text}");
}

#[test]
fn a_type_a_build_cannot_serve_is_refused_with_the_list() {
    let mut b = board();
    item(&mut b, "I-1", "ghost");
    let text = refusal(&b, false, Some("I-1"));
    assert!(text.contains("a build cannot serve I-1, a ghost"), "{text}");
    assert!(
        text.contains("epic, story, task, bug, debt, test-plan"),
        "{text}"
    );
}

#[test]
fn an_item_lists_its_live_and_closed_builds_and_its_worktree_links() {
    let mut b = board();
    item(&mut b, "I-1", "story");
    item(&mut b, "I-2", "story");
    build(&mut b, "web", Some("I-1"));
    build(&mut b, "old", Some("I-1"));
    build(&mut b, "other", Some("I-2"));
    retire_by_hand(&mut b, "old");
    b["worktreeLinks"] = json!({
        "wt-a": {"item": "I-1", "by": "ui", "at": "2026-01-01T00:00:00.000Z"},
        "wt-b": {"item": "I-2", "by": "ui", "at": "2026-01-01T00:00:00.000Z"},
    });
    let served = served_by(&b, "I-1");
    assert_eq!(served.builds, ["web"]);
    assert_eq!(served.closed, ["old"]);
    assert_eq!(served.worktree_links, ["wt-a"]);
    assert_eq!(served_by(&b, "I-3").builds.len(), 0);
}
