//! E3: parity of the item, test, entry, action, lock, log and inbox verbs against the Node CLI.

// test-tag: e2e

use std::fs;

use crate::support::differ::{Differ, Engines, LEAD_SESSION, Step, WORKER_SESSION};

const CTX: &str =
    "The login form can remember the email, which changes what is stored on the machine.";

/// A step on the `alpha` board as the coordinator.
fn lead(words: &[&str]) -> Step {
    on_alpha(words).session(LEAD_SESSION)
}

/// A step on the `alpha` board as the worker thread.
fn worker(words: &[&str]) -> Step {
    on_alpha(words).session(WORKER_SESSION)
}

fn on_alpha(words: &[&str]) -> Step {
    let mut argv: Vec<&str> = words.to_vec();
    argv.extend(["--project", "alpha"]);
    Step::new(&argv)
}

/// Runs the steps on both engines and fails with the whole report on any difference.
fn compare(test: &str, steps: &[Step]) {
    compare_with(test, steps, |_| {});
}

fn compare_with(test: &str, steps: &[Step], prepare: impl Fn(&Differ)) {
    let Some(engines) = Engines::or_skip(test) else {
        return;
    };
    let differ = Differ::standard(engines).expect("twin homes");
    prepare(&differ);
    let report = differ.run(steps).expect("run the steps");
    println!("{report}");
    report.assert_no_mismatch();
    assert_eq!(report.not_ported(), 0, "a verb is not ported:\n{report}");
}

