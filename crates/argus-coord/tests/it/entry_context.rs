//! Ports `entry.context.test.mjs` (the CLI cases): every question or decision to the owner
//! carries a context, and a decision carries 2 to 5 options. `coord ask|decide|notify|message|
//! entry set` against a temp `COORD_HOME`: the refusals (no context, too short, too long, one
//! option, six options), the accepted shapes, the context stored on the entry, clipped on
//! information, edited with `entry set --context`. The case "the context reaches the page docs,
//! and survives importing the owner answer" is the export and import of `argus-board`, ported in
//! `crates/argus-board/tests/export_import.rs`.

use crate::work_board::Work;
use serde_json::json;

const CTX: &str = "The login form can remember the email. It changes what is stored on the machine, so it is your call. I will store nothing if unanswered.";

fn entry_count(work: &Work) -> usize {
    work.board()["entries"]
        .as_object()
        .map_or(0, serde_json::Map::len)
}

fn refused(work: &Work, words: &[&str], needle: &str) -> String {
    let ran = work.worker(words);
    assert_eq!(ran.code, 1, "{words:?}\n{}\n{}", ran.out, ran.err);
    assert!(ran.err.contains(needle), "{needle:?} not in {:?}", ran.err);
    ran.err
}

fn ok(work: &Work, words: &[&str]) -> String {
    let ran = work.worker(words);
    assert_eq!(ran.code, 0, "{words:?}\n{}\n{}", ran.out, ran.err);
    ran.out
}

#[test]
fn ask_without_context_is_refused_with_what_to_write_and_nothing_is_stored() {
    let work = Work::new();
    let err = refused(
        &work,
        &[
            "ask",
            "I-1",
            "Remember the email?",
            "--option",
            "Yes",
            "--option",
            "No",
        ],
        "--context is required",
    );
    for part in [
        "someone who has not followed the work",
        "why it needs the owner",
        "what you will assume",
        "Example: --context",
    ] {
        assert!(err.contains(part), "{part}");
    }
    assert!(
        !err.contains('\u{2013}') && !err.contains('\u{2014}'),
        "plain dashes only"
    );
    refused(
        &work,
        &["ask", "I-1", "Which name?"],
        "--context is required",
    );
    refused(
        &work,
        &["ask", "I-1", "Which name?", "--context"],
        "--context is required",
    );
    assert_eq!(entry_count(&work), 0);
}

#[test]
fn a_context_under_20_or_over_600_characters_is_refused() {
    let work = Work::new();
    refused(
        &work,
        &["ask", "I-1", "Which name?", "--context", "too short"],
        "--context is 9 chars, the minimum is 20",
    );
    refused(
        &work,
        &["ask", "I-1", "Which name?", "--context", &"x".repeat(601)],
        "context is 601 chars, the limit is 600",
    );
    assert_eq!(entry_count(&work), 0);
}

#[test]
fn a_decision_needs_2_to_5_options_none_with_decide_one_and_six_are_refused() {
    let work = Work::new();
    let text = "a decision needs 2 to 5 options, recommended first";
    refused(
        &work,
        &["ask", "I-1", "Remember?", "--decide", "--context", CTX],
        text,
    );
    refused(
        &work,
        &[
            "ask",
            "I-1",
            "Remember?",
            "--option",
            "Yes",
            "--context",
            CTX,
        ],
        text,
    );
    refused(
        &work,
        &[
            "ask",
            "I-1",
            "Remember?",
            "--decide",
            "--option",
            "Yes",
            "--context",
            CTX,
        ],
        text,
    );
    let mut six = vec!["ask", "I-1", "Remember?"];
    for option in ["a", "b", "c", "d", "e", "f"] {
        six.extend(["--option", option]);
    }
    six.extend(["--context", CTX]);
    refused(&work, &six, text);
    assert_eq!(entry_count(&work), 0);
}

