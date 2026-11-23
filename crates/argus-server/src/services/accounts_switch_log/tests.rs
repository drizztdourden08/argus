// test-tag: critical
//! The step log: one log per process, kept across writers, closed by the end of the switch.

use argus_accounts::desktop::{Step, StepStatus};
use argus_accounts::handover::{Cycle, CycleMode, CyclePhase, Handover, HandoverState};
use serde_json::{Value, json};

use super::{LogState, cycle_lines, step_lines};

fn doc(phase: &str) -> Value {
    json!({ "id": "SW-1", "phase": phase, "steps": [], "error": null })
}

fn texts(doc: &Value) -> Vec<String> {
    doc["log"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|e| e["text"].as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn lines(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn steps_noted_before_the_countdown_reach_its_doc_and_every_later_write() {
    let mut log = LogState::default();
    log.add(1, &lines(&["Handover cycle SW-2 started."]));
    let countdown = log.stamp(doc("countdown"));
    assert_eq!(texts(&countdown), ["Handover cycle SW-2 started."]);
    log.add(2, &lines(&["Close Claude: started."]));
    let again = log.again(false).expect("a live doc");
    assert_eq!(texts(&again).len(), 2);
    assert_eq!(again["log"][1]["at"], 2);
}

#[test]
fn the_end_closes_the_log_and_the_next_process_starts_a_new_one() {
    let mut log = LogState::default();
    log.add(1, &lines(&["Countdown started."]));
    let done = log.stamp(doc("done"));
    assert_eq!(texts(&done), ["Countdown started."]);
    assert_eq!(log.again(false), None, "an ended doc is not written again");
    log.add(3, &lines(&["Handover cycle SW-3 started."]));
    let next = log.stamp(doc("countdown"));
    assert_eq!(texts(&next), ["Handover cycle SW-3 started."]);
}

#[test]
fn a_doc_with_an_error_ends_the_process() {
    let mut log = LogState::default();
    let mut failed = doc("closing");
    failed["error"] = json!("boom");
    log.stamp(failed);
    log.add(2, &lines(&["new"]));
    assert_eq!(texts(&log.stamp(doc("countdown"))), ["new"]);
}

#[test]
fn resume_lines_join_the_finished_log() {
    let mut log = LogState::default();
    log.add(1, &lines(&["Done."]));
    log.stamp(doc("done"));
    log.add_after(2, &lines(&["rotp/coordinator resumed on the new account."]));
    let again = log.again(true).expect("the finished doc");
    assert_eq!(texts(&again).len(), 2);
    assert_eq!(again["phase"], "done");
}

fn step(id: &'static str, status: StepStatus, detail: &str) -> Step {
    Step {
        id,
        label: format!("Step {id}"),
        status,
        detail: detail.to_owned(),
    }
}

#[test]
fn a_desktop_step_makes_a_line_only_when_its_status_changes() {
    let before = [
        step("close", StepStatus::Running, ""),
        step("sync", StepStatus::Pending, ""),
    ];
    let after = [
        step("close", StepStatus::Done, "closed"),
        step("sync", StepStatus::Pending, ""),
    ];
    assert_eq!(step_lines(&before, &after), ["Step close: done (closed)."]);
    assert_eq!(step_lines(&after, &after).len(), 0);
}

fn handover(thread: &str, state: HandoverState) -> Handover {
    Handover {
        id: "H-1".to_owned(),
        cycle: "SW-2".to_owned(),
        board: "rotp".to_owned(),
        thread: thread.to_owned(),
        session_id: None,
        link: None,
        state,
        estimate: None,
        file: None,
        agents: Vec::new(),
        estimate_asked_at: None,
        asked_at: None,
        ready_at: None,
    }
}

fn cycle(phase: CyclePhase, leads: Vec<Handover>) -> Cycle {
    Cycle {
        id: "SW-2".to_owned(),
        account: "hotmail".to_owned(),
        mode: CycleMode::Now,
        phase,
        started_at: "2026-11-23T00:00:00Z".to_owned(),
        ask_at: None,
        k: 1.0,
        use_at_request: None,
        use_at_ready: None,
        leads,
    }
}

#[test]
fn a_new_cycle_logs_its_start_and_every_lead_asked() {
    let after = cycle(
        CyclePhase::Asked,
        vec![handover("coordinator", HandoverState::Asked)],
    );
    let out = cycle_lines(None, Some(&after));
    assert_eq!(
        out.process,
        [
            "Handover cycle SW-2 started for hotmail: every lead is asked at once.",
            "rotp/coordinator was asked for its handover."
        ]
    );
    assert_eq!(out.resume.len(), 0);
}

#[test]
fn a_ready_lead_and_the_ready_cycle_are_logged_once() {
    let before = cycle(
        CyclePhase::Asked,
        vec![handover("coordinator", HandoverState::Asked)],
    );
    let after = cycle(
        CyclePhase::Ready,
        vec![handover("coordinator", HandoverState::Ready)],
    );
    let out = cycle_lines(Some(&before), Some(&after));
    assert_eq!(
        out.process,
        [
            "Handover received from rotp/coordinator.",
            "Every handover is in."
        ]
    );
    assert_eq!(cycle_lines(Some(&after), Some(&after)).process.len(), 0);
}

#[test]
fn a_resume_goes_to_the_resume_lines() {
    let before = cycle(
        CyclePhase::Ready,
        vec![handover("coordinator", HandoverState::Ready)],
    );
    let after = cycle(
        CyclePhase::Done,
        vec![handover("coordinator", HandoverState::Resumed)],
    );
    let out = cycle_lines(Some(&before), Some(&after));
    assert_eq!(out.process.len(), 0);
    assert_eq!(out.resume, ["rotp/coordinator resumed on the new account."]);
}
