use serde_json::json;

use super::*;
use crate::session::args::SessionArgs;

fn base() -> SessionArgs {
    SessionArgs {
        model: "sonnet".to_owned(),
        effort: None,
        resume: None,
        fork: false,
        mcp_config: "C:/tmp/mcp.json".to_owned(),
        budget_left_usd: 10.0,
    }
}

fn bash(command: &str) -> Verdict {
    verdict("Bash", &json!({ "command": command }))
}

// test-tag: critical
#[test]
fn the_shadow_cannot_run_a_write_verb() {
    for command in [
        "coord item status I-1 done",
        "coord inbox send coordinator hi",
        "coord entry add x",
        "coord item list; coord item status I-1 done",
        "coord item list && rm x",
        "coord item list > out.txt",
        "coord item show $(whoami)",
        "rm -rf x",
        "echo hi",
    ] {
        assert_eq!(bash(command), Verdict::Deny, "{command}");
    }
    for tool in [
        "Edit",
        "Write",
        "NotebookEdit",
        "mcp__coord-dashboard__post_entry",
    ] {
        assert_eq!(verdict(tool, &json!({})), Verdict::Deny, "{tool}");
    }
}

#[test]
fn the_shadow_may_read() {
    for command in [
        "coord item list",
        "coord thread show coordinator",
        "coord inbox",
        "node .claude/tools/coord/coord.mjs item show I-3",
    ] {
        assert_eq!(bash(command), Verdict::Allow, "{command}");
    }
    for tool in ["Read", "Grep", "Glob", "mcp__coord-dashboard__list_items"] {
        assert_eq!(verdict(tool, &json!({})), Verdict::Allow, "{tool}");
    }
}

// test-tag: critical
#[test]
fn the_shadow_registers_only_as_a_plain_thread() {
    assert_eq!(
        bash("coord thread register coordinator-hosted --role thread"),
        Verdict::Allow
    );
    assert_eq!(
        bash("coord thread register coordinator-hosted"),
        Verdict::Allow
    );
    for command in [
        "coord thread register coordinator",
        "coord thread register coordinator-hosted --role coordinator",
        "coord thread register other",
    ] {
        assert_eq!(bash(command), Verdict::Deny, "{command}");
    }
}

// test-tag: critical
#[test]
fn the_shadow_is_not_the_overlord_thread() {
    assert_ne!(THREAD, OVERLORD_THREAD);
    assert!(!mirrors(THREAD));
}

// test-tag: critical
#[test]
fn the_mirror_copies_every_overlord_wake_and_nothing_else() {
    assert!(mirrors("coordinator"));
    for other in [
        "coordinator-hosted",
        "argus",
        "brock",
        "*",
        "",
        "Coordinator",
    ] {
        assert!(!mirrors(other), "{other}");
    }
}

#[test]
fn the_first_start_forks_the_desktop_session() {
    let args = shadow_args(&base(), "desktop-1");
    assert_eq!(args.resume.as_deref(), Some("desktop-1"));
    assert!(args.fork);
    let line = shadow_argv(&base(), "desktop-1");
    assert!(line.windows(2).any(|w| w == ["--resume", "desktop-1"]));
    assert!(line.iter().any(|a| a == "--fork-session"));
    let at = line.iter().position(|a| a == "--disallowedTools").unwrap();
    assert!(line[at + 1].contains("Write"));
}

#[test]
fn a_restart_resumes_the_shadow_without_forking_again() {
    let mut again = base();
    again.resume = Some("shadow-9".to_owned());
    let args = shadow_args(&again, "desktop-1");
    assert_eq!(args.resume.as_deref(), Some("shadow-9"));
    assert!(!args.fork);
}

#[test]
fn a_turn_becomes_one_entry_and_an_empty_turn_none() {
    let mut turn = Turn::default();
    assert_eq!(turn.finish("t0"), None);
    turn.wake("W-1".to_owned());
    turn.said("I would reply".to_owned());
    let entry = turn.finish("t1").unwrap();
    assert_eq!(entry["wakes"], json!(["W-1"]));
    assert_eq!(entry["said"], "I would reply");
    assert_eq!(turn.finish("t2"), None);
}

#[test]
fn the_output_lines_are_told_apart() {
    let said = r#"{"type":"assistant","session_id":"s","message":{"content":[{"type":"text","text":"hi"}]}}"#;
    assert_eq!(seen(said), Seen::Said("hi".to_owned()));
    let result = r#"{"type":"result","subtype":"success","session_id":"s"}"#;
    assert_eq!(seen(result), Seen::TurnEnd);
    assert_eq!(seen("not json"), Seen::Nothing);
}

#[test]
fn a_denial_names_the_tool() {
    let entry = denial_entry("t", "Write", "a.txt");
    assert_eq!(entry["kind"], "denied");
    assert_eq!(entry["tool"], "Write");
}
