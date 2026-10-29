//! Example differential tests: the same verbs on the Node CLI and on `argus coord`, in twin temp
//! homes seeded with the standard fixture. A verb the Argus side does not serve yet shows as
//! "rust: not ported (120)" in the report and passes; the day it is ported the same test becomes
//! a byte for byte comparison. They skip with a reason when node, `ai-config` or the `argus`
//! binary is absent (set `ARGUS_EXE`, or build `-p argus`).

mod support;

use support::differ::{Differ, Engines, LEAD_SESSION, Report, Step, WORKER_SESSION};

fn run(test: &str, steps: &[Step]) -> Option<Report> {
    let engines = Engines::or_skip(test)?;
    let differ = Differ::standard(engines).expect("twin homes");
    let report = differ.run(steps).expect("run the steps");
    println!("{report}");
    Some(report)
}

#[test]
fn read_verbs_list_show_and_refuse() {
    let steps = [
        Step::new(&["item", "list", "--project", "alpha"]).session(LEAD_SESSION),
        Step::new(&["item", "show", "I-1", "--project", "alpha"]).session(LEAD_SESSION),
        Step::new(&["item", "show", "I-99", "--project", "alpha"]).session(LEAD_SESSION),
        Step::new(&["thread", "list", "--project", "alpha"]).session(LEAD_SESSION),
    ];
    let Some(report) = run("read_verbs_list_show_and_refuse", &steps) else {
        return;
    };
    report.assert_no_mismatch();
    assert_eq!(report.steps.len(), steps.len());
}

#[test]
fn write_verbs_add_move_and_read_back() {
    let steps = [
        Step::new(&[
            "item",
            "add",
            "Ship the harness",
            "--ac",
            "Differs",
            "--project",
            "alpha",
        ])
        .session(WORKER_SESSION),
        Step::new(&["item", "status", "I-4", "doing", "--project", "alpha"])
            .session(WORKER_SESSION),
        Step::new(&["log", "Wrote the harness", "--project", "alpha"]).session(WORKER_SESSION),
        Step::new(&["item", "show", "I-4", "--project", "alpha"]).session(WORKER_SESSION),
    ];
    let Some(report) = run("write_verbs_add_move_and_read_back", &steps) else {
        return;
    };
    report.assert_no_mismatch();
}

#[test]
fn inbox_and_a_hook_with_stdin() {
    let prompt =
        format!(r#"{{"session_id":"{WORKER_SESSION}","hook_event_name":"UserPromptSubmit"}}"#);
    let steps = [
        Step::new(&["inbox", "--project", "alpha"]).session(WORKER_SESSION),
        Step::new(&["hook", "prompt"])
            .session(WORKER_SESSION)
            .stdin(&prompt),
        Step::new(&["inbox", "--ack", "--project", "alpha"]).session(WORKER_SESSION),
    ];
    let Some(report) = run("inbox_and_a_hook_with_stdin", &steps) else {
        return;
    };
    report.assert_no_mismatch();
}

#[test]
fn router_answers_that_exist_on_both_sides_are_compared_for_real() {
    let steps = [
        Step::new(&["--help"]),
        Step::new(&["help"]),
        Step::new(&["frobnicate"]),
    ];
    let Some(report) = run("router_answers_compared_for_real", &steps) else {
        return;
    };
    report.assert_no_mismatch();
    assert_eq!(report.not_ported(), 0, "{report}");
}
