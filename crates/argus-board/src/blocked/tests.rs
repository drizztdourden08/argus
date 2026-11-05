// test-tag: critical

use serde_json::{Value, json};

use super::{Cause, CauseKind, causes};

fn blocked_by(ids: &[&str], rest: &Value) -> Value {
    let mut board = json!({"items": {"I-1": {"id": "I-1", "blockedBy": ids}}});
    for (coll, docs) in rest.as_object().into_iter().flatten() {
        board[coll.as_str()] = docs.clone();
    }
    board
}

fn kinds(board: &Value) -> Vec<CauseKind> {
    causes(board, "I-1").iter().map(|c| c.kind).collect()
}

fn only(board: &Value) -> Cause {
    let found = causes(board, "I-1");
    assert_eq!(found.len(), 1, "{found:?}");
    found[0].clone()
}

#[test]
fn a_bare_item_and_a_missing_item_have_no_cause() {
    assert_eq!(
        causes(&json!({"items": {"I-1": {"id": "I-1"}}}), "I-1").len(),
        0
    );
    assert_eq!(causes(&json!({}), "I-1").len(), 0);
}

#[test]
fn a_linked_item_that_is_not_done_or_dropped_blocks() {
    let board = blocked_by(
        &["I-2"],
        &json!({"items": {
            "I-1": {"id": "I-1", "blockedBy": ["I-2"]},
            "I-2": {"id": "I-2", "title": "The schema", "status": "doing"},
        }}),
    );
    let cause = only(&board);
    assert_eq!(cause.kind, CauseKind::Item);
    assert_eq!(cause.id, "I-2");
    assert_eq!(cause.label, "The schema");
    assert_eq!(cause.who, None);
}

#[test]
fn a_linked_test_that_has_not_passed_blocks() {
    let tests = json!({"tests": {"T-3": {"title": "Login works", "status": "fail"}}});
    let cause = only(&blocked_by(&["T-3"], &tests));
    assert_eq!(cause.kind, CauseKind::Test);
    assert_eq!(cause.id, "T-3");
    assert_eq!(cause.label, "Login works");
}

#[test]
fn a_linked_pending_todo_blocks_and_names_who_holds_it() {
    let todos = json!({"actions": {
        "H-4": {"text": "Run the bundle", "status": "pending", "assignee": "owner", "thread": "ui"},
        "H-5": {"text": "Merge it", "status": "pending", "thread": "api"},
    }});
    let owner = only(&blocked_by(&["H-4"], &todos));
    assert_eq!(owner.kind, CauseKind::Todo);
    assert_eq!(owner.label, "Run the bundle");
    assert_eq!(owner.who.as_deref(), Some("owner"));
    let thread = only(&blocked_by(&["H-5"], &todos));
    assert_eq!(thread.who.as_deref(), Some("api"));
}

#[test]
fn an_open_question_of_the_item_blocks() {
    let entries = json!({"entries": {"Q-6": {"item": "I-1", "kind": "question", "status": "open", "text": "Which port?"}}});
    let cause = only(&blocked_by(&[], &entries));
    assert_eq!(cause.kind, CauseKind::Ask);
    assert_eq!(cause.id, "Q-6");
    assert_eq!(cause.label, "Which port?");
}

#[test]
fn a_pending_decision_of_the_item_blocks() {
    let entries = json!({"entries": {"D-7": {"item": "I-1", "kind": "decision", "status": "pending", "text": "Pick one"}}});
    assert_eq!(only(&blocked_by(&[], &entries)).kind, CauseKind::Decision);
}

#[test]
fn a_linked_question_or_decision_blocks_while_it_is_open_even_on_another_item() {
    let entries = json!({"entries": {
        "Q-6": {"item": "I-9", "kind": "question", "status": "open"},
        "D-7": {"item": "I-9", "kind": "decision", "status": "pending"},
    }});
    let board = blocked_by(&["Q-6", "D-7"], &entries);
    assert_eq!(kinds(&board), vec![CauseKind::Ask, CauseKind::Decision]);
}

#[test]
fn a_question_that_is_both_linked_and_the_items_own_counts_once() {
    let entries =
        json!({"entries": {"Q-6": {"item": "I-1", "kind": "question", "status": "open"}}});
    assert_eq!(causes(&blocked_by(&["Q-6"], &entries), "I-1").len(), 1);
}

