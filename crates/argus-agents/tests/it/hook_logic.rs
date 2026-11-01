//! The pure rules of the sub-agent hooks that live on the dashboard side: the path of a sub-agent
//! transcript, the agent id Claude prints, the model a transcript shows, what a finished agent
//! leaves on its record, and how a hand-made record finds its hook record. The pure part of
//! `hook.transcript`, `hook.agents.claudeid`, `hook.agents` and `hook.agents.binding`; the cases
//! that run the Node hook script in a child process belong to the coord port.

use argus_agents::StdFs;
use argus_agents::fakes::{assistant_line, text_block};
use argus_agents::hook_logic::{
    close_record, launched_id, subagent_transcript, transcript_model, valid_claude_id,
};
use argus_agents::record::Record;
use argus_agents::text::{best_fit, overlap, same_model};
use serde_json::{Value, json};
use std::path::Path;

const SID: &str = "sid-lead-0001";

fn record(value: &Value) -> Record {
    value.as_object().cloned().unwrap()
}

#[test]
fn the_sub_agent_transcript_is_stored_next_to_the_parent_transcript() {
    let parent = Path::new("tmp")
        .join("projects")
        .join("x--ai-config")
        .join(format!("{SID}.jsonl"));
    let got = subagent_transcript(parent.to_str(), Some(SID), "a1b2c3d4e5").unwrap();
    let expected = Path::new("tmp")
        .join("projects")
        .join("x--ai-config")
        .join(SID)
        .join("subagents")
        .join("agent-a1b2c3d4e5.jsonl");
    assert_eq!(got, expected);
}

#[test]
fn without_a_transcript_path_or_a_session_there_is_no_path() {
    assert_eq!(subagent_transcript(None, Some(SID), "plain-0001"), None);
    assert_eq!(
        subagent_transcript(Some("tmp/p/x.jsonl"), None, "plain-0001"),
        None
    );
}

#[test]
fn an_agent_id_with_path_characters_gets_no_path() {
    let parent = Path::new("tmp").join("p").join(format!("{SID}.jsonl"));
    assert_eq!(
        subagent_transcript(parent.to_str(), Some(SID), "../../evil"),
        None
    );
    assert_eq!(
        subagent_transcript(parent.to_str(), Some("../sid"), "ok-id"),
        None
    );
}

#[test]
fn the_agent_id_is_read_from_the_structured_tool_response() {
    let response = json!({ "isAsync": true, "status": "async_launched", "agentId": "a0000000000000003", "description": "Rebuild the pad overlay" });
    assert_eq!(launched_id(&response).as_deref(), Some("a0000000000000003"));
    assert_eq!(
        launched_id(&json!({ "agent_id": "a0000000000000009" })).as_deref(),
        Some("a0000000000000009")
    );
}

#[test]
fn an_id_in_the_result_text_only_is_read_too() {
    let text = "Async agent launched successfully. (This tool result is internal metadata)\nagentId: a0000000000000004 (internal ID - do not mention to user. Use SendMessage with to: 'a0000000000000004')\nThe agent is working in the background.";
    let response = json!({ "content": [{ "type": "text", "text": text }] });
    assert_eq!(launched_id(&response).as_deref(), Some("a0000000000000004"));
    assert_eq!(
        launched_id(&json!(text)).as_deref(),
        Some("a0000000000000004")
    );
}

#[test]
fn a_result_with_no_id_or_an_odd_one_gives_none() {
    assert_eq!(launched_id(&json!({ "agentId": "" })), None);
    assert_eq!(launched_id(&json!(null)), None);
    assert_eq!(
        launched_id(&json!({ "content": [{ "type": "text", "text": "nothing here" }] })),
        None
    );
    assert!(!valid_claude_id("no spaces/allowed"));
    assert!(valid_claude_id("a0000000000000013"));
    assert!(!valid_claude_id(&"x".repeat(121)));
}

fn transcript_with(model: &str, later: Option<&str>) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("t.jsonl");
    let mut lines = assistant_line(1, &json!([text_block("hi")]), &json!({ "model": model }));
    if let Some(later) = later {
        lines += &assistant_line(2, &json!([text_block("again")]), &json!({ "model": later }));
    }
    std::fs::write(&file, lines).unwrap();
    (dir, file)
}

#[test]
fn stop_closes_the_record_with_the_answer_clipped_to_300_and_the_model_from_the_transcript() {
    let (_dir, file) = transcript_with("claude-haiku-4-5", Some("claude-sonnet-5-5"));
    let used = transcript_model(&StdFs, &file);
    assert_eq!(
        used.as_deref(),
        Some("claude-haiku-4-5"),
        "the first assistant line"
    );
    let mut agent = record(&json!({ "status": "working", "model": "sonnet" }));
    close_record(
        &mut agent,
        &"x".repeat(400),
        used.as_deref(),
        1_792_756_800_000,
    );
    assert_eq!(agent["status"], json!("done"));
    assert_eq!(agent["result"].as_str().unwrap().chars().count(), 300);
    assert!(agent["finishedAt"].as_str().unwrap().starts_with("2026-"));
    assert_eq!(agent["model"], json!("sonnet"), "the requested model stays");
    assert_eq!(agent["modelUsed"], json!("claude-haiku-4-5"));
    assert_eq!(agent["modelMismatch"], json!(true));
}

