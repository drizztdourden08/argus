//! The hooks on both engines: every event with the stdin JSON Claude Code sends, run in order on
//! the Node CLI and on `argus coord` in twin homes, compared on the exit code, stdout, stderr and
//! every file of the homes (board bytes included) after each step. The registered listener is
//! left out: its stamp holds the pid of each engine and its wait has no end to compare; the
//! listener cases that end at once are in. Skips with a reason when node, `ai-config` or the
//! `argus` binary is absent.

mod support;

use serde_json::json;
use support::differ::{Differ, Engines, LEAD_SESSION, Step, WORKER_SESSION};

const NEW_SESSION: &str = "33333333-3333-4333-8333-333333333333";
const STRANGER: &str = "44444444-4444-4444-8444-444444444444";

fn hook(event: &str, input: &serde_json::Value) -> Step {
    Step::new(&["hook", event]).stdin(&input.to_string())
}

fn session(event: &str, sid: &str) -> Step {
    hook(
        event,
        &json!({ "session_id": sid, "hook_event_name": event }),
    )
}

fn run(test: &str, steps: &[Step]) {
    let Some(engines) = Engines::or_skip(test) else {
        return;
    };
    let differ = Differ::new(
        engines,
        &support::differ::Seed::Fixture(support::differ::fixture::standard_boards(
            LEAD_SESSION,
            WORKER_SESSION,
        )),
        &[LEAD_SESSION, WORKER_SESSION, NEW_SESSION, STRANGER],
    )
    .expect("twin homes");
    let report = differ.run(steps).expect("run the steps");
    println!("{report}");
    report.assert_no_mismatch();
    assert_eq!(report.not_ported(), 0, "{report}");
}

#[test]
fn the_session_hooks_and_the_guard_match_node() {
    run(
        "the_session_hooks_and_the_guard_match_node",
        &[
            session("session-start", WORKER_SESSION),
            session("prompt", WORKER_SESSION),
            session("tool", LEAD_SESSION),
            session("stop", WORKER_SESSION),
            session("guard", LEAD_SESSION),
            session("session-start", NEW_SESSION).env("COORD_PROJECT", "alpha"),
            session("prompt", NEW_SESSION).env("COORD_PROJECT", "alpha"),
            session("tool", NEW_SESSION).env("COORD_PROJECT", "alpha"),
            session("stop", NEW_SESSION).env("COORD_PROJECT", "alpha"),
            session("guard", NEW_SESSION).env("COORD_PROJECT", "alpha"),
            session("session-start", STRANGER),
            session("listen", STRANGER).env("COORD_PROJECT", "alpha"),
            session("listen", NEW_SESSION).env("COORD_PROJECT", "alpha"),
            Step::new(&["hook", "frobnicate"]).stdin("{}"),
            Step::new(&["hook", "tool"]).stdin("null"),
            Step::new(&["hook", "prompt"]).stdin("not json"),
        ],
    );
}

/// A hook of the lead's session with these fields.
fn lead(event: &str, name: &str, fields: &serde_json::Value) -> Step {
    let mut input = json!({ "session_id": LEAD_SESSION, "hook_event_name": name });
    for (k, v) in fields.as_object().into_iter().flatten() {
        input[k] = v.clone();
    }
    hook(event, &input)
}

