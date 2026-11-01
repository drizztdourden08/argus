//! The notice digest of the prompt hook: a registered thread's prompt asks the dashboard for the
//! passive notices since its last turn (its `lastSeen` before the visit) and prints the digest
//! line it gets; a lead asks for every thread (`*`); no line, or a dashboard that is down, adds
//! nothing; new messages and the digest come together. A small stand-in serves the routes.

use std::io::Write;
use std::net::TcpStream;

use crate::hook_support::{Fixture, LEAD, StandIn, UI, ago, fixture};
use serde_json::{Value, json};

const LINE: &str = "Board notices since your last turn: 3 (kanban-stale 2, worktree-review 1). Newest: \"I-4 sits in doing for 3 days\". Not wakes: the board header lists them.";

fn reply(mut stream: TcpStream, body: &str) {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).ok();
    stream.write_all(body.as_bytes()).ok();
    stream.flush().ok();
}

/// Answers the health check, and the notices route with `digest`.
fn serve(digest: &Value) -> StandIn {
    let body = digest.to_string();
    StandIn::start(move |_, stream, path| {
        let answer = if path.starts_with("/api/health") {
            r#"{"ok":true}"#
        } else {
            body.as_str()
        };
        reply(stream, answer);
    })
}

fn notice_paths(server: &StandIn) -> Vec<String> {
    server
        .paths()
        .into_iter()
        .filter(|p| p.contains("/wakes/notices"))
        .collect()
}

fn input(f: &Fixture, sid: &str) -> Value {
    json!({ "session_id": sid, "cwd": f.cwd_text() })
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
    let server = serve(&digest(Some(LINE)));
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
    let server = serve(&digest(Some(LINE)));
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
    let server = serve(&digest(None));
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
    let server = serve(&digest(Some(LINE)));
    let f = fixture(&server);
    let mut board = f.board();
    let message = json!({ "id": "m-1790000000000-ab12", "ts": ago(1), "from": "coordinator", "kind": "note", "text": "Start with the parser.", "ref": null, "deliveredAt": null });
    board["inbox"]["ui"] = json!([message]);
    f.save(&board);
    let ran = f.hook("prompt", &input(&f, UI));
    assert!(
        ran.out
            .contains("- [note] from coordinator: Start with the parser.")
    );
    assert!(ran.out.ends_with(&format!("\n{LINE}")), "{}", ran.out);
}
