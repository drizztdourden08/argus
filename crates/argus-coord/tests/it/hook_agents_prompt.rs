//! K1: a starting sub-agent takes the dispatch whose prompt it was given.

// test-tag: critical

use std::fs;

use crate::hook_support::{Fixture, LEAD};
use serde_json::{Value, json};

const KINDS: [(&str, &str); 3] = [
    ("Build the parser", "haiku"),
    ("Build the printer", "sonnet"),
    ("Build the linter", "opus"),
];

fn prompt_of(description: &str) -> String {
    format!("  {description}.\n\nWork in   the worktree and return a short report.")
}

fn dispatch_all(f: &Fixture) {
    for (i, (description, model)) in KINDS.iter().enumerate() {
        let input = json!({
            "description": description, "prompt": prompt_of(description),
            "subagent_type": "general-purpose", "model": model,
        });
        assert_eq!(f.dispatch(LEAD, input, &format!("toolu_{i}")).code, 0);
    }
}

fn start_with_prompt(f: &Fixture, agent_id: &str, prompt: &str) {
    let dir = f.home.path().join("session-files");
    let sub = dir.join(LEAD).join("subagents");
    fs::create_dir_all(&sub).unwrap();
    let line = json!({ "type": "user", "message": { "role": "user", "content": prompt } });
    fs::write(
        sub.join(format!("agent-{agent_id}.jsonl")),
        format!("{line}\n"),
    )
    .unwrap();
    let body = json!({
        "session_id": LEAD, "cwd": f.cwd_text(), "hook_event_name": "SubagentStart",
        "agent_id": agent_id, "agent_type": "general-purpose",
        "transcript_path": dir.join(format!("{LEAD}.jsonl")),
    });
    assert_eq!(f.hook("subagent-start", &body).code, 0);
}

fn record(f: &Fixture, agent_id: &str) -> Value {
    f.by_hook(agent_id).expect("the record")
}

#[test]
fn parallel_dispatches_of_one_type_bind_to_their_own_prompt_in_any_start_order() {
    let f = Fixture::new();
    dispatch_all(&f);
    for index in [2, 0, 1] {
        let (description, _) = KINDS[index];
        start_with_prompt(&f, &format!("agent-par-{index}"), &prompt_of(description));
    }
    for (index, (description, model)) in KINDS.iter().enumerate() {
        let a = record(&f, &format!("agent-par-{index}"));
        assert_eq!(a["task"], *description);
        assert_eq!(a["model"], *model);
        assert_eq!(a["toolUseId"], format!("toolu_{index}"));
    }
}

#[test]
fn a_start_whose_prompt_matches_nothing_falls_back_to_type_then_oldest() {
    let f = Fixture::new();
    dispatch_all(&f);
    start_with_prompt(&f, "agent-odd-1", "something nobody dispatched");
    assert_eq!(record(&f, "agent-odd-1")["task"], KINDS[0].0);
    f.started(LEAD, "agent-odd-2", "general-purpose");
    assert_eq!(record(&f, "agent-odd-2")["task"], KINDS[1].0);
}