#[test]
fn a_request_to_another_board_that_is_opening_or_open_blocks() {
    for state in ["opening", "open"] {
        let mut board = blocked_by(&[], &json!({}));
        board["items"]["I-1"]["waitsOn"] =
            json!({"dep": "D-2", "project": "brock", "title": "A new gate", "state": state});
        let cause = only(&board);
        assert_eq!(cause.kind, CauseKind::DepWaits);
        assert_eq!(cause.id, "D-2");
        assert_eq!(cause.label, "A new gate");
        assert_eq!(cause.who.as_deref(), Some("lead:brock"));
    }
}

#[test]
fn a_blocked_request_of_another_board_blocks_and_one_that_is_not_does_not() {
    let mut board = blocked_by(&[], &json!({}));
    board["items"]["I-1"]["requestedBy"] = json!([
        {"dep": "D-3", "state": "open"},
        {"dep": "D-4", "title": "Waiting for us", "state": "blocked"},
    ]);
    let cause = only(&board);
    assert_eq!(cause.kind, CauseKind::DepBlocked);
    assert_eq!(cause.id, "D-4");
}

#[test]
fn the_legacy_blocker_text_blocks_until_it_is_migrated() {
    let mut board = blocked_by(&[], &json!({}));
    board["items"]["I-1"]["blocker"] = json!("external: owner must test");
    let cause = only(&board);
    assert_eq!(cause.kind, CauseKind::Legacy);
    assert_eq!(cause.label, "external: owner must test");
    board["items"]["I-1"]["blocker"] = json!("");
    assert_eq!(causes(&board, "I-1").len(), 0);
}

#[test]
fn a_settled_link_never_counts() {
    let links = [
        "I-2", "I-3", "I-4", "T-5", "H-6", "Q-7", "D-8", "I-1", "I-99", "X-1",
    ];
    let board = blocked_by(
        &links,
        &json!({
            "items": {
                "I-1": {"id": "I-1", "blockedBy": links},
                "I-2": {"id": "I-2", "status": "done"},
                "I-3": {"id": "I-3", "status": "dropped"},
                "I-4": {"id": "I-4", "status": "doing", "deletedAt": "2026-10-22T10:00:00.000Z"},
            },
            "tests": {"T-5": {"status": "pass"}},
            "actions": {"H-6": {"status": "done"}},
            "entries": {
                "Q-7": {"item": "I-1", "kind": "question", "status": "answered"},
                "D-8": {"item": "I-1", "kind": "decision", "status": "applied"},
            },
        }),
    );
    assert_eq!(
        causes(&board, "I-1").len(),
        0,
        "{:?}",
        causes(&board, "I-1")
    );
}

#[test]
fn a_deleted_or_archived_ask_of_the_item_does_not_count() {
    for field in ["deletedAt", "archivedAt"] {
        let mut ask = json!({"item": "I-1", "kind": "question", "status": "open"});
        ask[field] = json!("2026-10-22T10:00:00.000Z");
        let board = blocked_by(&[], &json!({"entries": {"Q-1": ask}}));
        assert_eq!(causes(&board, "I-1").len(), 0);
    }
}

#[test]
fn causes_come_in_the_order_of_the_kinds() {
    let ids = ["Q-6", "T-3", "I-2"];
    let mut board = blocked_by(
        &ids,
        &json!({
            "items": {
                "I-1": {"id": "I-1", "blockedBy": ids, "blocker": "later", "waitsOn": {"dep": "D-1", "state": "open"}},
                "I-2": {"id": "I-2", "status": "todo"},
            },
            "tests": {"T-3": {"status": "pending"}},
            "entries": {"Q-6": {"item": "I-1", "kind": "question", "status": "open"}},
        }),
    );
    board["items"]["I-1"]["requestedBy"] = json!({"dep": "D-2", "state": "blocked"});
    assert_eq!(
        kinds(&board),
        vec![
            CauseKind::Item,
            CauseKind::Test,
            CauseKind::Ask,
            CauseKind::DepWaits,
            CauseKind::DepBlocked,
            CauseKind::Legacy
        ]
    );
}

#[test]
fn a_cause_serializes_with_its_kind_in_kebab_case() {
    let mut board = blocked_by(&[], &json!({}));
    board["items"]["I-1"]["waitsOn"] = json!({"dep": "D-2", "state": "open"});
    let cause = serde_json::to_value(only(&board)).unwrap_or_default();
    assert_eq!(
        cause,
        json!({"kind": "dep-waits", "id": "D-2", "label": "D-2", "who": null})
    );
}