#[test]
fn a_matching_model_is_not_a_mismatch_and_an_inherited_model_is_filled_from_the_transcript() {
    let mut matching = record(&json!({ "status": "working", "model": "sonnet" }));
    close_record(&mut matching, "built", Some("claude-sonnet-5-5"), 0);
    assert_eq!(matching["modelMismatch"], json!(false));
    let mut inherited = record(&json!({ "status": "working", "model": null }));
    close_record(&mut inherited, "looked", Some("claude-opus-4-5"), 0);
    assert_eq!(inherited["model"], json!("claude-opus-4-5"));
    assert_eq!(inherited["modelUsed"], json!("claude-opus-4-5"));
    assert!(!inherited.contains_key("modelMismatch"));
}

#[test]
fn stop_with_a_missing_or_unreadable_transcript_still_closes_the_record() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(
        transcript_model(&StdFs, &dir.path().join("does-not-exist.jsonl")),
        None
    );
    assert_eq!(transcript_model(&StdFs, dir.path()), None);
    let mut agent = record(&json!({ "status": "working" }));
    close_record(&mut agent, "done anyway", None, 0);
    assert_eq!(agent["status"], json!("done"));
    assert_eq!(agent["result"], json!("done anyway"));
    assert!(!agent.contains_key("modelUsed"));
}

#[test]
fn a_last_message_that_starts_with_api_error_closes_the_record_as_failed() {
    let mut agent = record(&json!({ "status": "working" }));
    close_record(&mut agent, "API Error: 529 overloaded", None, 0);
    assert_eq!(agent["status"], json!("failed"));
    let mut other = record(&json!({ "status": "working" }));
    close_record(&mut other, "  error: bad", None, 0);
    assert_eq!(other["status"], json!("failed"));
}

#[test]
fn a_record_the_reaper_closed_is_corrected_by_a_late_stop_and_a_finished_one_is_left_alone() {
    let mut reaped = record(
        &json!({ "status": "done", "reaped": true, "note": "no activity for 25 min, closed automatically" }),
    );
    close_record(&mut reaped, "the real answer", None, 0);
    assert_eq!(reaped["result"], json!("the real answer"));
    assert!(!reaped.contains_key("reaped"));
    assert_eq!(reaped["note"], json!(null));
    let mut finished = record(&json!({ "status": "done", "result": "first" }));
    close_record(&mut finished, "second", Some("claude-haiku-4-5"), 0);
    assert_eq!(finished["result"], json!("first"));
    assert_eq!(finished["modelUsed"], json!("claude-haiku-4-5"));
}

#[test]
fn a_hand_made_record_finds_the_hook_record_whose_description_reads_like_it() {
    let hooked = [
        "r1:-restore-the-apworld-on-master",
        "rebuild-settings-screen",
    ];
    let manual = "apworld-r1 R1: restore apworld package, reproducible build, player-yaml, save flow on master";
    let found = best_fit(manual, &hooked, |name| (*name).to_owned(), 1);
    assert_eq!(found, Some(&"r1:-restore-the-apworld-on-master"));
    assert_eq!(
        best_fit(manual, &hooked, |name| (*name).to_owned(), 5),
        None
    );
}

#[test]
fn the_oldest_candidate_wins_a_tie_and_a_text_with_no_words_matches_only_a_single_candidate() {
    let both = [("oldest", "lobby client"), ("newest", "lobby client")];
    let text_of = |c: &(&str, &str)| c.1.to_owned();
    assert_eq!(
        best_fit("lobby client", &both, text_of, 1).map(|c| c.0),
        Some("oldest")
    );
    assert_eq!(best_fit("-- --", &both, text_of, 1), None);
    assert_eq!(
        best_fit("-- --", &both[..1], text_of, 1).map(|c| c.0),
        Some("oldest")
    );
}

#[test]
fn overlap_counts_shared_words_without_the_filler_and_model_names_match_either_way() {
    assert_eq!(
        overlap(
            "Rebuild the Controllers screen",
            "rebuild controllers to approved design"
        ),
        2
    );
    assert_eq!(overlap("a an of to", "a an of to"), 0);
    assert!(same_model("sonnet", "claude-sonnet-5-5"));
    assert!(same_model("claude-opus-5-5", "Opus"));
    assert!(!same_model("opus", "claude-haiku-4-5"));
}