/// The sub-agent rule of the tool hook on both engines: a call whose payload has a non-empty
/// `agent_id` marks no activity, so an idle lead stays idle; an empty `agent_id`, or a
/// transcript path alone, is the lead's own call. The board has no waiting message: Node
/// hands a sub-agent its lead's inbox, and the Argus hook (owner-approved, piece 2 of the
/// hook switch-over) does not, so a message here would differ by design. That half is tested
/// in `hook_session.rs`.
#[test]
fn a_sub_agents_tool_call_marks_no_activity_on_either_engine() {
    let call = |fields: serde_json::Value| lead("tool", "PostToolUse", &fields);
    let steps = [
        lead("prompt", "UserPromptSubmit", &json!({})),
        lead("stop", "Stop", &json!({})),
        call(json!({ "agent_id": "a1111111111111111" })),
        call(
            json!({ "agent_id": "a1111111111111111", "agent_transcript_path": "C:/t/agent-a9999999999999999.jsonl" }),
        ),
        call(json!({ "agent_id": "" })),
        lead("stop", "Stop", &json!({})),
        call(json!({ "agent_transcript_path": "C:/t/agent-a2222222222222222.jsonl" })),
        lead("stop", "Stop", &json!({})),
        call(json!({ "agent_id": "a1111111111111111" })),
    ];
    run(
        "a_sub_agents_tool_call_marks_no_activity_on_either_engine",
        &steps,
    );
}

fn dispatches() -> Vec<Step> {
    let tool_input = json!({
        "description": "Run the test suite", "prompt": "run cargo test and report",
        "subagent_type": "general-purpose", "model": "sonnet", "run_in_background": true,
    });
    let agent = json!({ "tool_name": "Agent", "tool_use_id": "toolu_1", "tool_input": tool_input });
    let bash = json!({ "tool_name": "Bash", "tool_use_id": "toolu_2", "tool_input": {} });
    let second = json!({ "tool_name": "Task", "tool_use_id": "toolu_3", "tool_input": { "description": "Explore the parser" } });
    vec![
        lead("agent-dispatch", "PreToolUse", &agent),
        lead("agent-dispatch", "PreToolUse", &agent),
        lead("agent-dispatch", "PreToolUse", &bash),
        lead("agent-dispatch", "PreToolUse", &second),
    ]
}

fn starts_and_results() -> Vec<Step> {
    let first = json!({ "agent_id": "a1111111111111111", "agent_type": "general-purpose", "transcript_path": "C:/t/parent.jsonl" });
    let other = json!({ "tool_name": "Agent", "tool_use_id": "toolu_1", "tool_response": { "agentId": "a2222222222222222", "description": "Run the test suite" } });
    let own = json!({ "tool_name": "Agent", "tool_use_id": "toolu_1", "tool_response": { "agentId": "a1111111111111111" } });
    let text = json!({ "tool_name": "Task", "tool_use_id": "toolu_9", "tool_response": { "content": [{ "type": "text", "text": "agentId: a3333333333333333 (internal ID)" }] } });
    vec![
        lead("subagent-start", "SubagentStart", &first),
        lead("agent-result", "PostToolUse", &other),
        lead("agent-result", "PostToolUse", &own),
        lead("agent-result", "PostToolUse", &text),
    ]
}

fn stops() -> Vec<Step> {
    let done =
        json!({ "agent_id": "a1111111111111111", "last_assistant_message": "  all green  " });
    let early = json!({ "agent_id": "a4444444444444444", "last_assistant_message": "early" });
    let late = json!({ "agent_id": "a4444444444444444", "agent_type": "Explore" });
    let confused = json!({ "agent_id": "a1111111111111111", "agent_transcript_path": "C:/t/agent-a9999999999999999.jsonl" });
    vec![
        lead("subagent-stop", "SubagentStop", &done),
        lead(
            "subagent-start",
            "SubagentStart",
            &json!({ "agent_id": "a1111111111111111" }),
        ),
        lead("subagent-stop", "SubagentStop", &early),
        lead("subagent-start", "SubagentStart", &late),
        lead("subagent-stop", "SubagentStop", &confused),
        hook(
            "subagent-start",
            &json!({ "session_id": STRANGER, "agent_id": "a5555555555555555" }),
        ),
        hook("subagent-stop", &json!({})),
    ]
}

#[test]
fn the_sub_agent_hooks_match_node() {
    let mut steps = dispatches();
    steps.extend(starts_and_results());
    steps.extend(stops());
    run("the_sub_agent_hooks_match_node", &steps);
}
