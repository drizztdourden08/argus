//! KEEP: K1.
//! `wakes_for`: one test per row of the policy (what wakes whom, and what must not).

// test-tag: critical

use argus_wake::board::{BoardView, EntryView};
use argus_wake::policy::{ApplyCtx, RunView, failed_run_line, wake_for_run, wakes_for};
use argus_wake::reasons::{EXTERNAL_REASONS, IMMEDIATE_REASONS, WAKE_REASONS};
use argus_wake::testkit::{board, failed_run};
use argus_wake::wake::WakeInput;
use serde_json::{Value, json};

fn test_record(id: &str, thread: &str, build: Option<&str>, status: &str, mode: &str) -> Value {
    json!({"id": id, "thread": thread, "build": build, "status": status, "mode": mode, "item": "I-1"})
}

fn manual(id: &str, thread: &str, build: &str, status: &str) -> Value {
    test_record(id, thread, Some(build), status, "manual")
}

fn mk_board(tests: Vec<Value>) -> BoardView {
    let tests: serde_json::Map<String, Value> = tests
        .into_iter()
        .map(|t| (t["id"].as_str().unwrap_or_default().to_owned(), t))
        .collect();
    board(&json!({
        "threads": {
            "lead": {"id": "lead", "role": "lead", "status": "active"},
            "ui": {"id": "ui", "status": "active"},
            "api": {"id": "api", "status": "paused"},
        },
        "items": {
            "I-1": {"id": "I-1", "title": "Login screen", "thread": "ui"},
            "I-2": {"id": "I-2", "title": "Settings", "thread": "ui"},
        },
        "tests": tests,
        "entries": {
            "Q-1": {"id": "Q-1", "kind": "question", "thread": "ui", "author": "ui"},
            "D-2": {"id": "D-2", "kind": "decision", "thread": "api", "author": "api"},
        },
    }))
}

#[allow(clippy::unnecessary_wraps)]
fn lead(_: &str) -> Option<String> {
    Some("lead".to_owned())
}

fn quiet(wakes: &[WakeInput]) {
    assert_eq!(wakes.len(), 0, "{wakes:?}");
}

struct Post<'a> {
    scope: &'a str,
    status: Option<&'a str>,
    board: Option<&'a BoardView>,
    before: Option<&'a BoardView>,
}

impl Post<'_> {
    fn wakes(&self, entry: &Value) -> Vec<WakeInput> {
        self.wakes_with(entry, &lead)
    }

    fn wakes_with(
        &self,
        entry: &Value,
        lead_of: &dyn Fn(&str) -> Option<String>,
    ) -> Vec<WakeInput> {
        let entry: EntryView = serde_json::from_value(entry.clone()).expect("entry");
        let ctx = ApplyCtx {
            scope: self.scope,
            entry: Some(&entry),
            status: self.status,
            board: self.board,
            before: self.before,
            lead_of,
        };
        wakes_for(&ctx)
    }
}

fn alpha<'a>(board: &'a BoardView, before: Option<&'a BoardView>) -> Post<'a> {
    Post {
        scope: "alpha",
        status: Some("applied"),
        board: Some(board),
        before,
    }
}

fn result(id: &str, outcome: &str, text: &str, entry_id: &str) -> Value {
    json!({"id": entry_id, "kind": "result", "replyTo": id, "outcome": outcome, "text": text})
}

fn summary(wakes: &[WakeInput]) -> Vec<(String, String, Option<String>)> {
    wakes
        .iter()
        .map(|w| (w.thread.clone(), w.reason.clone(), w.subject.clone()))
        .collect()
}

fn pairs(wakes: &[WakeInput]) -> Vec<(String, String)> {
    wakes
        .iter()
        .map(|w| (w.reason.clone(), w.text.clone()))
        .collect()
}

fn texts(wakes: &[WakeInput]) -> Vec<String> {
    wakes.iter().map(|w| w.text.clone()).collect()
}

