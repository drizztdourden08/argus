//! S2: a hook that shows a thread the digest, its messages or a `WAKE` line stamps the read at the
//! dashboard (`POST /api/p/alpha/inbox/read`), as `coord inbox` does, against a stand-in dashboard.

// test-tag: standard

use crate::hook_support::listen::{READ_PATH, always, listen, serve, wake};
use crate::hook_support::{
    Fixture, LEAD, StandIn, UI, digest_dashboard, fixture, message, session_input as input,
};
use serde_json::json;

const LINE: &str = "Board notices since your last turn: 1 (kanban-stale 1). Not wakes: the board header lists them.";

fn dashboard(line: Option<&str>) -> StandIn {
    digest_dashboard(&json!({ "count": 1, "kinds": [], "newest": [], "line": line }))
}

fn stamps(server: &StandIn) -> usize {
    server.paths().iter().filter(|p| *p == READ_PATH).count()
}

#[test]
fn a_prompt_that_prints_the_digest_stamps_the_read() {
    let server = dashboard(Some(LINE));
    let f = fixture(&server);
    let ran = f.hook("prompt", &input(&f, LEAD));
    assert_eq!(ran.out, LINE);
    assert_eq!(stamps(&server), 1);
}

#[test]
fn a_prompt_that_prints_new_messages_stamps_the_read() {
    let server = dashboard(None);
    let f = fixture(&server);
    let mut board = f.board();
    board["inbox"]["ui"] = json!([message("Start with the parser.")]);
    f.save(&board);
    let ran = f.hook("prompt", &input(&f, UI));
    assert!(ran.out.contains("Start with the parser."), "{}", ran.out);
    assert_eq!(stamps(&server), 1);
}

#[test]
fn a_hook_that_prints_nothing_stamps_nothing() {
    let server = dashboard(None);
    let f = fixture(&server);
    let ran = f.hook("prompt", &input(&f, UI));
    assert_eq!(ran.out, "");
    assert_eq!(stamps(&server), 0);
}

#[test]
fn a_sub_agent_call_does_not_stamp_its_lead() {
    let server = dashboard(Some(LINE));
    let f = fixture(&server);
    let mut body = input(&f, LEAD);
    body["agent_id"] = json!("agent-1");
    f.hook("prompt", &body);
    assert_eq!(stamps(&server), 0);
}

#[test]
fn a_stamp_that_cannot_be_posted_never_breaks_the_hook() {
    let f = Fixture::new();
    let mut board = f.board();
    board["inbox"]["ui"] = json!([message("Start with the parser.")]);
    f.save(&board);
    let ran = f.hook("prompt", &input(&f, UI));
    assert_eq!(ran.code, 0);
    assert!(ran.out.contains("Start with the parser."), "{}", ran.out);
}

#[test]
fn a_heard_wake_stamps_the_read_of_the_thread_that_listens() {
    let server = serve(always(Some(wake(
        &json!({ "thread": "ui", "lines": ["lead: look at the build"], "count": 1 }),
    ))));
    let f = fixture(&server);
    let ran = listen(&f, LEAD, "Stop", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(stamps(&server), 1);
}