#[test]
#[ignore = "e2e"]
fn item_add_in_every_shape_and_every_refusal() {
    compare(
        "item_add",
        &[
            worker(&["item", "add", "Plain item"]),
            worker(&[
                "item",
                "add",
                "Full item",
                "--desc",
                "A longer text",
                "--ac",
                "First",
                "--ac",
                "Second",
                "--status",
                "doing",
            ]),
            lead(&["item", "add", "For the worker", "--for", "worker", "--top"]),
            lead(&["item", "add", "Placed", "--before", "I-1"]),
            lead(&["item", "add", "After", "--after", "I-2", "--bottom"]),
            worker(&["item", "add", "Bottom", "--bottom"]),
            worker(&["item", "add"]),
            worker(&["item", "add", &"t".repeat(81)]),
            worker(&["item", "add", &"t".repeat(80), "--desc", &"d".repeat(401)]),
            worker(&["item", "add", "Bad status", "--status", "wip"]),
            worker(&["item", "add", "Bad status", "--status"]),
            worker(&["item", "add", "No ac text", "--ac", "--ac", "x"]),
            worker(&["item", "add", "For the lead", "--for", "lead"]),
            lead(&["item", "add", "For nobody", "--for", "nobody"]),
            lead(&["item", "add", "Beside nobody", "--before", "I-99"]),
            worker(&[
                "item",
                "add",
                "Unicode \u{1F600} title",
                "--ac",
                "\u{e9}\u{e8} done",
            ]),
            Step::new(&["item", "add", "No session", "--project", "alpha"]),
            Step::new(&[
                "item",
                "add",
                "As worker",
                "--project",
                "alpha",
                "--as",
                "worker",
            ]),
            worker(&["item", "list"]),
            worker(&["item", "list", "--json"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn item_set_status_move_and_the_revision_check() {
    compare(
        "item_set",
        &[
            worker(&[
                "item", "set", "I-2", "--title", "Renamed", "--desc", "Why", "--status", "review",
            ]),
            worker(&["item", "set", "I-2", "--desc", ""]),
            worker(&["item", "set", "I-2"]),
            worker(&["item", "set", "I-2", "--title", &"x".repeat(81)]),
            worker(&["item", "set", "I-1", "--title", "Not mine"]),
            lead(&["item", "set", "I-1", "--title", "Mine now"]),
            worker(&["item", "status", "I-2", "done"]),
            worker(&["item", "status", "I-2", "finished"]),
            worker(&["item", "status", "I-2"]),
            worker(&["item", "status", "I-3", "doing", "--title", "Both"]),
            worker(&["item", "set", "I-3", "--status", "done", "--if-rev", "1"]),
            worker(&["item", "set", "I-3", "--status", "todo", "--if-rev", "1"]),
            worker(&["item", "set", "I-3", "--status", "todo", "--if-rev", "x"]),
            worker(&["item", "set", "I-99", "--title", "Gone"]),
            worker(&["item", "set", "--title", "No id"]),
            worker(&["item", "move", "I-3", "--status", "review"]),
            worker(&["item", "move", "I-3", "--top"]),
            worker(&["item", "move", "I-3", "--before", "I-1"]),
            worker(&["item", "move", "I-3"]),
            worker(&["item", "move", "I-3", "--to", "lead"]),
            lead(&["item", "move", "I-3", "--to", "lead", "--bottom"]),
            lead(&["item", "move", "I-3", "--to", "nobody"]),
            lead(&["item", "move", "I-3", "--status", "nope"]),
            lead(&["item", "move", "I-3", "--after", "I-3"]),
            lead(&["item", "show", "I-3"]),
            lead(&["item", "list", "--status", "review"]),
            lead(&["item", "list", "--type", "task"]),
            worker(&["item", "list", "--mine"]),
            lead(&["item", "list", "--for", "worker", "--json"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn item_acceptance_criteria_every_verb() {
    compare(
        "item_ac",
        &[
            worker(&["item", "ac", "I-2", "add", "Second case"]),
            worker(&["item", "ac", "I-2", "Legacy add"]),
            worker(&["item", "ac", "I-2", "add"]),
            worker(&["item", "ac", "I-2", "add", &"c".repeat(121)]),
            worker(&["item", "ac", "I-2", "set", "2", "Reworded"]),
            worker(&["item", "ac", "I-2", "set", "AC3", "Spelled"]),
            worker(&["item", "ac", "I-2", "set", "x", "Bad"]),
            worker(&["item", "ac", "I-2", "set", "9", "Out of range"]),
            worker(&["item", "ac", "I-2", "done", "1"]),
            worker(&["item", "ac", "I-2", "done", "1", "--undo"]),
            worker(&["item", "ac", "I-2", "done", "ac2"]),
            worker(&["item", "ac-done", "I-2", "3"]),
            worker(&["item", "ac", "I-2", "move", "3", "1"]),
            worker(&["item", "ac", "I-2", "move", "1"]),
            worker(&["item", "ac", "I-2", "rm", "2"]),
            worker(&["item", "ac", "rm", "I-2", "1"]),
            worker(&["item", "ac", "I-2", "rm", "9"]),
            worker(&["item", "ac", "I-2", "rm", "1-9"]),
            worker(&["item", "ac", "I-2", "rm", "zero"]),
            worker(&["item", "ac-rm", "I-2", "1"]),
            lead(&["item", "ac", "I-3", "add", "One", "--if-rev", "1"]),
            lead(&["item", "ac", "I-3", "add", "Two"]),
            lead(&["item", "ac", "I-3", "set", "1", "Changed", "--if-rev", "1"]),
            lead(&["item", "ac", "I-3", "rm", "1-2"]),
            worker(&["item", "ac", "I-1", "add", "Not mine"]),
            worker(&["item", "ac", "I-99", "add", "Gone"]),
            worker(&["item", "show", "I-2"]),
            worker(&["item", "show", "I-3"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn item_soft_verbs_and_unknown_sub_verbs() {
    compare(
        "item_soft",
        &[
            worker(&["item", "archive", "I-2"]),
            worker(&["item", "archive", "I-2"]),
            worker(&["item", "list"]),
            worker(&["item", "list", "--archived"]),
            worker(&["item", "set", "I-2", "--title", "On archived"]),
            worker(&["item", "unarchive", "I-2"]),
            worker(&["item", "unarchive", "I-2"]),
            worker(&["item", "rm", "I-2", "--reason", "duplicate of I-3"]),
            worker(&["item", "set", "I-2", "--title", "In the trash"]),
            worker(&["item", "list", "--deleted"]),
            worker(&["item", "show", "I-2"]),
            worker(&["item", "restore", "I-2"]),
            worker(&["item", "restore", "I-2"]),
            worker(&["item", "rm", "I-2", "I-3"]),
            worker(&["item", "rm", "I-2", "I-1"]),
            worker(&["item", "rm", "I-2", "--reason", &"r".repeat(121)]),
            worker(&["item", "archive"]),
            worker(&["item", "archive", "I-1"]),
            lead(&["item", "archive", "I-1", "--reason", "done with it"]),
            worker(&["item", "bogus"]),
            worker(&["item"]),
            lead(&["item", "rm", "I-3", "--if-rev", "abc"]),
            lead(&["item", "list", "--deleted", "--json"]),
        ],
    );
}

fn test_add_set_and_the_parts_first() -> Vec<Step> {
    vec![
        worker(&[
            "test",
            "add",
            "I-2",
            "Parser cases",
            "--mode",
            "manual",
            "--build",
            "b1",
            "--ac",
            "1",
            "--covers",
            "The parser",
            "--run",
            "cargo test",
            "--context",
            "Why it matters",
            "--changed",
            "a.rs",
            "--changed",
            "b.rs",
            "--note",
            "n1",
        ]),
        worker(&["test", "add", "I-2"]),
        worker(&["test", "add", "I-2", "Bad mode", "--mode", "fast"]),
        worker(&["test", "add", "I-2", "Bad ac", "--ac", "7"]),
        worker(&["test", "add", "I-2", "Bad build", "--build"]),
        worker(&["test", "add", "I-1", "Not mine"]),
        worker(&["test", "add", "I-99", "Gone"]),
        worker(&[
            "test",
            "add",
            "I-2",
            "Cleared",
            "--build",
            "b2",
            "--clear-build",
        ]),
        worker(&["test", "set", "T-1", "--title", "Renamed", "--build", ""]),
        worker(&["test", "set", "T-1", "--ac", "1", "--ac", "1"]),
        worker(&["test", "set", "T-1"]),
        worker(&["test", "set", "T-1", "--note", &"n".repeat(301)]),
        worker(&["test", "step", "T-1", "Open the page"]),
        worker(&["test", "step", "T-1", "Click it"]),
        worker(&["test", "step", "T-1", "set", "1", "Open the app"]),
        worker(&["test", "step", "T-1", "set", "9", "Nope"]),
    ]
}

fn test_add_set_and_the_parts_second() -> Vec<Step> {
    vec![
        worker(&["test", "step", "T-1", "rm", "2"]),
        worker(&["test", "step", "T-1", "rm", "5"]),
        worker(&["test", "step", "T-1"]),
        worker(&["test", "setup", "T-1", "Server", "Start it first"]),
        worker(&["test", "setup", "T-1", "Data", "Load the fixture"]),
        worker(&["test", "setup", "T-1", "Only label"]),
        worker(&["test", "setup", "T-1", "rm", "1"]),
        worker(&["test", "setup", "T-1", "rm", "4"]),
        worker(&[
            "test",
            "expect",
            "T-1",
            "--pass",
            "It opens",
            "--pass",
            "It closes",
            "--fail",
            "It crashes",
        ]),
        worker(&["test", "expect", "T-1", "rm", "--pass", "1"]),
        worker(&["test", "expect", "T-1", "rm", "--fail", "1", "--pass", "1"]),
        worker(&["test", "expect", "T-1", "rm"]),
        worker(&["test", "expect", "T-1", "rm", "--pass", "5"]),
        worker(&["test", "expect", "T-1", "--pass", &"p".repeat(201)]),
        worker(&["test", "show", "T-1"]),
        worker(&["test", "list"]),
        worker(&["test", "list", "--json"]),
    ]
}

#[test]
#[ignore = "e2e"]
fn test_add_set_and_the_parts() {
    compare(
        "test_parts",
        &[
            test_add_set_and_the_parts_first(),
            test_add_set_and_the_parts_second(),
        ]
        .concat(),
    );
}

#[test]
#[ignore = "e2e"]
fn test_results_move_and_soft_verbs() {
    compare(
        "test_results",
        &[
            worker(&["test", "add", "I-2", "First", "--ac", "1"]),
            worker(&["test", "add", "I-3", "Second"]),
            worker(&["test", "result", "T-1", "pass", "All green"]),
            worker(&["test", "result", "T-1", "fail", "Broke later"]),
            worker(&["test", "result", "T-1", "partial", "Half"]),
            worker(&["test", "result", "T-1", "blocked", "Waiting"]),
            worker(&["test", "result", "T-1", "bogus", "Nope"]),
            worker(&["test", "result", "T-1", "pass"]),
            worker(&["test", "result", "T-1", "pass", &"r".repeat(301)]),
            lead(&["test", "result", "T-2", "pass", "Lead ran it"]),
            worker(&["test", "move", "T-1", "--item", "I-3", "--ac", "1"]),
            worker(&["test", "move", "T-1", "--item", "I-3", "--ac", "9"]),
            worker(&["test", "move", "T-1", "--item", "I-1"]),
            worker(&["test", "move", "T-1"]),
            worker(&["test", "move", "T-2", "--item", "I-2"]),
            worker(&["test", "archive", "T-2"]),
            worker(&["test", "list"]),
            worker(&["test", "list", "--archived"]),
            worker(&["test", "unarchive", "T-2"]),
            worker(&["test", "rm", "T-2", "--reason", "obsolete"]),
            worker(&["test", "result", "T-2", "pass", "In the trash"]),
            worker(&["test", "restore", "T-2"]),
            worker(&["test", "list", "--item", "I-3", "--mine"]),
            worker(&["test", "list", "--build"]),
            worker(&["test", "list", "--build", "none"]),
            worker(&["test", "bogus"]),
            worker(&["item", "show", "I-2"]),
            worker(&["item", "show", "I-3"]),
            worker(&["log", "list"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn test_proofs_are_kept_next_to_the_board() {
    let steps = [
        worker(&["test", "add", "I-2", "With proof"]),
        worker(&["test", "proof", "T-1", "shot.png", "The login page"]),
        worker(&["test", "proof", "T-1", "shot.PNG", "Again"]),
        worker(&["test", "proof", "T-1", "shot.txt", "Not an image"]),
        worker(&["test", "proof", "T-1", "missing.png", "Not there"]),
        worker(&["test", "proof", "T-1", "shot.png"]),
        worker(&["test", "proof", "T-1"]),
        worker(&["test", "proof", "T-1", "rm", "1"]),
        worker(&["test", "proof", "T-1", "rm", "7"]),
        worker(&[
            "test",
            "result",
            "T-1",
            "fail",
            "See picture",
            "--proof",
            "shot.png",
            "--caption",
            "The error",
        ]),
        worker(&[
            "test",
            "result",
            "T-1",
            "pass",
            "Two pictures",
            "--proof",
            "shot.png",
            "--proof",
            "shot.png",
            "--caption",
            "One",
        ]),
        worker(&["test", "show", "T-1"]),
    ];
    compare_with("test_proofs", &steps, |differ| {
        for home in [&differ.twin().node, &differ.twin().rust] {
            fs::write(home.work().join("shot.png"), b"\x89PNG proof bytes").expect("png");
            fs::write(home.work().join("shot.PNG"), b"\x89PNG upper").expect("png");
            fs::write(home.work().join("shot.txt"), b"text").expect("txt");
        }
    });
}

#[test]
#[ignore = "e2e"]
fn test_spec_files_are_merged_under_the_flags() {
    let spec = r#"{"title":"From file","steps":["one","two"],"pass":["ok"],"mode":"manual","setup":[{"label":"L","text":"T"}],"covers":null}"#;
    let steps = [
        worker(&[
            "test",
            "add",
            "I-2",
            "Positional title",
            "--file",
            "spec.json",
        ]),
        worker(&[
            "test",
            "add",
            "I-2",
            "--file",
            "spec.json",
            "--title",
            "Flag title",
        ]),
        worker(&[
            "test",
            "set",
            "T-1",
            "--file",
            "spec.json",
            "--mode",
            "auto",
        ]),
        worker(&["test", "add", "I-2", "--file", "absent.json"]),
        worker(&["test", "show", "T-1"]),
        worker(&["test", "show", "T-2"]),
    ];
    compare_with("test_spec_files", &steps, |differ| {
        for home in [&differ.twin().node, &differ.twin().rust] {
            fs::write(home.work().join("spec.json"), spec).expect("spec");
        }
    });
}

fn entries_ask_decide_notify_message_first() -> Vec<Step> {
    vec![
        worker(&[
            "ask",
            "I-2",
            "Remember the email?",
            "--option",
            "Yes",
            "--option",
            "No",
            "--option",
            "Ask",
            "--context",
            CTX,
        ]),
        worker(&["ask", "I-2", "Free question?", "--context", CTX]),
        worker(&[
            "ask",
            "I-2",
            "Forced?",
            "--decide",
            "--option",
            "A",
            "--option",
            "B",
            "--context",
            CTX,
        ]),
        worker(&["ask", "I-2", "No context?"]),
        worker(&["ask", "I-2", "No context?", "--context"]),
        worker(&["ask", "I-2", "Short?", "--context", "too short"]),
        worker(&["ask", "I-2", "Long?", "--context", &"x".repeat(601)]),
    ]
}

fn entries_ask_decide_notify_message_first_more() -> Vec<Step> {
    vec![
        worker(&["ask", "I-2", "No options?", "--decide", "--context", CTX]),
        worker(&[
            "ask",
            "I-2",
            "One option?",
            "--option",
            "A",
            "--context",
            CTX,
        ]),
        worker(&[
            "ask",
            "I-2",
            "Six options?",
            "--option",
            "a",
            "--option",
            "b",
            "--option",
            "c",
            "--option",
            "d",
            "--option",
            "e",
            "--option",
            "f",
            "--context",
            CTX,
        ]),
        worker(&["ask", "I-2", &"q".repeat(241), "--context", CTX]),
        worker(&["ask", "-", "No item?", "--context", CTX]),
        worker(&["ask", "I-99", "Gone item?", "--context", CTX]),
        worker(&["ask", "I-1", "Not my item?", "--context", CTX]),
        lead(&["ask", "I-2", "Lead asks on it", "--context", CTX]),
    ]
}

fn entries_ask_decide_notify_message_second() -> Vec<Step> {
    vec![
        worker(&["decide", "I-2", "Keep the field", "--context", CTX]),
        worker(&["decide", "I-2", "No context"]),
        worker(&["decide", "I-2", &"d".repeat(201), "--context", CTX]),
        worker(&["notify", "A notice"]),
        worker(&[
            "notify",
            "A notice with context",
            "--context",
            "Short is fine",
            "--level",
            "warn",
            "--item",
            "I-2",
        ]),
        worker(&["notify", "Bad level", "--level", "loud"]),
        worker(&["notify", "Two", "words"]),
        worker(&["notify"]),
        worker(&["notify", &"n".repeat(141)]),
        worker(&["notify", "Bad item", "--item", "I-99"]),
        worker(&[
            "message",
            "A message for the owner",
            "--context",
            &"y".repeat(900),
        ]),
        worker(&["message", "overlord", "text"]),
        worker(&["message", "With item", "--item", "I-3"]),
        worker(&["entry", "list"]),
        worker(&["entry", "list", "--json"]),
    ]
}

#[test]
#[ignore = "e2e"]
fn entries_ask_decide_notify_message() {
    compare(
        "entry_make",
        &[
            entries_ask_decide_notify_message_first(),
            entries_ask_decide_notify_message_first_more(),
            entries_ask_decide_notify_message_second(),
        ]
        .concat(),
    );
}

fn entries_answer_pick_done_set_move_and_soft_verbs_first() -> Vec<Step> {
    vec![
        worker(&[
            "ask",
            "I-2",
            "Remember?",
            "--option",
            "Yes",
            "--option",
            "No",
            "--context",
            CTX,
        ]),
        worker(&["notify", "Heads up"]),
        worker(&["message", "Hello owner"]),
        worker(&["decide", "I-2", "We keep it", "--context", CTX]),
        worker(&["answer", "Q-1", "Yes please"]),
        lead(&["answer", "Q-1", "Yes please"]),
        lead(&["answer", "Q-1", "A decision", "--decision"]),
        lead(&["answer", "M-1", "Reply to message"]),
        lead(&["answer", "N-1", "Reply to notice"]),
        lead(&["answer", "Q-99", "Gone"]),
        lead(&["answer", "Q-1", &"a".repeat(241)]),
        lead(&["answer", "Q-1", &"a".repeat(201), "--decision"]),
        worker(&["entry", "pick", "F-1"]),
        worker(&["entry", "pick", "D-2"]),
        worker(&["entry", "pick", "Q-1"]),
        worker(&["entry", "done", "F-1"]),
        worker(&["entry", "done", "D-1"]),
        worker(&["entry", "done", "Q-1"]),
        worker(&["entry", "done", "N-1"]),
        lead(&["entry", "done", "M-1"]),
        worker(&["entry", "set", "N-1", "Reworded notice"]),
    ]
}

fn entries_answer_pick_done_set_move_and_soft_verbs_second() -> Vec<Step> {
    vec![
        worker(&[
            "entry",
            "set",
            "Q-1",
            "Reworded question",
            "--option",
            "A",
            "--option",
            "B",
            "--option",
            "C",
            "--context",
            CTX,
        ]),
        worker(&["entry", "set", "Q-1"]),
        worker(&["entry", "set", "Q-1", "--option", "Only"]),
        worker(&["entry", "set", "N-1", "--option", "A"]),
        worker(&["entry", "set", "F-1", "Not a worker entry"]),
        lead(&["entry", "set", "F-1", "Lead rewords it"]),
        lead(&["entry", "set", "F-1", "--context", "short"]),
        worker(&["entry", "move", "N-1", "--item", "I-3"]),
        worker(&["entry", "move", "N-1", "--item", "I-1"]),
        worker(&["entry", "move", "N-1"]),
        worker(&["entry", "archive", "N-1"]),
        worker(&["entry", "unarchive", "N-1"]),
        worker(&["entry", "rm", "N-1", "--reason", "noise"]),
        worker(&["entry", "restore", "N-1"]),
        worker(&["entry", "show", "Q-1"]),
        worker(&["entry", "show", "Q-99"]),
        worker(&["entry", "list", "--open"]),
        worker(&["entry", "list", "--kind", "question", "--mine"]),
        worker(&["entry", "list", "--deleted"]),
        worker(&["entry", "bogus"]),
        worker(&["entry"]),
        worker(&["inbox"]),
    ]
}

#[test]
#[ignore = "e2e"]
fn entries_answer_pick_done_set_move_and_soft_verbs() {
    compare(
        "entry_change",
        &[
            entries_answer_pick_done_set_move_and_soft_verbs_first(),
            entries_answer_pick_done_set_move_and_soft_verbs_second(),
        ]
        .concat(),
    );
}

#[test]
#[ignore = "e2e"]
fn actions_add_set_done_move_and_soft_verbs() {
    compare(
        "action_all",
        &[
            worker(&["action", "add", "Approve the PR"]),
            worker(&["todo", "add", "Plug in the device", "--item", "I-2"]),
            lead(&["action", "add", "Sign in", "--for", "worker", "--top"]),
            lead(&["action", "add", "Placed", "--before", "H-1"]),
            worker(&["action", "add"]),
            worker(&["action", "add", &"a".repeat(201)]),
            worker(&["action", "add", "Bad item", "--item", "I-99"]),
            worker(&["action", "add", "For the lead", "--for", "lead"]),
            worker(&["action", "list"]),
            worker(&[
                "action",
                "set",
                "H-1",
                "--text",
                "Approve the big PR",
                "--item",
                "I-3",
            ]),
            worker(&["action", "set", "H-1"]),
            worker(&["action", "set", "H-1", "--item", ""]),
            worker(&["action", "set", "H-1", "--item", "I-99"]),
            worker(&["action", "set", "H-1", "--text", "x", "--if-rev", "1"]),
            worker(&["action", "done", "H-1"]),
            worker(&["action", "done", "H-1"]),
            worker(&["action", "done", "H-1", "--undo"]),
            lead(&["action", "done", "H-2"]),
            worker(&["action", "move", "H-1", "--bottom"]),
            worker(&["action", "move", "H-1", "--to", "lead"]),
            lead(&["action", "move", "H-1", "--to", "lead"]),
            lead(&["action", "move", "H-1"]),
            lead(&["action", "move", "H-1", "--to", "nobody"]),
            worker(&["action", "archive", "H-2"]),
            worker(&["action", "list", "--archived"]),
            worker(&["action", "unarchive", "H-2"]),
            worker(&["action", "rm", "H-2", "--reason", "not needed"]),
            worker(&["action", "list", "--deleted"]),
            worker(&["action", "restore", "H-2"]),
            worker(&["action", "show", "H-2"]),
            worker(&["action", "show", "H-99"]),
            worker(&["action", "list", "--mine", "--json"]),
            lead(&["action", "list", "--for", "worker"]),
            worker(&["action", "bogus"]),
            lead(&["item", "list", "--for", "worker"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn locks_claim_queue_release_free_drop_and_remove() {
    compare(
        "lock_all",
        &[
            worker(&["lock", "list"]),
            worker(&["lock", "claim", "Wasm Build", "--desc", "Building the wasm"]),
            worker(&["lock", "claim", "wasm-build"]),
            lead(&["lock", "claim", "wasm-build", "--desc", "Second claim"]),
            lead(&["lock", "claim", "wasm-build"]),
            worker(&["lock", "claim", "7"]),
            worker(&["lock", "claim", "alpha"]),
            worker(&["lock", "claim", "2"]),
            worker(&["lock", "claim"]),
            worker(&["lock", "claim", "x", "--desc", &"d".repeat(401)]),
            worker(&["lock", "list"]),
            lead(&["lock", "drop", "wasm-build"]),
            lead(&["lock", "drop", "nothing"]),
            lead(&["lock", "claim", "wasm-build"]),
            worker(&["lock", "release", "wasm-build"]),
            worker(&["lock", "release", "wasm-build"]),
            worker(&["lock", "release", "alpha"]),
            lead(&["lock", "release", "wasm-build"]),
            lead(&["lock", "release", "wasm-build"]),
            lead(&["lock", "release", "nothing"]),
            lead(&["lock", "claim", "alpha"]),
            worker(&["lock", "claim", "alpha"]),
            worker(&["lock", "free", "alpha"]),
            lead(&["lock", "free", "alpha"]),
            lead(&["lock", "free", "7", "--to", "lead"]),
            lead(&["lock", "free", "7", "--to", "nobody"]),
            lead(&["lock", "free", "nothing"]),
            worker(&["lock", "rm", "2"]),
            lead(&["lock", "rm", "2"]),
            lead(&["lock", "rm", "nothing"]),
            lead(&["lock", "list"]),
            lead(&["inbox", "--ack"]),
            worker(&["inbox"]),
            worker(&["lock", "bogus"]),
            worker(&["lock"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn log_write_list_and_remove_with_an_audit_line() {
    compare(
        "log_all",
        &[
            worker(&["log", "Wrote the parser"]),
            worker(&["log", "Second line", "--item", "I-2"]),
            worker(&["log", "Other subject", "--subject", "Core Parser"]),
            worker(&["log", "Numbered subject", "--subject", "5"]),
            worker(&["log", "Numbered again", "--subject", "10"]),
            worker(&["log", "Numbered first", "--subject", "2"]),
            lead(&["log", "From the lead", "--item", "I-1"]),
            worker(&["log"]),
            worker(&["log", &"l".repeat(121)]),
            worker(&["log", "Bad item", "--item", "I-99"]),
            worker(&["log", "list"]),
            worker(&["log", "list", "--n", "1"]),
            worker(&["log", "list", "--n", "0"]),
            worker(&["log", "list", "--n", "-1"]),
            worker(&["log", "list", "--n", "x"]),
            worker(&["log", "list", "--subject", "Core Parser"]),
            worker(&["log", "list", "--subject", "nothing"]),
            worker(&["log", "rm", "general", "1"]),
            worker(&["log", "rm", "general", "9"]),
            worker(&["log", "rm", "general"]),
            worker(&["log", "rm", "general", "x"]),
            worker(&["log", "rm", "plan", "1"]),
            lead(&["log", "rm", "core-parser", "1"]),
            lead(&["log", "rm", "plan", "1"]),
            lead(&["log", "rm", "audit", "1"]),
            worker(&["log", "list"]),
            worker(&["inbox"]),
            worker(&["inbox", "--ack"]),
        ],
    );
}

#[test]
#[ignore = "e2e"]
fn inbox_read_ack_send_and_remove() {
    compare(
        "inbox_all",
        &[
            worker(&["inbox"]),
            worker(&["inbox", "list"]),
            lead(&["inbox"]),
            lead(&["inbox", "list", "--for", "worker"]),
            worker(&["inbox", "list", "--for", "lead"]),
            lead(&["inbox", "send", "worker", "Build is ready"]),
            lead(&[
                "inbox",
                "send",
                "worker",
                "Second",
                "--kind",
                "lock-granted",
            ]),
            lead(&["inbox", "send", "worker", &"s".repeat(240)]),
            lead(&["inbox", "send", "worker", &"s".repeat(241)]),
            lead(&["inbox", "send", "worker", "   "]),
            lead(&["inbox", "send", "nobody", "x"]),
            lead(&["inbox", "send", "lead", "To myself"]),
            lead(&["inbox", "send", "worker"]),
            lead(&["inbox", "send"]),
            worker(&["inbox", "send", "lead", "Worker to lead"]),
            worker(&["inbox"]),
            lead(&["inbox"]),
            lead(&["inbox", "rm", "m-0-nothing", "--for", "worker"]),
            lead(&["inbox", "rm"]),
            worker(&["inbox", "rm", "--all", "--for", "lead"]),
            lead(&["inbox", "rm", "--all", "--for", "nobody"]),
            lead(&["inbox", "--ack", "--for", "worker"]),
            lead(&["inbox", "--ack", "--for", "worker"]),
            lead(&["inbox", "--ack"]),
            lead(&["inbox", "rm", "--all"]),
            worker(&["inbox", "--ack", "--for", "lead"]),
            worker(&["inbox"]),
            Step::new(&["inbox", "--project", "alpha"]),
            Step::new(&["inbox", "--project", "alpha", "--as", "worker"]),
            Step::new(&["inbox", "--project", "nowhere", "--as", "worker"]),
        ],
    );
}
