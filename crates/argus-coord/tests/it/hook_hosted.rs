//! K2: a hosted session (`ARGUS_HOSTED=1`) registers as `host: argus`, is told it is woken by user
//! messages, and is left alone by the tab-closing guard.

// test-tag: critical

use crate::common::Ran;
use crate::hook_support::{Fixture, LEAD, UI, hook_with, session_input};
use serde_json::Value;

const HOSTED_LINE: &str = "Waking: hosted. Wakes come as user messages; no listener, no board tab.";

fn hosted(f: &Fixture, event: &str, sid: &str) -> Ran {
    let env = f.home.env_with("ARGUS_HOSTED", "1");
    hook_with(env, event, &session_input(f, sid).to_string(), &f.cwd)
}

fn context_of(ran: &Ran) -> String {
    let out: Value = serde_json::from_str(&ran.out).expect("json out");
    out["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("context")
        .to_owned()
}

#[test]
fn a_hosted_session_start_sets_host_argus_and_gets_the_hosted_wake_line_and_no_board_tab() {
    let f = Fixture::new();
    let ran = hosted(&f, "session-start", LEAD);
    assert_eq!(ran.code, 0, "{}", ran.err);
    let text = context_of(&ran);
    assert!(text.contains(HOSTED_LINE), "{text}");
    assert!(!text.contains("preview_start"), "{text}");
    assert!(!text.contains("wake-stream"), "{text}");
    assert_eq!(f.board()["threads"]["coordinator"]["host"], "argus");
}

#[test]
fn a_desktop_session_start_sets_no_host_and_keeps_the_listener_and_board_text() {
    let f = Fixture::new();
    let ran = f.hook("session-start", &session_input(&f, UI));
    let text = context_of(&ran);
    assert!(!text.contains(HOSTED_LINE), "{text}");
    assert!(text.contains("preview_start"), "{text}");
    assert_eq!(f.board()["threads"]["ui"].get("host"), None);
}

#[test]
fn a_desktop_session_on_a_thread_that_was_hosted_takes_the_host_off() {
    let f = Fixture::new();
    hosted(&f, "session-start", LEAD);
    f.hook("prompt", &session_input(&f, LEAD));
    assert_eq!(f.board()["threads"]["coordinator"].get("host"), None);
}

#[test]
fn the_tab_guard_skips_a_hosted_thread() {
    let f = Fixture::new();
    hosted(&f, "session-start", LEAD);
    let input = serde_json::json!({
        "session_id": LEAD,
        "cwd": f.cwd_text(),
        "tool_name": "mcp__Claude_Browser__tabs_close",
    });
    let ran = f.hook("guard", &input);
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
}
