//! The checks behind the wake routes: a wake from outside the policy (`coord lead`, `coord inbox send`) goes
//! through the engine like any other, only the Overlord clears a fault, and the stream and presence
//! parameters are cleaned.

use argus_wake::board::BoardView;
use argus_wake::engine::AttachOptions;
use argus_wake::external::{
    ClearRequest, RequestError, clean_label, clean_tab, clear_request, external_wake,
    listener_cap_message, no_open_fault, presence_thread, scope_of, stream_thread, valid_key,
    valid_thread_id,
};
use argus_wake::testkit::{Rig, Tap, board, thread};
use argus_wake::wake::WakeStatus;
use serde_json::{Value, json};

fn rig() -> Rig {
    let rig = Rig::new(
        &argus_wake::config::WakeConfig {
            window_ms: 150,
            ..Default::default()
        },
        false,
    );
    rig.boards.set(
        "beta",
        board(
            &json!({"threads": {"coordinator": thread("coordinator", "coordinator", &json!({}))}}),
        ),
    );
    rig
}

fn known(scope: &str, thread: &str) -> bool {
    scope == "beta" && thread == "coordinator"
}

fn post(rig: &mut Rig, body: &Value) -> Result<(), RequestError> {
    let wake = external_wake("beta", body, &known)?;
    rig.service.submit(&[wake]).expect("submit");
    Ok(())
}

fn text_post(text: &str, reason: &str) -> Value {
    json!({"thread": "coordinator", "reason": reason, "text": text})
}

fn listen(rig: &mut Rig, thread: &str) -> Tap {
    rig.listen("beta", thread, AttachOptions::default())
}

fn error(body: &Value) -> (u16, String) {
    let err = external_wake("beta", body, &known).expect_err("refused");
    (err.status(), err.message().to_owned())
}

#[test]
fn a_lead_message_wake_goes_through_the_engine_with_the_window_a_star_listener_and_a_queued_replay()
{
    let mut rig = rig();
    let mut star = listen(&mut rig, "*");
    let text = format!("beta lead: {}", "a very long message ".repeat(4));
    post(&mut rig, &text_post(&text, "lead-message")).expect("accepted");
    post(
        &mut rig,
        &text_post("pixelarttool lead: second", "lead-message"),
    )
    .expect("accepted");
    rig.advance(150);
    let event = star.events()[0].clone();
    assert_eq!(
        (event.thread.as_str(), event.count),
        ("coordinator", 2),
        "coalesced in the same window as any other wake"
    );
    assert_eq!(event.lines, [text.trim(), "pixelarttool lead: second"]);
    let delivered = rig
        .store
        .all()
        .into_iter()
        .find(|r| r.reason == "lead-message" && r.status == WakeStatus::Delivered)
        .expect("row");
    assert_eq!(
        (
            delivered.thread.as_str(),
            delivered.listeners,
            delivered.count
        ),
        ("coordinator", 1, 2)
    );

    rig.service.detach(&star.attached.id);
    post(&mut rig, &text_post("tessera lead: ping", "lead-message")).expect("accepted");
    rig.advance(450);
    let queued = rig
        .store
        .all()
        .into_iter()
        .find(|r| r.text == "tessera lead: ping")
        .expect("row");
    assert_eq!((queued.status, queued.listeners), (WakeStatus::Queued, 0));
    let mut late = listen(&mut rig, "coordinator");
    assert_eq!(late.lines(), [["tessera lead: ping"]]);
}

#[test]
fn a_thread_message_wake_goes_through_the_engine_like_a_lead_message() {
    let mut rig = rig();
    post(
        &mut rig,
        &text_post("ui: please look at T-9", "thread-message"),
    )
    .expect("accepted");
    rig.advance(450);
    let row = rig
        .store
        .all()
        .into_iter()
        .find(|r| r.reason == "thread-message")
        .expect("row");
    assert_eq!(
        (row.status, row.listeners, row.text.as_str()),
        (WakeStatus::Queued, 0, "ui: please look at T-9")
    );
    let mut late = listen(&mut rig, "coordinator");
    assert_eq!(late.lines(), [["ui: please look at T-9"]]);
    let mut second = listen(&mut rig, "coordinator");
    assert_eq!(second.count(), 0, "replayed once");
}

#[test]
fn only_the_outside_reasons_an_existing_thread_and_a_text_are_accepted_and_nothing_is_recorded_otherwise()
 {
    let mut rig = rig();
    let refused = [
        (
            json!({"thread": "coordinator", "reason": "test-fail", "text": "x"}),
            400,
            "reason must be one of lead-message, thread-message, wake-fault",
        ),
        (
            json!({"thread": "coordinator", "text": "x"}),
            400,
            "reason must be one of",
        ),
        (
            json!({"thread": "*", "reason": "lead-message", "text": "x"}),
            400,
            "thread must be a thread id",
        ),
        (
            json!({"reason": "lead-message", "text": "x"}),
            400,
            "thread must be a thread id",
        ),
        (
            json!({"thread": "coordinator", "reason": "lead-message", "text": "  "}),
            400,
            "text is required",
        ),
        (
            json!({"thread": "nobody", "reason": "lead-message", "text": "x"}),
            404,
            "no thread \"nobody\" on beta",
        ),
    ];
    for (body, status, message) in refused {
        let (got, text) = error(&body);
        assert_eq!(got, status, "{body}");
        assert!(text.starts_with(message), "{text}");
    }
    assert_eq!(
        external_wake("nowhere", &text_post("x", "lead-message"), &known)
            .expect_err("unknown board")
            .status(),
        404
    );
    rig.advance(450);
    assert_eq!(rig.store.all().len(), 0, "nothing was recorded");
    assert!(
        post(&mut rig, &text_post("x", "wake-fault")).is_ok(),
        "wake-fault is an outside reason too"
    );
}

