//! K1: the real id of every sub-agent.

// test-tag: critical

use crate::hook_support::{Fixture, LEAD};
use serde_json::{Value, json};

fn dispatch(f: &Fixture, description: &str, tool_use_id: &str) {
    let input = json!({ "description": description, "subagent_type": "general-purpose", "model": "sonnet" });
    f.dispatch(LEAD, input, tool_use_id);
}

fn structured(id: &str, description: &str) -> Value {
    json!({ "isAsync": true, "status": "async_launched", "agentId": id, "description": description })
}

fn text_only(id: &str) -> Value {
    let text = format!(
        "Async agent launched successfully. (This tool result is internal metadata)\nagentId: {id} (internal ID - do not mention to user. Use SendMessage with to: '{id}')\nThe agent is working in the background."
    );
    json!({ "content": [{ "type": "text", "text": text }] })
}

/// Removes `claudeId` from the record of a hook id, as a record made before it existed.
fn only_hook_id(f: &Fixture, hook_id: &str) -> String {
    let id = f.by_hook(hook_id).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut board = f.board();
    board["agents"][&id]
        .as_object_mut()
        .unwrap()
        .shift_remove("claudeId");
    f.save(&board);
    id
}

#[test]
fn subagent_start_sets_claude_id_hook_id_and_the_tool_use_id_of_the_dispatch() {
    let f = Fixture::new();
    dispatch(&f, "Write the plan", "toolu_plan");
    f.started(LEAD, "a0000000000000001", "general-purpose");
    let a = f.by_hook("a0000000000000001").unwrap();
    assert_eq!(a["claudeId"], "a0000000000000001");
    assert_eq!(a["toolUseId"], "toolu_plan");
    f.stopped(
        LEAD,
        "a0000000000000001",
        &json!({ "last_assistant_message": "ok" }),
    );
    assert_eq!(
        f.by_hook("a0000000000000001").unwrap()["claudeId"],
        "a0000000000000001"
    );
}

#[test]
fn a_record_with_only_a_hook_id_gets_its_claude_id_when_it_stops() {
    let f = Fixture::new();
    dispatch(&f, "Old style record", "toolu_c1");
    f.started(LEAD, "a0000000000000002", "general-purpose");
    let id = only_hook_id(&f, "a0000000000000002");
    f.stopped(
        LEAD,
        "a0000000000000002",
        &json!({ "last_assistant_message": "ok" }),
    );
    assert_eq!(f.board()["agents"][&id]["claudeId"], "a0000000000000002");
}

#[test]
fn the_agent_id_of_the_tool_response_goes_on_the_record_of_that_tool_use_id() {
    let f = Fixture::new();
    dispatch(&f, "Rebuild the pad overlay", "toolu_pad");
    f.started(LEAD, "a0000000000000003", "general-purpose");
    let id = only_hook_id(&f, "a0000000000000003");
    let ran = f.launched(
        LEAD,
        "toolu_pad",
        structured("a0000000000000003", "Rebuild the pad overlay"),
    );
    assert_eq!((ran.code, ran.out.as_str(), ran.err.as_str()), (0, "", ""));
    let a = &f.board()["agents"][&id];
    assert_eq!(a["claudeId"], "a0000000000000003");
    assert!(a.get("idMismatch").is_none());
}

#[test]
fn an_id_in_the_result_text_only_is_read_too() {
    let f = Fixture::new();
    dispatch(&f, "Text only result", "toolu_text");
    f.started(LEAD, "a0000000000000004", "general-purpose");
    let id = only_hook_id(&f, "a0000000000000004");
    f.launched(LEAD, "toolu_text", text_only("a0000000000000004"));
    assert_eq!(f.board()["agents"][&id]["claudeId"], "a0000000000000004");
}

#[test]
fn a_result_for_another_id_never_rewrites_the_record_that_merely_holds_its_tool_use_id() {
    let f = Fixture::new();
    dispatch(&f, "Contradiction", "toolu_contra");
    f.started(LEAD, "a0000000000000005", "general-purpose");
    f.launched(
        LEAD,
        "toolu_contra",
        structured("a0000000000000055", "Contradiction"),
    );
    let a = f.by_hook("a0000000000000005").unwrap();
    assert_eq!(a["claudeId"], "a0000000000000005");
    assert_eq!(a["hookId"], "a0000000000000005");
    assert!(a.get("idMismatch").is_none());
    let notes = f.board()["agentLaunches"].clone();
    assert!(
        notes
            .as_array()
            .unwrap()
            .iter()
            .any(|l| l["claudeId"] == "a0000000000000055")
    );
}

#[test]
fn a_record_of_the_same_agent_whose_two_ids_disagree_is_flagged_and_the_result_wins() {
    let f = Fixture::new();
    dispatch(&f, "Same agent", "toolu_same");
    f.started(LEAD, "a0000000000000008", "general-purpose");
    let id = f.by_hook("a0000000000000008").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let mut board = f.board();
    board["agents"][&id]["claudeId"] = json!("a0000000000000088");
    f.save(&board);
    f.launched(
        LEAD,
        "toolu_same",
        structured("a0000000000000008", "Same agent"),
    );
    let a = &f.board()["agents"][&id];
    assert_eq!(a["claudeId"], "a0000000000000008");
    assert_eq!(a["idMismatch"], true);
}

#[test]
fn other_tools_unknown_sessions_and_results_with_no_id_change_nothing() {
    let f = Fixture::new();
    let before = f.raw();
    let bash = json!({
        "session_id": LEAD, "cwd": f.cwd_text(), "hook_event_name": "PostToolUse",
        "tool_name": "Bash", "tool_use_id": "x", "tool_response": { "agentId": "a0000000000000006" },
    });
    f.hook("agent-result", &bash);
    f.launched(LEAD, "toolu_none", structured("", "nothing"));
    f.launched(
        "sid-stranger-1",
        "toolu_stranger",
        structured("a0000000000000007", "x"),
    );
    assert_eq!(f.raw(), before);
}

#[test]
fn with_no_record_yet_the_id_waits_in_a_launch_note() {
    let f = Fixture::new();
    dispatch(&f, "R1: restore the apworld on master", "toolu_note");
    f.launched(
        LEAD,
        "toolu_note",
        json!({ "agentId": "a0000000000000011" }),
    );
    let board = f.board();
    let note = board["agentLaunches"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .clone();
    assert_eq!(note["claudeId"], "a0000000000000011");
    assert_eq!(note["thread"], "lead");
    assert_eq!(note["toolUseId"], "toolu_note");
    assert_eq!(
        note["description"], "R1: restore the apworld on master",
        "from the dispatch"
    );
    f.launched(
        LEAD,
        "toolu_note",
        structured("a0000000000000011", "Told by the result"),
    );
    let board = f.board();
    let notes = board["agentLaunches"].as_array().unwrap();
    assert_eq!(notes.len(), 1, "one note per id");
    assert_eq!(notes[0]["description"], "Told by the result");
}
