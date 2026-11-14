//! S2: `coord hook listen` against a stand-in wake stream on a loopback port: the wake line and
//! exit 2, the replaced listener, the lead listening to every thread, the stream joined again.

// test-tag: standard

use std::fs;
use std::sync::Arc;

use crate::hook_support::listen::{Answer, always, listen, serve, serve_one_wake, streams, wake};
use crate::hook_support::{LEAD, UI, fixture};
use serde_json::{Value, json};

#[test]
fn a_wake_for_the_thread_ends_the_listener_with_exit_2_and_the_line_on_stderr() {
    let server = serve_one_wake();
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(
        ran.err,
        "WAKE ui: lead: look at the build. Run: coord inbox\n"
    );
    assert_eq!(ran.out, "");
    assert_eq!(
        streams(&server),
        ["/api/p/alpha/wakes/stream?thread=ui&v=2&id=Alpha%20-%20UI"]
    );
}

#[test]
fn under_the_hosted_overlord_the_listener_ends_at_once_without_joining_the_stream() {
    let server = serve_one_wake();
    let f = fixture(&server);
    let ran = listen(&f, LEAD, "SessionStart", &[("ARGUS_HOSTED", "1")]);
    assert_eq!((ran.code, ran.out.as_str(), ran.err.as_str()), (0, "", ""));
    assert_eq!(streams(&server).len(), 0);
}

#[test]
fn several_wakes_are_counted_and_unreadable_data_still_wakes() {
    let data = json!({ "thread": "ui", "lines": ["a", "b"], "count": 3 });
    let server = serve(always(Some(wake(&data))));
    let f = fixture(&server);
    assert_eq!(
        listen(&f, UI, "Stop", &[]).err,
        "WAKE ui: a; b (3 wakes). Run: coord inbox\n"
    );
    let bad = serve(always(Some("event: wake\ndata: not json\n\n".to_owned())));
    let f = fixture(&bad);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!(
        (ran.code, ran.err.as_str()),
        (2, "WAKE ui: see coord inbox\n")
    );
}

#[test]
fn a_newer_listener_replaces_this_one_which_ends_silently_with_exit_0() {
    let server = serve(always(Some("event: replaced\ndata: {}\n\n".to_owned())));
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!((ran.code, ran.out.as_str(), ran.err.as_str()), (0, "", ""));
}

#[test]
fn a_lead_listens_for_every_thread_of_its_project() {
    let server = serve(always(Some(wake(
        &json!({ "thread": "ui", "lines": ["ui needs something"] }),
    ))));
    let f = fixture(&server);
    let ran = listen(&f, LEAD, "SessionStart", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(ran.err, "WAKE ui: ui needs something. Run: coord inbox\n");
    assert_eq!(
        streams(&server),
        ["/api/p/alpha/wakes/stream?thread=*&v=2&id=Alpha"]
    );
}

#[test]
fn a_lead_skips_the_wakes_it_sent_and_hears_the_next_one_from_another_sender() {
    let own = wake(&json!({ "thread": "ui", "lines": ["coordinator: do it"], "count": 1 }));
    let other = wake(&json!({ "thread": "ui", "lines": ["ui: done"], "count": 1 }));
    let server = serve(always(Some(format!("{own}{other}"))));
    let f = fixture(&server);
    let ran = listen(&f, LEAD, "SessionStart", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(
        ran.err,
        "WAKE ui: ui: done. Run: coord inbox
"
    );
    assert_eq!(streams(&server).len(), 1);
}

#[test]
fn a_thread_listener_still_wakes_for_a_line_that_starts_with_its_own_id() {
    let own = wake(&json!({ "thread": "ui", "lines": ["ui: from me"], "count": 1 }));
    let server = serve(always(Some(own)));
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[]);
    assert_eq!(ran.code, 2);
    assert_eq!(
        ran.err,
        "WAKE ui: ui: from me. Run: coord inbox
"
    );
}

#[test]
fn an_unregistered_session_exits_0_with_nothing_to_say_and_opens_no_stream() {
    let server = serve(always(None));
    let f = fixture(&server);
    let stranger = listen(&f, "sid-stranger", "Stop", &[]);
    assert_eq!((stranger.code, stranger.err.as_str()), (0, ""));
    assert!(server.paths().is_empty(), "no stream for a stranger");
}

#[test]
fn a_stream_that_ended_is_joined_again_and_the_wake_still_arrives() {
    let answer_with: Answer = Arc::new(|n| {
        Some(if n == 0 {
            String::new()
        } else {
            wake(&json!({ "thread": "ui", "lines": ["second try"] }))
        })
    });
    let server = serve(answer_with);
    let f = fixture(&server);
    let ran = listen(&f, UI, "Stop", &[("COORD_LISTEN_MS", "20000")]);
    assert_eq!(ran.code, 2);
    assert_eq!(ran.err, "WAKE ui: second try. Run: coord inbox\n");
    assert_eq!(streams(&server).len(), 2);
}

#[test]
fn session_start_says_wakes_are_automatic_only_where_the_settings_carry_the_listen_hook() {
    let server = serve(always(None));
    let f = fixture(&server);
    let start = || {
        let ran = f.hook(
            "session-start",
            &json!({ "session_id": UI, "cwd": f.cwd_text() }),
        );
        let out: Value = serde_json::from_str(&ran.out).unwrap();
        out["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let before = start();
    assert!(before.contains("start your wake-stream now with the Monitor tool"));
    assert!(!before.contains("Waking: automatic"));
    let settings = json!({ "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "node .claude/tools/coord/coord.mjs hook listen", "async": true, "asyncRewake": true }] }] } });
    fs::create_dir_all(f.cwd.join(".claude")).unwrap();
    fs::write(
        f.cwd.join(".claude").join("settings.json"),
        settings.to_string(),
    )
    .unwrap();
    let after = start();
    assert!(after.contains("Waking: automatic. A background hook is your wake listener"));
    assert!(after.contains("Do NOT start a Monitor wake-stream"));
    assert!(!after.contains("with the Monitor tool"));
}