#[test]
fn the_text_of_an_outside_wake_is_one_line_of_at_most_300_characters() {
    let wake = external_wake(
        "beta",
        &text_post(&format!("a\n\n  b {}", "y".repeat(400)), "thread-message"),
        &known,
    )
    .expect("wake");
    assert_eq!(wake.text.chars().count(), 300);
    assert!(wake.text.starts_with("a b yyy"));
    assert_eq!(
        (
            wake.scope.as_str(),
            wake.thread.as_str(),
            wake.subject.as_deref()
        ),
        ("beta", "coordinator", Some("coordinator"))
    );
}

fn overlord() -> BoardView {
    board(&json!({"threads": {
        "coordinator": thread("coordinator", "coordinator", &json!({})),
        "helper": thread("helper", "thread", &json!({})),
        "retired": thread("retired", "coordinator", &json!({"status": "archived"})),
    }}))
}

fn clear(body: &Value) -> Result<ClearRequest, RequestError> {
    clear_request(body, Some(&overlord()))
}

const NOTE: &str = "the ui session had crashed, restarted its wake-stream";

#[test]
fn clearing_a_fault_needs_a_project_a_thread_a_note_of_10_to_300_characters_and_the_overlord() {
    let ok = clear(&json!({"project": "alpha", "thread": "ui", "by": "coordinator", "note": NOTE}))
        .expect("accepted");
    assert_eq!(
        (
            ok.project.as_str(),
            ok.thread.as_str(),
            ok.by.as_str(),
            ok.note.as_str()
        ),
        ("alpha", "ui", "coordinator", NOTE)
    );
    let status = |body: Value| clear(&body).expect_err("refused").status();
    assert_eq!(
        status(json!({"project": "alpha", "thread": "ui", "by": "coordinator"})),
        400
    );
    assert_eq!(
        status(
            json!({"project": "alpha", "thread": "ui", "by": "coordinator", "note": "too short"})
        ),
        400
    );
    assert_eq!(
        status(
            json!({"project": "alpha", "thread": "ui", "by": "coordinator", "note": "x".repeat(301)})
        ),
        400
    );
    assert_eq!(
        status(json!({"project": "alpha", "thread": "*", "by": "coordinator", "note": NOTE})),
        400
    );
    assert_eq!(
        status(json!({"project": "Bad Key", "thread": "ui", "by": "coordinator", "note": NOTE})),
        400
    );
    assert_eq!(clear(&json!({"project": "alpha", "thread": "ui", "by": "coordinator", "note": "x".repeat(300)})).map(|r| r.note.len()), Ok(300));
}

#[test]
fn only_a_live_coordinator_of_the_portfolio_board_may_clear_a_fault() {
    for by in ["helper", "lead", "nobody", "", "retired"] {
        let err = clear(&json!({"project": "alpha", "thread": "ui", "by": by, "note": NOTE}))
            .expect_err("refused");
        assert_eq!(err.status(), 403, "by \"{by}\"");
        assert!(err.message().starts_with("only the Overlord"));
    }
    let no_board = clear_request(
        &json!({"project": "alpha", "thread": "ui", "by": "coordinator", "note": NOTE}),
        None,
    );
    assert_eq!(no_board.expect_err("no board").status(), 403);
    assert_eq!(
        no_open_fault("alpha", "ui"),
        RequestError::NotFound("no open wake fault for alpha/ui".to_owned())
    );
}

#[test]
fn keys_threads_labels_and_tabs_are_cleaned() {
    assert!(
        valid_key("pixelarttool")
            && valid_key("a1-b")
            && !valid_key("Bad_Key")
            && !valid_key("-a")
            && !valid_key("")
    );
    assert!(
        valid_thread_id("ui")
            && valid_thread_id("A.b_c~d:e@f+g-h")
            && !valid_thread_id("a b")
            && !valid_thread_id("*")
    );
    assert!(!valid_thread_id(&"x".repeat(61)) && valid_thread_id(&"x".repeat(60)));
    assert_eq!(stream_thread(None), Ok("*".to_owned()));
    assert_eq!(stream_thread(Some("")), Ok("*".to_owned()));
    assert_eq!(stream_thread(Some("ui")), Ok("ui".to_owned()));
    assert_eq!(stream_thread(Some("a b")).expect_err("bad").status(), 400);
    assert_eq!(presence_thread(Some("ui")), Ok("ui".to_owned()));
    assert_eq!(presence_thread(Some("*")).expect_err("star").status(), 400);
    assert_eq!(scope_of("alpha"), Ok("alpha"));
    assert_eq!(
        scope_of("Bad_Key").expect_err("bad").message(),
        "invalid project key \"Bad_Key\""
    );
    assert_eq!(clean_label("  ui\nsession ").as_deref(), Some("ui session"));
    assert_eq!(clean_label(" \t ").as_deref(), None);
    assert_eq!(clean_label(&"y".repeat(100)).map(|l| l.len()), Some(80));
    assert_eq!(clean_tab("ta b\n1"), "tab1");
    assert_eq!(clean_tab(" \n"), "tab");
    assert_eq!(clean_tab(&"z".repeat(50)).len(), 40);
    assert_eq!(
        listener_cap_message("alpha"),
        "too many wake listeners on alpha (max 64)"
    );
}