#[test]
fn a_failing_result_wakes_the_test_thread_with_the_first_100_characters_of_the_note() {
    let board = mk_board(vec![manual("T-5", "ui", "b4", "fail")]);
    let note = "The form does not close after saving and the spinner stays on forever. ".repeat(3);
    let wakes = alpha(&board, Some(&board)).wakes(&result("T-5", "fail", &note, "O-1"));
    assert_eq!(wakes.len(), 1);
    let w = &wakes[0];
    assert_eq!(
        (
            w.scope.as_str(),
            w.thread.as_str(),
            w.reason.as_str(),
            w.subject.as_deref()
        ),
        ("alpha", "ui", "test-fail", Some("T-5"))
    );
    assert!(w.text.starts_with("T-5 failed: The form does not close"));
    let tail = " [full text in O-1: coord inbox]";
    assert_eq!(
        w.text.chars().count(),
        "T-5 failed: ".len() + 100 + tail.len()
    );
    assert!(w.text.ends_with(&format!("…{tail}")));
}

#[test]
fn a_long_owner_text_is_clipped_in_the_wake_but_never_lost() {
    let board = mk_board(vec![manual("T-5", "ui", "b4", "fail")]);
    let post = alpha(&board, Some(&board));
    let long = "word ".repeat(4000);
    let wakes = post.wakes(&result("T-5", "fail", &long, "O-77"));
    assert!(
        wakes[0].text.chars().count() <= 300,
        "{}",
        wakes[0].text.len()
    );
    assert!(wakes[0].text.starts_with("T-5 failed: word word word"));
    assert!(wakes[0].text.ends_with("[full text in O-77: coord inbox]"));
    assert_eq!(
        texts(&post.wakes(&result("T-5", "fail", "one line only", "O-78"))),
        ["T-5 failed: one line only"]
    );
    let plain = mk_board(vec![]);
    let entry = json!({"id": "O-79", "kind": "followup", "replyTo": "Q-1", "thread": "ui", "text": "a ".repeat(500)});
    let answer = alpha(&plain, None).wakes(&entry);
    assert!(answer[0].text.starts_with("owner answered Q-1: a a a"));
    assert!(answer[0].text.ends_with("[full text in O-79: coord inbox]"));
}

#[test]
fn partial_and_blocked_wake_too_and_carry_the_owner_note() {
    let board = mk_board(vec![
        manual("T-5", "ui", "b4", "partial"),
        manual("T-6", "ui", "b4", "blocked"),
    ]);
    let post = alpha(&board, None);
    let partial = post.wakes(&result(
        "T-5",
        "partial",
        "the save works, the list does not refresh",
        "O-1",
    ));
    assert_eq!(
        pairs(&partial),
        [(
            "test-partial".to_owned(),
            "T-5 partial: the save works, the list does not refresh".to_owned()
        )]
    );
    let none = post.wakes(&result("T-5", "partial", "", "O-1"));
    assert_eq!(
        pairs(&none),
        [("test-partial".to_owned(), "T-5 partial: no note".to_owned())]
    );
    let blocked = post.wakes(&result("T-6", "blocked", "no device", "O-1"));
    assert_eq!(
        pairs(&blocked),
        [(
            "test-blocked".to_owned(),
            "T-6 blocked: no device".to_owned()
        )]
    );
}

fn with_run(mut raw: Value, stage: &str, thread: &str, results: &Value) -> BoardView {
    raw["items"]["I-9"] = json!({
        "id": "I-9", "type": "test-run", "thread": thread, "stage": stage, "n": 4,
        "fields": {"plan": "I-1", "results": results}
    });
    board(&raw)
}

fn results() -> Value {
    json!([
        {"test": "T-1", "status": "pass"},
        {"test": "T-2", "status": "fail"},
        {"test": "T-3", "status": "pass"},
    ])
}

fn base_json() -> Value {
    json!({
        "threads": {
            "lead": {"id": "lead", "role": "lead", "status": "active"},
            "ui": {"id": "ui", "status": "active"},
        },
        "items": {},
        "tests": {
            "T-3": {"id": "T-3", "thread": "ui", "status": "pass", "mode": "manual", "item": "I-1"},
        },
    })
}

#[test]
fn a_pass_wakes_nobody_even_the_one_that_ends_a_run() {
    let running = with_run(base_json(), "running", "ui", &results());
    quiet(&alpha(&running, Some(&running)).wakes(&result("T-3", "pass", "", "O-1")));
    let after = with_run(base_json(), "reported", "ui", &results());
    quiet(&alpha(&after, Some(&running)).wakes(&result("T-3", "pass", "", "O-1")));
}

