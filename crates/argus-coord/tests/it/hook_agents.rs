//! K1: the sub-agent records the hooks make.

// test-tag: critical

use crate::hook_support::{FRESH, Fixture, LEAD, STRANGER, UI, ago, transcript, wire};
use argus_core::clock::{now_ms, parse_ms};
use serde_json::{Value, json};

fn quiet(ran: &crate::common::Ran) {
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert_eq!(ran.out, "");
    assert_eq!(ran.err, "");
}

fn dispatches(f: &Fixture) -> Vec<Value> {
    f.board()["agentDispatches"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[test]
fn a_dispatch_is_stored_for_a_registered_thread_silently() {
    let f = Fixture::new();
    let input = json!({
        "description": "Run the test suite", "prompt": "p".repeat(500),
        "subagent_type": "general-purpose", "model": "haiku", "run_in_background": true,
    });
    quiet(&f.dispatch(LEAD, input, "toolu_1"));
    let list = dispatches(&f);
    assert_eq!(list.len(), 1);
    let d = &list[0];
    assert_eq!(d["thread"], "coordinator");
    assert_eq!(d["toolUseId"], "toolu_1");
    assert_eq!(d["description"], "Run the test suite");
    assert_eq!(d["promptHead"].as_str().unwrap().len(), 300);
    assert_eq!(d["type"], "general-purpose");
    assert_eq!(d["model"], "haiku");
    assert_eq!(d["background"], true);
    assert!(parse_ms(d["at"].as_str().unwrap()).is_some());
}

#[test]
fn another_tool_is_ignored_and_a_missing_model_is_null() {
    let f = Fixture::new();
    let before = f.raw();
    quiet(&f.dispatch_tool(LEAD, "Bash", json!({ "description": "x" }), "toolu_2"));
    assert_eq!(f.raw(), before, "the board is not even saved");
    quiet(&f.dispatch_tool(
        LEAD,
        "Task",
        json!({ "description": "Explore the repo", "prompt": "look" }),
        "toolu_3",
    ));
    let list = dispatches(&f);
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["model"], Value::Null);
    assert_eq!(list[0]["type"], Value::Null);
    assert_eq!(list[0]["background"], false);
}

#[test]
fn dispatches_older_than_ten_minutes_go_and_the_list_keeps_twenty() {
    let f = Fixture::new();
    let mut board = f.board();
    let mut list = vec![json!({ "thread": "coordinator", "description": "old", "at": ago(11) })];
    for i in 0..22 {
        list.push(json!({ "thread": "coordinator", "description": format!("d{i}"), "at": ago(0) }));
    }
    board["agentDispatches"] = Value::Array(list);
    f.save(&board);
    f.dispatch(LEAD, json!({ "description": "newest" }), "toolu_4");
    let list = dispatches(&f);
    assert_eq!(list.len(), 20);
    assert!(!list.iter().any(|d| d["description"] == "old"));
    assert_eq!(list.last().unwrap()["description"], "newest");
}

#[test]
fn dispatch_then_start_makes_one_record_with_the_dispatch_fields() {
    let f = Fixture::new();
    let input = json!({
        "description": "Run the test suite", "prompt": "run npm test and report",
        "subagent_type": "general-purpose", "model": "sonnet",
    });
    f.dispatch(LEAD, input, "toolu_5");
    quiet(&f.started(LEAD, "agent-abcdef-1234", "general-purpose"));
    let board = f.board();
    assert_eq!(board["agents"].as_object().unwrap().len(), 3);
    let a = f.by_hook("agent-abcdef-1234").expect("the record");
    assert_eq!(a["id"], "A-3");
    assert_eq!(a["thread"], "coordinator");
    assert_eq!(a["name"], "run-the-test-suite");
    assert_eq!(a["task"], "Run the test suite");
    assert_eq!(a["model"], "sonnet");
    assert_eq!(a["modelWhy"], Value::Null);
    assert_eq!(a["kind"], "test");
    assert_eq!(a["status"], "working");
    assert_eq!(a["source"], "hook");
    assert_eq!(a["finishedAt"], Value::Null);
    assert_eq!(a["toolUseId"], "toolu_5");
    assert_eq!(dispatches(&f).len(), 0, "the dispatch was consumed");
    f.started(LEAD, "agent-abcdef-1234", "general-purpose");
    assert_eq!(
        f.board()["agents"].as_object().unwrap().len(),
        3,
        "the same start again adds nothing"
    );
}

#[test]
fn without_a_dispatch_the_name_is_the_type_and_the_last_four_of_the_id() {
    let f = Fixture::new();
    f.started(LEAD, "agent-zzzz-9f3c", "Explore");
    let a = f.by_hook("agent-zzzz-9f3c").unwrap();
    assert_eq!(a["name"], "explore-9f3c");
    assert_eq!(a["model"], Value::Null);
    assert_eq!(a["kind"], "research", "from the word Explore");
    assert_eq!(a["agentType"], "Explore");
}

#[test]
fn a_record_of_the_lead_with_the_exact_claude_id_is_the_record_of_that_start() {
    let f = Fixture::new();
    let claude = json!({ "claudeId": "agent-bound-0001" });
    let id = f.manual("A-9", "fixer", "fix the thing", &claude);
    let before = f.board()["agents"][&id].clone();
    f.started(LEAD, "agent-bound-0001", "general-purpose");
    let board = f.board();
    assert_eq!(
        board["agents"].as_object().unwrap().len(),
        3,
        "no second record"
    );
    assert_eq!(
        board["agents"][&id], before,
        "live, with its id: nothing to change"
    );
    quiet(&f.stopped(
        LEAD,
        "agent-bound-0001",
        &json!({ "last_assistant_message": "fixed" }),
    ));
    let a = &f.board()["agents"][&id];
    assert_eq!(
        (a["status"].as_str(), a["result"].as_str()),
        (Some("done"), Some("fixed"))
    );
    assert_eq!(a["source"], Value::Null, "a manual record stays manual");
}

#[test]
fn stop_closes_with_the_answer_clipped_and_the_model_of_the_transcript() {
    let f = Fixture::new();
    f.dispatch(
        LEAD,
        json!({ "description": "Run the test suite", "model": "sonnet" }),
        "t7",
    );
    f.started(LEAD, "agent-abcdef-1234", "general-purpose");
    let file = transcript(
        f.home.path(),
        "t-haiku.jsonl",
        &["claude-haiku-4-5", "claude-sonnet-5-5"],
    );
    let extra = json!({ "agent_transcript_path": file, "last_assistant_message": "x".repeat(400) });
    quiet(&f.stopped(LEAD, "agent-abcdef-1234", &extra));
    let a = f.by_hook("agent-abcdef-1234").unwrap();
    assert_eq!(a["status"], "done");
    assert_eq!(a["result"].as_str().unwrap().len(), 300);
    assert!(parse_ms(a["finishedAt"].as_str().unwrap()).unwrap() <= now_ms());
    assert_eq!(a["model"], "sonnet", "the requested model stays");
    assert_eq!(a["modelUsed"], "claude-haiku-4-5");
    assert_eq!(a["modelMismatch"], true);
}

#[test]
fn a_matching_model_is_no_mismatch_and_an_inherited_one_is_filled_in() {
    let f = Fixture::new();
    f.dispatch(
        LEAD,
        json!({ "description": "Build the form", "model": "sonnet" }),
        "t8",
    );
    f.started(LEAD, "agent-match-0002", "general-purpose");
    let sonnet = transcript(f.home.path(), "t-sonnet.jsonl", &["claude-sonnet-5-5"]);
    let extra = json!({ "agent_transcript_path": sonnet, "last_assistant_message": "built" });
    f.stopped(LEAD, "agent-match-0002", &extra);
    let a = f.by_hook("agent-match-0002").unwrap();
    assert_eq!(a["modelUsed"], "claude-sonnet-5-5");
    assert_eq!(a["modelMismatch"], false);
    f.started(LEAD, "agent-zzzz-9f3c", "Explore");
    let opus = transcript(f.home.path(), "t-inh.jsonl", &["claude-opus-4-5"]);
    let extra = json!({ "agent_transcript_path": opus, "last_assistant_message": "looked" });
    f.stopped(LEAD, "agent-zzzz-9f3c", &extra);
    let inherited = f.by_hook("agent-zzzz-9f3c").unwrap();
    assert_eq!(inherited["model"], "claude-opus-4-5");
    assert_eq!(inherited["modelUsed"], "claude-opus-4-5");
    assert_ne!(inherited["modelMismatch"], true);
}

#[test]
fn a_missing_or_unreadable_transcript_still_closes_the_record() {
    let f = Fixture::new();
    f.started(LEAD, "agent-gone-0003", "general-purpose");
    let missing = f.home.path().join("does-not-exist.jsonl");
    let extra =
        json!({ "agent_transcript_path": missing, "last_assistant_message": "done anyway" });
    quiet(&f.stopped(LEAD, "agent-gone-0003", &extra));
    let a = f.by_hook("agent-gone-0003").unwrap();
    assert_eq!(a["status"], "done");
    assert_eq!(a["result"], "done anyway");
    assert_eq!(a["modelUsed"], Value::Null);
    f.started(LEAD, "agent-dir-0004", "general-purpose");
    let dir = f.home.path().to_string_lossy().into_owned();
    f.stopped(
        LEAD,
        "agent-dir-0004",
        &json!({ "agent_transcript_path": dir }),
    );
    assert_eq!(f.by_hook("agent-dir-0004").unwrap()["status"], "done");
}

#[test]
fn a_last_message_starting_with_api_error_closes_the_record_as_failed() {
    let f = Fixture::new();
    f.started(LEAD, "agent-err-0005", "general-purpose");
    let extra = json!({ "last_assistant_message": "API Error: 529 overloaded" });
    f.stopped(LEAD, "agent-err-0005", &extra);
    assert_eq!(f.by_hook("agent-err-0005").unwrap()["status"], "failed");
}

#[test]
fn a_stop_for_an_unknown_agent_or_with_no_body_adds_no_record() {
    let f = Fixture::new();
    let agents = f.board()["agents"].clone();
    quiet(&f.stopped(LEAD, "agent-never-seen", &json!({})));
    quiet(&f.hook("subagent-stop", &json!({})));
    let no_id = json!({ "session_id": LEAD, "cwd": f.cwd_text() });
    quiet(&f.hook("subagent-start", &no_id));
    let board = f.board();
    assert_eq!(board["agents"], agents);
    let stops: Vec<&Value> = board["agentStops"].as_array().unwrap().iter().collect();
    assert_eq!(
        stops.len(),
        1,
        "only a note for a start that may still come"
    );
    assert_eq!(stops[0]["hookId"], "agent-never-seen");
    assert_eq!(stops[0]["thread"], "coordinator");
    assert_eq!(stops[0]["used"], Value::Null);
}

#[test]
fn an_unknown_session_gets_nothing_and_the_board_file_is_untouched() {
    let f = Fixture::new();
    let before = f.raw();
    for ran in [
        f.dispatch(STRANGER, json!({ "description": "x" }), "t9"),
        f.started(STRANGER, "agent-stranger-1", "general-purpose"),
        f.stopped(STRANGER, "agent-stranger-1", &json!({})),
    ] {
        quiet(&ran);
    }
    assert_eq!(f.raw(), before);
}

#[test]
fn a_placeholder_thread_gets_nothing() {
    let f = Fixture::new();
    let start = json!({ "session_id": FRESH, "cwd": f.cwd_text() });
    assert_eq!(f.hook("session-start", &start).code, 0);
    let board = f.board();
    let adopted = board["threads"]
        .as_object()
        .unwrap()
        .values()
        .any(|t| t["placeholder"] == true && t["sessionId"] == FRESH);
    assert!(adopted, "adopted as a placeholder");
    let before = f.raw();
    f.dispatch(FRESH, json!({ "description": "x" }), "t10");
    f.started(FRESH, "agent-placeholder-1", "general-purpose");
    f.stopped(FRESH, "agent-placeholder-1", &json!({}));
    assert_eq!(f.raw(), before);
}

#[test]
fn a_registered_thread_that_is_not_the_lead_gets_its_own_record() {
    let f = Fixture::new();
    f.started(UI, "agent-ui-0006", "general-purpose");
    assert_eq!(f.by_hook("agent-ui-0006").unwrap()["thread"], "ui");
}

#[test]
fn a_project_with_no_board_gets_nothing_and_exits_0() {
    let f = Fixture::new();
    let ghost = f.home.path().join("checkout").join("ghost");
    wire(&ghost, "ghost");
    let input = json!({
        "session_id": "sid-nobody", "cwd": ghost, "agent_id": "agent-ghost-1", "agent_type": "x",
    });
    quiet(&f.hook("subagent-start", &input));
    assert!(!f.home.coord_home().join("ghost").exists());
}