#[test]
fn a_valid_decision_request_stores_the_context_the_options_in_order_and_needs_decision() {
    let work = Work::new();
    ok(
        &work,
        &[
            "ask",
            "I-1",
            "Remember the email?",
            "--option",
            "Yes, remember it",
            "--option",
            "No",
            "--option",
            "Ask each time",
            "--context",
            CTX,
        ],
    );
    let q = work.board()["entries"]["Q-1"].clone();
    assert_eq!(q["kind"], "question");
    assert_eq!(q["needs"], "decision");
    assert_eq!(q["context"], CTX);
    assert_eq!(
        q["options"],
        json!(["Yes, remember it", "No", "Ask each time"])
    );
    assert_eq!(q["status"], "open");
    let mut five = vec!["ask", "I-1", "Pick a theme"];
    for option in ["a", "b", "c", "d", "e"] {
        five.extend(["--option", option]);
    }
    five.extend(["--context", CTX]);
    ok(&work, &five);
    assert_eq!(
        work.board()["entries"]["Q-2"]["options"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
}

#[test]
fn a_plain_question_without_options_is_a_free_text_question_and_still_needs_a_context() {
    let work = Work::new();
    refused(
        &work,
        &["ask", "I-1", "What should the button say?"],
        "--context is required",
    );
    ok(
        &work,
        &[
            "ask",
            "I-1",
            "What should the button say?",
            "--context",
            CTX,
        ],
    );
    let q = work.board()["entries"]["Q-1"].clone();
    assert_eq!(q["needs"], "answer");
    assert_eq!(q["options"], json!([]));
    assert_eq!(q["context"], CTX);
}

#[test]
fn coord_decide_requires_a_context_too_and_stores_it() {
    let work = Work::new();
    refused(
        &work,
        &["decide", "I-1", "Keep the email field"],
        "--context is required",
    );
    ok(
        &work,
        &[
            "decide",
            "I-1",
            "Keep the email field",
            "--context",
            "The owner is away; I assume the email stays because removing it breaks login.",
        ],
    );
    let d = work.board()["entries"]["D-1"].clone();
    assert_eq!(d["kind"], "decision");
    assert!(
        d["context"]
            .as_str()
            .is_some_and(|c| c.contains("breaks login"))
    );
}

#[test]
fn notify_and_message_need_no_context_one_given_is_stored_and_clipped_to_600() {
    let work = Work::new();
    ok(&work, &["notify", "Build b1 is ready"]);
    ok(&work, &["message", "The review is done"]);
    assert_eq!(work.board()["entries"]["N-1"]["context"], json!(null));
    assert_eq!(work.board()["entries"]["M-1"]["context"], json!(null));
    ok(
        &work,
        &[
            "notify",
            "Build b2 is ready",
            "--context",
            "It has the new login form.",
        ],
    );
    ok(
        &work,
        &["message", "Heads up", "--context", &"y".repeat(900)],
    );
    assert_eq!(
        work.board()["entries"]["N-2"]["context"],
        "It has the new login form."
    );
    assert_eq!(
        work.board()["entries"]["M-2"]["context"]
            .as_str()
            .map(str::len),
        Some(600)
    );
}

#[test]
fn coord_entry_set_context_edits_it_and_a_question_keeps_the_20_character_minimum() {
    let work = Work::new();
    ok(
        &work,
        &[
            "ask",
            "I-1",
            "Remember the email?",
            "--option",
            "Yes",
            "--option",
            "No",
            "--context",
            CTX,
        ],
    );
    ok(&work, &["notify", "Build b1 is ready"]);
    ok(
        &work,
        &[
            "entry",
            "set",
            "Q-1",
            "--context",
            "Only the stored email is at stake, never the password. Default if unanswered: store nothing.",
        ],
    );
    let q = work.board()["entries"]["Q-1"].clone();
    assert!(
        q["context"]
            .as_str()
            .is_some_and(|c| c.contains("never the password"))
    );
    assert_eq!(q["text"], "Remember the email?");
    refused(
        &work,
        &["entry", "set", "Q-1", "--context", "short"],
        "minimum is 20",
    );
    refused(
        &work,
        &["entry", "set", "Q-1", "--option", "Only one"],
        "a decision needs 2 to 5 options",
    );
    ok(
        &work,
        &[
            "entry",
            "set",
            "N-1",
            "--context",
            "Short is fine on a notice.",
        ],
    );
    assert_eq!(
        work.board()["entries"]["N-1"]["context"],
        "Short is fine on a notice."
    );
    refused(&work, &["entry", "set", "Q-1"], "question text is required");
}

#[test]
fn the_printed_confirmation_of_an_ask_names_the_kind_it_made() {
    let work = Work::new();
    let free = ok(
        &work,
        &["ask", "I-1", "Another free question?", "--context", CTX],
    );
    assert!(
        free.contains("asked on I-1") && !free.contains("decision request"),
        "{free}"
    );
    let decision = ok(
        &work,
        &[
            "ask",
            "I-1",
            "Ship it?",
            "--option",
            "Ship",
            "--option",
            "Wait",
            "--context",
            CTX,
        ],
    );
    assert!(decision.contains("decision request"), "{decision}");
}