#[test]
fn the_owner_answering_a_question_wakes_the_thread_that_asked_and_a_decision_answer_too() {
    let board = mk_board(vec![]);
    let post = alpha(&board, None);
    let answer = post.wakes(&json!({"id": "O-2", "kind": "followup", "replyTo": "Q-1", "thread": "ui", "text": "Yes, remember the user."}));
    assert_eq!(
        answer,
        [WakeInput::about(
            "alpha",
            "ui",
            "owner-answer",
            "Q-1",
            "owner answered Q-1: Yes, remember the user."
        )]
    );
    let pending = Post {
        status: Some("pending"),
        ..alpha(&board, None)
    };
    let decided = pending.wakes(&json!({"id": "O-3", "kind": "decision", "replyTo": "D-2", "thread": "api", "text": "JSON."}));
    assert_eq!(
        pairs(&decided),
        [(
            "owner-answer".to_owned(),
            "owner decided on D-2: JSON.".to_owned()
        )]
    );
    assert_eq!(decided[0].thread, "api");
}

#[test]
fn an_owner_note_or_decision_wakes_that_thread_and_an_unknown_thread_falls_back_to_the_lead() {
    let board = mk_board(vec![]);
    let post = alpha(&board, None);
    let note = post.wakes(&json!({"id": "O-4", "kind": "followup", "item": "I-1", "thread": "ui", "text": "Please also handle the empty state."}));
    assert_eq!(
        summary(&note),
        [(
            "ui".to_owned(),
            "owner-note".to_owned(),
            Some("I-1".to_owned())
        )]
    );
    assert!(note[0].text.starts_with("owner note on I-1: Please also"));
    let pending = Post {
        status: Some("pending"),
        ..alpha(&board, None)
    };
    let decision = pending
        .wakes(&json!({"id": "O-5", "kind": "decision", "thread": "ui", "text": "Ship it."}));
    assert_eq!(
        summary(&decision),
        [(
            "ui".to_owned(),
            "owner-decision".to_owned(),
            Some("ui".to_owned())
        )]
    );
    assert_eq!(decision[0].text, "owner decision: Ship it.");
    let lost =
        post.wakes(&json!({"id": "O-6", "kind": "followup", "thread": "ghost", "text": "hello"}));
    assert_eq!(lost[0].thread, "lead");
}

#[test]
fn a_worktree_request_from_the_page_is_an_owner_note_with_how_to_answer() {
    let board = mk_board(vec![]);
    let entry = json!({"id": "O-12", "kind": "followup", "thread": "ui", "text": "delete it", "worktree": {"id": "w1", "path": "X:/wt/one", "kind": "delete", "message": "stale"}});
    let wakes = alpha(&board, None).wakes(&entry);
    assert_eq!(
        wakes,
        [WakeInput::about(
            "alpha",
            "ui",
            "owner-note",
            "O-12",
            "owner asks to delete worktree X:/wt/one: stale. Answer: coord worktree done O-12 --note \"...\""
        )]
    );
}

#[test]
fn an_item_reassigned_to_another_thread_wakes_the_new_thread_also_inside_a_bulk() {
    let before = mk_board(vec![]);
    let mut moved = json!({"threads": {"ui": {"id": "ui"}, "api": {"id": "api"}}, "items": {"I-1": {"id": "I-1", "title": "Login screen", "thread": "api"}, "I-2": {"id": "I-2", "thread": "ui"}}});
    moved["entries"] = json!({});
    let board = board(&moved);
    let post = alpha(&board, Some(&before));
    let mv = json!({"id": "O-7", "kind": "move", "target": {"c": "items", "id": "I-1"}, "to": {"thread": "api"}});
    assert_eq!(
        summary(&post.wakes(&mv)),
        [(
            "api".to_owned(),
            "item-assigned".to_owned(),
            Some("I-1".to_owned())
        )]
    );
    let op = |id: &str, thread: &str| json!({"kind": "move", "target": {"c": "items", "id": id}, "to": {"thread": thread}});
    let edit =
        json!({"kind": "edit", "target": {"c": "items", "id": "I-2"}, "set": {"title": "x"}});
    let applied = json!([{"status": "applied"}, {"status": "applied"}, {"status": "applied"}]);
    let bulk = json!({"id": "O-8", "kind": "bulk", "ops": [op("I-1", "api"), op("I-2", "ui"), edit], "results": applied});
    assert_eq!(
        post.wakes(&bulk)
            .iter()
            .map(|w| w.thread.as_str())
            .collect::<Vec<_>>(),
        ["api"]
    );
    let refused = json!({"id": "O-8", "kind": "bulk", "ops": bulk["ops"], "results": [{"status": "refused"}, {"status": "applied"}, {"status": "applied"}]});
    quiet(&post.wakes(&refused));
}

