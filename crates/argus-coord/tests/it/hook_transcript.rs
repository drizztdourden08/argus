//! Ports `hook.transcript.test.mjs`: `coord hook subagent-start` stores where the sub-agent
//! writes its own transcript, from the parent session's `transcript_path`, `session_id` and
//! `agent_id`: `<dir>/<sid>/subagents/agent-<agentId>.jsonl`.

use crate::hook_support::{Fixture, LEAD};
use serde_json::{Value, json};

fn start(f: &Fixture, agent_id: &str, parent: Option<&std::path::Path>) -> crate::common::Ran {
    let mut input = json!({
        "session_id": LEAD, "cwd": f.cwd_text(), "hook_event_name": "SubagentStart",
        "agent_id": agent_id, "agent_type": "general-purpose",
    });
    if let Some(parent) = parent {
        input["transcript_path"] = json!(parent);
    }
    f.hook("subagent-start", &input)
}

#[test]
fn the_transcript_path_is_stored_next_to_the_parent_transcript() {
    let f = Fixture::new();
    let dir = f.home.path().join("projects").join("x--ai-config");
    let ran = start(&f, "a1b2c3d4e5", Some(&dir.join(format!("{LEAD}.jsonl"))));
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
    let expected = dir
        .join(LEAD)
        .join("subagents")
        .join("agent-a1b2c3d4e5.jsonl");
    assert_eq!(
        f.by_hook("a1b2c3d4e5").unwrap()["transcriptFile"],
        json!(expected)
    );
}

#[test]
fn a_parent_path_with_forward_slashes_is_joined_as_node_joins_it() {
    let f = Fixture::new();
    let dir = f.home.path().join("projects").join("y");
    let parent = dir
        .join(format!("{LEAD}.jsonl"))
        .to_string_lossy()
        .replace('\\', "/");
    start(&f, "slash-0001", Some(std::path::Path::new(&parent)));
    let expected = dir
        .join(LEAD)
        .join("subagents")
        .join("agent-slash-0001.jsonl");
    assert_eq!(
        f.by_hook("slash-0001").unwrap()["transcriptFile"],
        json!(expected)
    );
}

#[test]
fn without_a_transcript_path_the_record_has_none_and_the_hook_is_silent() {
    let f = Fixture::new();
    let ran = start(&f, "plain-0001", None);
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
    assert_eq!(
        f.by_hook("plain-0001").unwrap()["transcriptFile"],
        Value::Null
    );
}

#[test]
fn an_agent_id_with_path_characters_gets_no_transcript_path() {
    let f = Fixture::new();
    let parent = f.home.path().join("p").join(format!("{LEAD}.jsonl"));
    start(&f, "../../evil", Some(&parent));
    assert_eq!(
        f.by_hook("../../evil").unwrap()["transcriptFile"],
        Value::Null
    );
}

#[test]
fn a_record_of_the_lead_that_carries_the_exact_agent_id_is_bound_and_gets_the_path_too() {
    let f = Fixture::new();
    let id = f.manual(
        "A-77",
        "hand-made",
        "a hand made record",
        &json!({ "claudeId": "bind-0002" }),
    );
    let dir = f.home.path().join("projects").join("y");
    start(&f, "bind-0002", Some(&dir.join(format!("{LEAD}.jsonl"))));
    let a = &f.board()["agents"][&id];
    assert_eq!(a["claudeId"], "bind-0002");
    assert_eq!(a["rev"], 2, "stamped: it got its path");
    let expected = dir
        .join(LEAD)
        .join("subagents")
        .join("agent-bind-0002.jsonl");
    assert_eq!(a["transcriptFile"], json!(expected));
}
