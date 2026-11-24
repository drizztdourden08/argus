//! K1: the board side of the session hooks.

// test-tag: critical

use crate::hook_support::{Fixture, LEAD, UI, ago, hook_with};
use argus_coord::Env;
use serde_json::{Value, json};

fn input(f: &Fixture, sid: &str) -> Value {
    json!({ "session_id": sid, "cwd": f.cwd_text() })
}

fn with_message(f: &Fixture, to: &str, text: &str) {
    let mut board = f.board();
    let message = json!({ "id": "m-1790000000000-ab12", "ts": ago(1), "from": "lead", "kind": "note", "text": text, "ref": null, "deliveredAt": null });
    board["inbox"][to] = json!([message]);
    f.save(&board);
}

const MESSAGES: &str = "Coordination board: 1 new message(s) for thread \"ui\".\n- [note] from lead: Start with the parser.\nAct on them, then run `coord inbox --ack`.";

#[test]
fn a_prompt_hands_the_new_messages_over_once_and_marks_them_delivered() {
    let f = Fixture::new();
    with_message(&f, "ui", "Start with the parser.");
    let ran = f.hook("prompt", &input(&f, UI));
    assert_eq!(ran.code, 0);
    assert!(ran.out.ends_with(MESSAGES), "{}", ran.out);
    assert!(f.board()["inbox"]["ui"][0]["deliveredAt"].is_string());
    let again = f.hook("prompt", &input(&f, UI));
    assert_eq!((again.code, again.out.as_str()), (0, ""));
}

#[test]
fn a_tool_hook_says_nothing_without_messages_and_hands_them_over_as_json() {
    let f = Fixture::new();
    assert_eq!(f.hook("tool", &input(&f, UI)).out, "");
    with_message(&f, "ui", "Start with the parser.");
    let ran = f.hook("tool", &input(&f, UI));
    let expected = json!({ "hookSpecificOutput": { "hookEventName": "PostToolUse", "additionalContext": MESSAGES } });
    assert_eq!(ran.out, expected.to_string());
}

#[test]
fn stop_marks_the_thread_idle_and_blocks_with_waiting_messages() {
    let f = Fixture::new();
    let quiet = f.hook("stop", &input(&f, UI));
    assert_eq!((quiet.code, quiet.out.as_str()), (0, ""));
    assert_eq!(f.board()["threads"]["ui"]["activity"], "idle");
    with_message(&f, "ui", "Start with the parser.");
    let ran = f.hook("stop", &input(&f, UI));
    assert_eq!(
        ran.out,
        json!({ "decision": "block", "reason": MESSAGES }).to_string()
    );
    assert_eq!(f.hook("prompt", &input(&f, UI)).out, "");
    assert_eq!(f.board()["threads"]["ui"]["activity"], "working");
}

#[test]
fn a_fresh_heartbeat_with_the_same_activity_leaves_last_seen_alone() {
    let f = Fixture::new();
    let mut board = f.board();
    let seen = ago(0);
    board["threads"]["ui"]["lastSeen"] = json!(seen);
    f.save(&board);
    f.hook("tool", &input(&f, UI));
    let after = f.board();
    assert_eq!(after["threads"]["ui"]["lastSeen"], json!(seen));
    assert!(after["dirty"].get("threads/ui").is_none());
    assert_ne!(
        after["updatedAt"], board["updatedAt"],
        "the board is saved on every visit"
    );
}

#[test]
fn an_unknown_session_joins_as_a_placeholder_and_stop_never_adopts() {
    let f = Fixture::new();
    let before = f.raw();
    assert_eq!(f.hook("stop", &input(&f, "sid-new-1")).out, "");
    assert_eq!(f.raw(), before, "stop adopts nothing and saves nothing");
    let ran = f.hook("tool", &input(&f, "Sid_New-ABCDEF-9"));
    let out: Value = serde_json::from_str(&ran.out).unwrap();
    let text = out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(text.contains("placeholder thread \"s-sidnew\""), "{text}");
    let t = f.board()["threads"]["s-sidnew"].clone();
    assert_eq!(t["placeholder"], true);
    assert_eq!(t["sessionId"], "Sid_New-ABCDEF-9");
    assert_eq!(t["note"], "new session, not named yet");
    assert_eq!(t["createdBy"], "s-sidnew");
    assert_eq!(t["rank"], 3072);
    assert_eq!(t["link"], Value::Null);
    let keys: Vec<&String> = t.as_object().unwrap().keys().collect();
    let order = [
        "id",
        "createdAt",
        "createdBy",
        "updatedAt",
        "updatedBy",
        "rank",
        "name",
        "role",
        "placeholder",
        "status",
        "activity",
        "registeredAt",
        "lastSeen",
        "sessionId",
        "link",
        "root",
        "session",
        "subject",
        "note",
        "flags",
        "rev",
    ];
    assert_eq!(keys, order);
    let prompt = f.hook("prompt", &input(&f, "Sid_New-ABCDEF-9"));
    assert!(prompt.out.starts_with(
        "Coordination board: this session is still the unnamed placeholder \"s-sidnew\""
    ));
}