#[test]
fn resuming_a_paused_thread_wakes_it_and_resuming_one_that_was_not_paused_does_not() {
    let before = mk_board(vec![]);
    let board = board(
        &json!({"threads": {"api": {"id": "api", "status": "active"}, "ui": {"id": "ui", "status": "active"}}}),
    );
    let post = alpha(&board, Some(&before));
    let resume = |thread: &str| json!({"id": "O-9", "kind": "resume", "replyTo": thread});
    assert_eq!(
        summary(&post.wakes(&resume("api"))),
        [(
            "api".to_owned(),
            "resume".to_owned(),
            Some("api".to_owned())
        )]
    );
    quiet(&post.wakes(&resume("ui")));
}

#[test]
fn a_portfolio_send_wakes_the_lead_of_every_project_it_was_delivered_to() {
    let entry = json!({"id": "O-10", "kind": "decision", "project": "all", "text": "Freeze the release.", "deliveredTo": ["alpha", "beta"]});
    let post = Post {
        scope: "_portfolio",
        status: Some("delivered"),
        board: None,
        before: None,
    };
    let lead_of = |key: &str| Some(if key == "alpha" { "lead" } else { "lead" }.to_owned());
    let wakes = post.wakes_with(&entry, &lead_of);
    let rows: Vec<(&str, &str, &str)> = wakes
        .iter()
        .map(|w| (w.scope.as_str(), w.thread.as_str(), w.reason.as_str()))
        .collect();
    assert_eq!(
        rows,
        [
            ("alpha", "lead", "portfolio-decision"),
            ("beta", "lead", "portfolio-decision")
        ]
    );
    assert!(
        wakes[0]
            .text
            .starts_with("owner decision from the Overlord: Freeze the release")
    );
    let note = json!({"id": "O-10", "kind": "followup", "text": "Freeze the release.", "deliveredTo": ["alpha"]});
    assert_eq!(wakes_reasons(&post.wakes(&note)), ["portfolio-note"]);
    let waiting = Post {
        status: Some("waiting for a board"),
        ..post
    };
    let nothing = json!({"id": "O-10", "kind": "decision", "text": "x", "deliveredTo": []});
    quiet(&waiting.wakes(&nothing));
}

fn wakes_reasons(wakes: &[WakeInput]) -> Vec<&str> {
    wakes.iter().map(|w| w.reason.as_str()).collect()
}

#[test]
fn these_never_wake_edits_status_moves_archive_delete_rerun_pause_hand_acks_proofs() {
    let board = mk_board(vec![manual("T-1", "ui", "b4", "pending")]);
    let before = mk_board(vec![manual("T-1", "ui", "b4", "pass")]);
    let post = alpha(&board, Some(&before));
    let target = |c: &str, id: &str| json!({"c": c, "id": id});
    let entries = [
        json!({"kind": "edit", "target": target("items", "I-1"), "set": {"title": "New"}}),
        json!({"kind": "edit", "target": target("items", "I-1"), "set": {"status": "done"}}),
        json!({"kind": "move", "target": target("items", "I-1"), "to": {"status": "doing"}}),
        json!({"kind": "move", "target": target("items", "I-1"), "to": {"before": "I-2"}}),
        json!({"kind": "archive", "target": target("items", "I-1")}),
        json!({"kind": "delete", "target": target("tests", "T-1")}),
        json!({"kind": "restore", "target": target("tests", "T-1")}),
        json!({"kind": "rerun", "replyTo": "T-1"}),
        json!({"kind": "pause", "replyTo": "ui"}),
        json!({"kind": "build", "handed": "b4"}),
        json!({"kind": "build", "handed": null}),
        json!({"kind": "ack", "replyTo": "N-1"}),
        json!({"kind": "done", "replyTo": "H-1"}),
        json!({"kind": "create", "target": {"c": "items"}, "set": {"title": "x"}}),
    ];
    for mut entry in entries {
        entry["id"] = json!("O-x");
        assert!(post.wakes(&entry).is_empty(), "{}", entry["kind"]);
    }
}

