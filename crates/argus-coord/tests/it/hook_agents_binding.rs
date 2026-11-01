//! Ports the hook side of `hook.agents.binding.test.mjs`: one sub-agent, one record. A hook
//! event belongs to the record of its own agent id and to no other: a start never takes a
//! record of the lead however alike it reads, a stop always finds the record of its id (before
//! its start, under another session id, after the dashboard reaped it) and only notes an id it
//! has no record of, a start for a known id reopens a closed record, and an Agent call result
//! never rewrites the record of another agent. The cases where `coord agent start` binds to a
//! hook record belong to the agent verbs (WP5).

use std::fs;

use crate::hook_support::{Fixture, LEAD, STRANGER, ago};
use serde_json::{Value, json};

fn dispatch(f: &Fixture, description: &str, model: &str, tool_use_id: &str) {
    let input = json!({
        "description": description, "prompt": format!("{description}. Work in the worktree."),
        "subagent_type": "general-purpose", "model": model,
    });
    assert_eq!(f.dispatch(LEAD, input, tool_use_id).code, 0);
}

fn last(text: &str) -> Value {
    json!({ "last_assistant_message": text })
}

fn id_of(f: &Fixture, hook_id: &str) -> String {
    f.by_hook(hook_id).expect("a record")["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn agent_count(f: &Fixture) -> usize {
    f.board()["agents"].as_object().unwrap().len()
}

#[test]
fn a_hook_start_never_takes_a_record_of_the_lead_however_alike_it_reads() {
    let f = Fixture::new();
    let task = "R3: lobby client vs Archipelia fake server, LAN discovery";
    let lobby = f.manual("A-10", "lobby-r3", task, &json!({}));
    dispatch(&f, "Rebuild the pad overlay screen", "sonnet", "toolu_b1");
    f.started(LEAD, "pad-0002", "general-purpose");
    dispatch(
        &f,
        "R3: build the Archipelia lobby client",
        "sonnet",
        "toolu_b2",
    );
    f.started(LEAD, "adeccacd42e92e27b", "general-purpose");
    assert!(
        f.board()["agents"][&lobby].get("hookId").is_none(),
        "only an exact id binds"
    );
    assert!(
        f.by_hook("adeccacd42e92e27b").is_some(),
        "the hook made its own record"
    );
    f.stopped(LEAD, "adeccacd42e92e27b", &last("lobby built"));
    f.stopped(LEAD, "pad-0002", &last("pad done"));
    let status = &f.board()["agents"][&lobby]["status"];
    assert_eq!(
        status, "working",
        "a stop of another agent never closes the lead record"
    );
}

#[test]
fn b_stops_and_its_answer_lands_on_b_while_a_stays_a_running_sonnet_record() {
    let f = Fixture::new();
    let (a_id, b_id) = ("aaaaaaaaaaaa0001a", "bbbbbbbbbbbb0002b");
    let mine = f.manual(
        "A-10",
        "engine-tests",
        "Run the engine tests and report",
        &json!({}),
    );
    dispatch(
        &f,
        "Write the sonnet part of the engine",
        "sonnet",
        "toolu_e1",
    );
    f.started(LEAD, a_id, "general-purpose");
    dispatch(&f, "Run the engine tests", "haiku", "toolu_e2");
    f.started(LEAD, b_id, "general-purpose");
    let (a, b) = (id_of(&f, a_id), id_of(&f, b_id));
    assert_ne!(a, b);
    let transcript = f
        .home
        .path()
        .join("projects")
        .join(format!("agent-{b_id}.jsonl"));
    let extra =
        json!({ "agent_transcript_path": transcript, "last_assistant_message": "tests pass" });
    f.stopped(LEAD, b_id, &extra);
    let after = f.board()["agents"].clone();
    assert_eq!(
        (after[&b]["status"].as_str(), after[&b]["result"].as_str()),
        (Some("done"), Some("tests pass"))
    );
    assert_eq!(after[&a]["status"], "working", "A is still running");
    assert_eq!(
        (after[&a]["result"].clone(), after[&a]["finishedAt"].clone()),
        (Value::Null, Value::Null)
    );
    assert_eq!(after[&a]["model"], "sonnet");
    assert_eq!(after[&mine]["status"], "working");
    assert!(
        after[&mine].get("transcriptFile").is_none(),
        "no transcript of another agent"
    );
    f.stopped(LEAD, a_id, &last("engine written"));
    let after = f.board()["agents"].clone();
    assert_eq!(after[&a]["result"], "engine written");
    assert_eq!(
        after[&b]["result"], "tests pass",
        "the stop of A leaves B alone"
    );
}

#[test]
fn a_payload_whose_agent_id_and_transcript_name_disagree_touches_nothing() {
    let f = Fixture::new();
    f.started(LEAD, "aaaaaaaaaaaa0001a", "general-purpose");
    let before = f.raw();
    let other = f.home.path().join("agent-bbbbbbbbbbbb0002b.jsonl");
    let extra = json!({ "agent_transcript_path": other, "last_assistant_message": "confused" });
    assert_eq!(f.stopped(LEAD, "aaaaaaaaaaaa0001a", &extra).code, 0);
    assert_eq!(f.raw(), before, "not even saved");
}

#[test]
fn a_start_for_a_known_id_creates_nothing_and_reopens_a_record_closed_for_silence() {
    let f = Fixture::new();
    dispatch(&f, "Slow opus pass", "opus", "toolu_s1");
    f.started(LEAD, "slowslow0000003", "general-purpose");
    let id = id_of(&f, "slowslow0000003");
    let mut board = f.board();
    let closed = json!({ "status": "done", "finishedAt": ago(0), "note": "no activity for 21 min, closed automatically", "reaped": true, "closedBy": "reaper" });
    for (k, v) in closed.as_object().unwrap() {
        board["agents"][&id][k] = v.clone();
    }
    f.save(&board);
    let count = agent_count(&f);
    f.started(LEAD, "slowslow0000003", "general-purpose");
    let a = f.board()["agents"][&id].clone();
    assert_eq!(agent_count(&f), count, "no second record");
    assert_eq!(
        (a["status"].as_str(), a["finishedAt"].clone()),
        (Some("working"), Value::Null)
    );
    assert!(
        a["note"]
            .as_str()
            .unwrap()
            .contains("reopened automatically")
    );
    assert!(a.get("reaped").is_none() && a.get("closedBy").is_none());
    f.stopped(LEAD, "slowslow0000003", &last("finally"));
    assert_eq!(f.board()["agents"][&id]["status"], "done");
}

#[test]
fn an_archived_record_of_the_id_is_left_alone_and_no_new_record_is_made() {
    let f = Fixture::new();
    f.started(LEAD, "archived00000004", "general-purpose");
    let id = id_of(&f, "archived00000004");
    let mut board = f.board();
    board["agents"][&id]["status"] = json!("done");
    board["agents"][&id]["archivedAt"] = json!(ago(0));
    f.save(&board);
    let count = agent_count(&f);
    f.started(LEAD, "archived00000004", "general-purpose");
    assert_eq!(agent_count(&f), count);
    assert_eq!(f.board()["agents"][&id]["status"], "done");
}

#[test]
fn the_agent_call_result_never_rewrites_a_record_of_another_agent_with_its_tool_use_id() {
    let f = Fixture::new();
    dispatch(&f, "First dispatch", "opus", "toolu_cross_1");
    dispatch(&f, "Second dispatch", "haiku", "toolu_cross_2");
    f.started(LEAD, "crosscross000005", "general-purpose");
    let id = id_of(&f, "crosscross000005");
    assert_eq!(
        f.board()["agents"][&id]["toolUseId"],
        "toolu_cross_1",
        "the oldest dispatch"
    );
    let response =
        json!({ "isAsync": true, "agentId": "otherother000006", "description": "First dispatch" });
    f.launched(LEAD, "toolu_cross_1", response);
    let board = f.board();
    assert_eq!(board["agents"][&id]["claudeId"], "crosscross000005");
    assert!(board["agents"][&id].get("idMismatch").is_none());
    let noted = board["agentLaunches"].as_array().unwrap();
    assert!(
        noted.iter().any(|l| l["claudeId"] == "otherother000006"),
        "noted as a launch"
    );
}

#[test]
fn a_stop_that_arrives_before_its_start_closes_the_record_the_start_makes() {
    let f = Fixture::new();
    let ran = f.stopped(LEAD, "quick-0004", &last("answered at once"));
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
    assert!(f.by_hook("quick-0004").is_none());
    assert_eq!(f.board()["agentStops"][0]["hookId"], "quick-0004");
    dispatch(&f, "Look up the port", "haiku", "toolu_b5");
    f.started(LEAD, "quick-0004", "general-purpose");
    let a = f.by_hook("quick-0004").unwrap();
    assert_eq!(
        (a["status"].as_str(), a["result"].as_str()),
        (Some("done"), Some("answered at once"))
    );
    assert!(a["finishedAt"].is_string());
    assert_eq!(f.board()["agentStops"], json!([]), "the note is used up");
}

#[test]
fn a_stop_under_another_session_id_than_the_start_still_closes_the_record() {
    let f = Fixture::new();
    dispatch(&f, "Check the long build", "sonnet", "toolu_b6");
    f.started(LEAD, "resumed-0005", "general-purpose");
    let mut board = f.board();
    board["threads"]["coordinator"]["sessionId"] = json!("sid-lead-resumed-9999");
    f.save(&board);
    assert_eq!(
        f.stopped(LEAD, "resumed-0005", &last("build checked")).code,
        0
    );
    let a = f.by_hook("resumed-0005").unwrap();
    assert_eq!(
        (a["status"].as_str(), a["result"].as_str()),
        (Some("done"), Some("build checked"))
    );
}

#[test]
fn a_stop_for_an_unknown_id_from_a_stranger_session_is_dropped() {
    let f = Fixture::new();
    let before = f.raw();
    assert_eq!(f.stopped(STRANGER, "stranger-0006", &last("x")).code, 0);
    assert_eq!(f.raw(), before);
}

#[test]
fn a_stop_for_an_id_with_no_record_is_only_noted_however_alike_a_lead_record_reads() {
    let f = Fixture::new();
    let task = "Rebuild Controllers to approved wireframe: live nav, no nested boxes";
    let mine = f.manual("A-13", "controllers-opus", task, &json!({ "item": "I-2" }));
    let dir = f.home.path();
    let file = dir.join("agent-controllers0008.jsonl");
    fs::write(&file, "").unwrap();
    let meta = json!({ "agentType": "general-purpose", "description": "Rebuild Controllers screen to approved design" });
    fs::write(
        dir.join("agent-controllers0008.meta.json"),
        meta.to_string(),
    )
    .unwrap();
    let extra = json!({ "agent_transcript_path": file, "last_assistant_message": "rebuilt" });
    f.stopped(LEAD, "controllers0008", &extra);
    let board = f.board();
    assert_eq!(board["agents"][&mine]["status"], "working");
    assert!(board["agents"][&mine].get("hookId").is_none());
    let stops = board["agentStops"].as_array().unwrap();
    assert!(
        stops.iter().any(|s| s["hookId"] == "controllers0008"),
        "noted for a later start"
    );
}

#[test]
fn a_record_the_dashboard_reaped_is_corrected_when_its_real_stop_arrives() {
    let f = Fixture::new();
    dispatch(&f, "Slow research pass", "sonnet", "toolu_b7");
    f.started(LEAD, "slow-0009", "general-purpose");
    let id = id_of(&f, "slow-0009");
    let mut board = f.board();
    let a = &mut board["agents"][&id];
    a["status"] = json!("done");
    a["finishedAt"] = json!(ago(0));
    a["note"] = json!("no activity for 21 min, closed automatically");
    a["reaped"] = json!(true);
    a["closedBy"] = json!("reaper");
    f.save(&board);
    f.stopped(LEAD, "slow-0009", &last("API Error: 529 overloaded"));
    let after = f.by_hook("slow-0009").unwrap();
    assert_eq!(after["status"], "failed");
    assert_eq!(after["result"], "API Error: 529 overloaded");
    assert_eq!(after["note"], Value::Null);
    assert!(after.get("reaped").is_none() && after.get("closedBy").is_none());
}

#[test]
fn the_same_tool_call_announced_twice_is_one_dispatch() {
    let f = Fixture::new();
    dispatch(&f, "Count the twins", "sonnet", "toolu_twin");
    dispatch(&f, "Count the twins", "sonnet", "toolu_twin");
    assert_eq!(f.board()["agentDispatches"].as_array().unwrap().len(), 1);
    f.started(LEAD, "twin-0010", "general-purpose");
    assert_eq!(f.board()["agentDispatches"], json!([]), "consumed once");
}
