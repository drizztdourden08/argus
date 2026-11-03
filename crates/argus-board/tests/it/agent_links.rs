//! Agents as linked records in the board ops and the export: the `items`, `master` and `blockedBy`
//! fields of an edit, and the page docs that carry `items`, `where` and the agents of an item.
//! I-85 P2.

// test-tag: critical

use crate::common::{alpha, applied, exported, refused};
use argus_board::ops::{OpResult, Who, apply_op};
use argus_core::Coll;
use serde_json::{Value, json};

fn edit_agent(board: &mut Value, id: &str, set: Value) -> OpResult {
    let mut op = json!({"kind": "edit", "target": {"c": "agents", "id": id}});
    op["set"] = set;
    apply_op(board, &op, &Who::human(), None)
}

#[test]
fn an_old_agent_record_exports_its_item_as_the_list_and_no_place_without_a_claude_id() {
    let b = alpha();
    let a1 = exported(&b, Coll::Agents, "A-1");
    assert_eq!(a1["items"], json!(["I-1"]));
    assert_eq!(a1["master"], false);
    assert_eq!(a1["blockedBy"], json!([]));
    assert_eq!(a1["where"], Value::Null);
    assert_eq!(a1["whereSource"], Value::Null);
    assert_eq!(exported(&b, Coll::Agents, "A-2")["items"], json!([]));
}

#[test]
fn where_is_the_named_worktree_master_or_the_claude_id_folder() {
    let mut b = alpha();
    b["agents"]["A-1"]["worktree"] = json!("X:/p/.claude/worktrees/agent-a1");
    b["agents"]["A-2"]["claudeId"] = json!("ab12");
    let manual = exported(&b, Coll::Agents, "A-1");
    assert_eq!(manual["where"], "X:/p/.claude/worktrees/agent-a1");
    assert_eq!(manual["whereSource"], "manual");
    let auto = exported(&b, Coll::Agents, "A-2");
    assert_eq!(auto["where"], "agent-ab12");
    assert_eq!(auto["whereSource"], "auto");
    b["agents"]["A-2"]["master"] = json!(true);
    let master = exported(&b, Coll::Agents, "A-2");
    assert_eq!(master["where"], "master");
    assert_eq!(master["whereSource"], "manual");
}

#[test]
fn an_item_lists_the_working_agents_that_name_it_and_not_a_finished_one() {
    let mut b = alpha();
    b["agents"]["A-2"]["items"] = json!(["I-1", "I-2"]);
    let item = exported(&b, Coll::Items, "I-1");
    assert_eq!(
        item["agents"],
        json!([{"id": "A-1", "name": "builder", "status": "working"}])
    );
    assert_eq!(exported(&b, Coll::Items, "I-2")["agents"], json!([]));
    b["agents"]["A-2"]["status"] = json!("working");
    let both = exported(&b, Coll::Items, "I-2")["agents"].clone();
    assert_eq!(both[0]["id"], "A-2");
}

#[test]
fn editing_items_keeps_item_as_the_first_and_checks_every_id() {
    let mut b = alpha();
    applied(&edit_agent(&mut b, "A-1", json!({"items": ["I-2", "I-1"]})));
    assert_eq!(b["agents"]["A-1"]["item"], "I-2");
    assert_eq!(b["agents"]["A-1"]["items"], json!(["I-2", "I-1"]));
    let missing = edit_agent(&mut b, "A-1", json!({"items": ["I-99"]}));
    assert_eq!(refused(&missing), "no item \"I-99\".");
    applied(&edit_agent(&mut b, "A-1", json!({"item": "I-1"})));
    assert_eq!(b["agents"]["A-1"]["items"], json!(["I-1"]));
}

#[test]
fn master_and_a_worktree_never_hold_together() {
    let mut b = alpha();
    applied(&edit_agent(&mut b, "A-1", json!({"worktree": "agent-a1"})));
    applied(&edit_agent(&mut b, "A-1", json!({"master": true})));
    assert_eq!(b["agents"]["A-1"]["worktree"], Value::Null);
    applied(&edit_agent(&mut b, "A-1", json!({"worktree": "agent-a1"})));
    assert_eq!(b["agents"]["A-1"]["master"], false);
    let bad = edit_agent(&mut b, "A-1", json!({"master": "yes"}));
    assert_eq!(refused(&bad), "master must be true or false.");
}

#[test]
fn blocked_by_takes_ids_of_the_board_and_never_free_text() {
    let mut b = alpha();
    applied(&edit_agent(
        &mut b,
        "A-1",
        json!({"blockedBy": ["I-2", "A-2"]}),
    ));
    assert_eq!(b["agents"]["A-1"]["blockedBy"], json!(["I-2", "A-2"]));
    let text = edit_agent(&mut b, "A-1", json!({"blockedBy": ["waiting for the key"]}));
    assert!(refused(&text).contains("is not an id"));
    let gone = edit_agent(&mut b, "A-1", json!({"blockedBy": ["A-99"]}));
    assert_eq!(refused(&gone), "A-99 is not on this board.");
}
