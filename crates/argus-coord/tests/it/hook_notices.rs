//! S2: the notice digest of the prompt hook against a stand-in dashboard.

// test-tag: standard

use crate::hook_support::{
    Fixture, LEAD, StandIn, UI, ago, digest_dashboard, fixture, prompt_after_message,
    session_input as input,
};
use serde_json::{Value, json};

const LINE: &str = "Board notices since your last turn: 3 (kanban-stale 2, worktree-review 1). Newest: \"I-4 sits in doing for 3 days\". Not wakes: the board header lists them.";

fn notice_paths(server: &StandIn) -> Vec<String> {
    server
        .paths()
        .into_iter()
        .filter(|p| p.contains("/wakes/notices"))
        .collect()
}

fn digest(line: Option<&str>) -> Value {
    json!({ "count": 3, "kinds": [], "newest": [], "line": line })
}

fn seen_at(f: &Fixture, id: &str, at: &str) {
    let mut board = f.board();
    board["threads"][id]["lastSeen"] = json!(at);
    f.save(&board);
}

#[test]
fn a_prompt_prints_the_digest_of_the_notices_since_the_thread_last_turn() {
    let server = digest_dashboard(&digest(Some(LINE)));
    let f = fixture(&server);
    let last_turn = ago(30);
    seen_at(&f, "ui", &last_turn);
    let ran = f.hook("prompt", &input(&f, UI));
    assert_eq!((ran.code, ran.out.as_str()), (0, LINE));
    let since = last_turn.replace(':', "%3A");
    assert_eq!(
        notice_paths(&server),
        [format!(
            "/api/p/alpha/wakes/notices?thread=ui&since={since}"
        )]
    );
}

#[test]
fn a_lead_asks_for_the_notices_of_every_thread() {
    let server = digest_dashboard(&digest(Some(LINE)));
    let f = fixture(&server);
    let ran = f.hook("prompt", &input(&f, LEAD));
    assert_eq!(ran.out, LINE);
    let asked = notice_paths(&server);
    assert_eq!(asked.len(), 1);
    assert!(
        asked[0].starts_with("/api/p/alpha/wakes/notices?thread=*&since="),
        "{}",
        asked[0]
    );
}

#[test]
fn no_notice_and_no_message_say_nothing() {
    let server = digest_dashboard(&digest(None));
    let f = fixture(&server);
    let ran = f.hook("prompt", &input(&f, UI));
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
}

#[test]
fn a_dashboard_that_is_down_adds_nothing() {
    let f = Fixture::new();
    let ran = f.hook("prompt", &input(&f, UI));
    assert_eq!((ran.code, ran.out.as_str()), (0, ""));
}

#[test]
fn new_messages_and_the_digest_come_in_one_text_the_digest_last() {
    let server = digest_dashboard(&digest(Some(LINE)));
    let f = fixture(&server);
    let ran = prompt_after_message(&f, "Start with the parser.");
    assert!(
        ran.out
            .contains("- [note] from coordinator: Start with the parser.")
    );
    assert!(ran.out.ends_with(&format!("\n{LINE}")), "{}", ran.out);
}