#[test]
fn a_post_that_was_not_applied_wakes_nobody() {
    let board = mk_board(vec![manual("T-5", "ui", "b4", "fail")]);
    for status in ["duplicate", "refused", "conflict", "cancelled"] {
        let post = Post {
            status: Some(status),
            ..alpha(&board, None)
        };
        quiet(&post.wakes(&result("T-5", "fail", "no", "O-1")));
    }
    let ctx = ApplyCtx {
        scope: "alpha",
        entry: None,
        status: Some("applied"),
        board: Some(&board),
        before: None,
        lead_of: &lead,
    };
    quiet(&wakes_for(&ctx));
}

#[test]
fn lead_message_lead_info_thread_message_and_wake_fault_come_only_from_outside_the_policy() {
    for reason in ["lead-message", "lead-info", "thread-message", "wake-fault"] {
        assert!(WAKE_REASONS.contains(&reason));
    }
    assert_eq!(
        EXTERNAL_REASONS,
        ["lead-message", "lead-info", "thread-message", "wake-fault"]
    );
    assert!(EXTERNAL_REASONS.iter().all(|r| WAKE_REASONS.contains(r)));
    let board = mk_board(vec![manual("T-5", "ui", "b4", "fail")]);
    let post = alpha(&board, None);
    let mut produced = post.wakes(&result("T-5", "fail", "no", "O-1"));
    produced.extend(
        post.wakes(&json!({"id": "O-3", "kind": "followup", "thread": "ui", "text": "hi"})),
    );
    let portfolio = Post {
        scope: "_portfolio",
        status: Some("delivered"),
        board: None,
        before: None,
    };
    produced.extend(
        portfolio.wakes(
            &json!({"id": "S-1", "kind": "decision", "text": "x", "deliveredTo": ["alpha"]}),
        ),
    );
    assert!(produced.len() >= 3);
    assert!(
        produced
            .iter()
            .all(|w| WAKE_REASONS.contains(&w.reason.as_str())
                && !EXTERNAL_REASONS.contains(&w.reason.as_str()))
    );
}

fn run() -> RunView {
    failed_run()
}

#[test]
fn a_failed_run_wakes_the_lead_with_how_it_failed_and_its_last_line() {
    let wake = wake_for_run(&run(), Some("lead")).expect("wake");
    let text = "R-6 launcher-master failed (exit 1): Node.js v24.8.0. Open its log, fix the cause and tell the owner.";
    assert_eq!(
        wake,
        WakeInput::about("alpha", "lead", "run-failed", "R-6", text)
    );
    let start = RunView {
        exit_code: None,
        error: Some("failed to start: working directory is not usable".to_owned()),
        ..run()
    };
    assert!(
        wake_for_run(&start, Some("lead"))
            .expect("wake")
            .text
            .contains("failed (failed to start: working directory")
    );
    let timeout = RunView {
        exit_code: None,
        error: Some("timeout".to_owned()),
        last_line: None,
        ..run()
    };
    assert!(
        wake_for_run(&timeout, Some("lead"))
            .expect("wake")
            .text
            .contains("failed (timeout).")
    );
    assert!(
        failed_run_line(&RunView {
            exit_code: None,
            error: None,
            last_line: None,
            ..run()
        })
        .expect("line")
        .ends_with("failed (failed)")
    );
}

#[test]
fn stopped_exited_and_orphaned_runs_and_a_missing_lead_wake_nobody() {
    for quiet in [
        RunView {
            status: "exited".to_owned(),
            ..run()
        },
        RunView {
            status: "stopped".to_owned(),
            ..run()
        },
        RunView {
            orphan: true,
            ..run()
        },
    ] {
        assert!(wake_for_run(&quiet, Some("lead")).is_none());
    }
    assert!(wake_for_run(&run(), None).is_none());
    assert!(WAKE_REASONS.contains(&"run-failed") && IMMEDIATE_REASONS.contains(&"run-failed"));
}