#[test]
fn the_host_session_gives_a_placeholder_its_link() {
    let f = Fixture::new();
    let env = f.home.env_with("CLAUDE_CODE_HOST_SESSION_ID", "host-123");
    hook_with(env, "prompt", &input(&f, "sid-linked").to_string(), &f.cwd);
    let link = &f.board()["threads"]["s-sidlin"]["link"];
    assert_eq!(link, "claude://claude.ai/epitaxy/host-123");
}

#[test]
fn a_session_whose_threads_here_all_ended_is_left_alone() {
    let f = Fixture::new();
    let mut board = f.board();
    board["threads"]["ui"]["status"] = json!("done");
    f.save(&board);
    let before = f.raw();
    for event in ["session-start", "prompt", "tool", "stop", "guard"] {
        let ran = f.hook(event, &input(&f, UI));
        assert_eq!((ran.code, ran.out.as_str()), (0, ""), "{event}");
    }
    assert_eq!(f.raw(), before);
}

#[test]
fn a_hook_with_no_folder_a_null_input_or_an_unknown_event_does_nothing() {
    let f = Fixture::new();
    let before = f.raw();
    let gone = json!({ "session_id": LEAD, "cwd": f.home.path().join("gone") });
    assert_eq!(f.hook("prompt", &gone).out, "");
    let env: Env = f.home.env();
    for (event, stdin) in [
        ("prompt", "null"),
        ("frobnicate", "{}"),
        ("tool", "not json"),
    ] {
        let ran = hook_with(env.clone(), event, stdin, f.home.path());
        assert_eq!(
            (ran.code, ran.out.as_str(), ran.err.as_str()),
            (0, "", ""),
            "{event}"
        );
    }
    assert_eq!(f.raw(), before);
}

#[test]
fn a_session_found_by_its_id_wins_over_the_folder() {
    let f = Fixture::new();
    let elsewhere = f.home.folder("elsewhere");
    let body = json!({ "session_id": LEAD, "cwd": elsewhere });
    let ran = crate::hook_support::hook_in(&f.home, "session-start", &body, &elsewhere);
    let out: Value = serde_json::from_str(&ran.out).unwrap();
    let text = out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(text.starts_with("You are thread \"lead\""), "{text}");
}

#[test]
fn a_sub_agents_tool_call_never_puts_an_idle_lead_back_to_work() {
    let f = Fixture::new();
    f.hook("prompt", &input(&f, UI));
    f.hook("stop", &input(&f, UI));
    let mut sub = input(&f, UI);
    sub["agent_id"] = json!("a1b2c3d4e5f6a7b8c");
    f.hook("tool", &sub);
    assert_eq!(f.board()["threads"]["ui"]["activity"], "idle");
    f.hook("tool", &input(&f, UI));
    assert_eq!(f.board()["threads"]["ui"]["activity"], "working");
    f.hook("tool", &sub);
    assert_eq!(f.board()["threads"]["ui"]["activity"], "working");
}

fn is_sub_agent(f: &Fixture, sid: &str) -> Value {
    let mut sub = input(f, sid);
    sub["agent_id"] = json!("a1b2c3d4e5f6a7b8c");
    sub
}

fn waiting_for_the_lead(f: &Fixture) {
    with_message(f, "ui", "Start with the parser.");
    assert!(f.board()["inbox"]["ui"][0]["deliveredAt"].is_null());
}

#[test]
fn a_sub_agents_prompt_tool_and_stop_leave_the_inbox_of_its_lead_alone() {
    for event in ["prompt", "tool", "stop"] {
        let f = Fixture::new();
        waiting_for_the_lead(&f);
        let ran = f.hook(event, &is_sub_agent(&f, UI));
        assert_eq!((ran.code, ran.out.as_str()), (0, ""), "{event}");
        assert!(
            f.board()["inbox"]["ui"][0]["deliveredAt"].is_null(),
            "{event}"
        );
        let lead = f.hook(event, &input(&f, UI));
        assert!(lead.out.contains("Start with the parser."), "{event}");
        assert!(f.board()["inbox"]["ui"][0]["deliveredAt"].is_string());
    }
}

#[test]
fn a_sub_agents_session_start_names_no_messages_and_marks_none_delivered() {
    let f = Fixture::new();
    waiting_for_the_lead(&f);
    let ran = f.hook("session-start", &is_sub_agent(&f, UI));
    assert!(ran.out.contains("You are thread \\\"ui\\\""), "{}", ran.out);
    assert!(!ran.out.contains("new message(s)"), "{}", ran.out);
    assert!(f.board()["inbox"]["ui"][0]["deliveredAt"].is_null());
    let lead = f.hook("session-start", &input(&f, UI));
    assert!(lead.out.contains("1 new message(s)"), "{}", lead.out);
}

#[test]
fn only_a_non_empty_agent_id_makes_a_call_a_sub_agents() {
    let f = Fixture::new();
    waiting_for_the_lead(&f);
    let mut blank = input(&f, UI);
    blank["agent_id"] = json!("");
    blank["agent_transcript_path"] = json!("C:/t/agent-a2222222222222222.jsonl");
    let ran = f.hook("tool", &blank);
    assert!(ran.out.contains("Start with the parser."), "{}", ran.out);
    assert!(f.board()["inbox"]["ui"][0]["deliveredAt"].is_string());
}
