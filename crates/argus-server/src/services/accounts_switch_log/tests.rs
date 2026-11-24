// test-tag: critical
//! The step log: one log per process, kept across writers, closed by the end of the switch, and
//! the doc in the shape the web reads.

use argus_accounts::desktop::{Step, StepStatus};
use argus_accounts::handover::{Cycle, CycleMode, CyclePhase, Handover, HandoverState};
use argus_core::clock::iso_from_ms;
use serde_json::{Value, json};

use super::{Line, LineState, LogState, Stage, cycle_lines, step_lines, threads_of};

fn doc(phase: &str) -> Value {
    json!({ "id": "SW-1", "phase": phase, "steps": [], "error": null })
}

fn texts(doc: &Value) -> Vec<String> {
    doc["steps"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|e| e["text"].as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn lines(items: &[&str]) -> Vec<Line> {
    items
        .iter()
        .map(|s| Line::new(Stage::Switch, LineState::Done, *s))
        .collect()
}

fn line_texts(lines: &[Line]) -> Vec<&str> {
    lines.iter().map(|l| l.text.as_str()).collect()
}

#[test]
fn steps_noted_before_the_countdown_reach_its_doc_and_every_later_write() {
    let mut log = LogState::default();
    log.add(1, &lines(&["Handover cycle SW-2 started."]));
    let countdown = log.stamp(doc("countdown"), 1);
    assert_eq!(texts(&countdown), ["Handover cycle SW-2 started."]);
    log.add(2, &lines(&["Close Claude: started."]));
    let again = log.again(false, 2).expect("a live doc");
    assert_eq!(texts(&again).len(), 2);
    assert_eq!(again["steps"][1]["at"], iso_from_ms(2));
    assert_eq!(again["steps"][1]["stage"], "switch");
    assert_eq!(again["steps"][1]["state"], "done");
}

#[test]
fn the_end_closes_the_log_and_the_next_process_starts_a_new_one() {
    let mut log = LogState::default();
    log.add(1, &lines(&["Countdown started."]));
    let done = log.stamp(doc("done"), 1);
    assert_eq!(texts(&done), ["Countdown started."]);
    assert_eq!(
        log.again(false, 2),
        None,
        "an ended doc is not written again"
    );
    log.add(3, &lines(&["Handover cycle SW-3 started."]));
    let next = doc("countdown");
    log.begin(&next);
    assert_eq!(texts(&log.stamp(next, 3)), ["Handover cycle SW-3 started."]);
}

#[test]
fn a_doc_with_an_error_ends_the_process() {
    let mut log = LogState::default();
    let mut failed = doc("closing");
    failed["error"] = json!("boom");
    log.stamp(failed, 1);
    log.add(2, &lines(&["new"]));
    assert_eq!(texts(&log.stamp(doc("countdown"), 2)), ["new"]);
}

#[test]
fn a_doc_of_another_switch_starts_a_new_log() {
    let mut log = LogState::default();
    log.add(1, &lines(&["old"]));
    log.stamp(doc("verifying"), 1);
    let mut other = doc("countdown");
    other["id"] = json!("SW-9");
    log.begin(&other);
    log.add(2, &lines(&["new"]));
    assert_eq!(texts(&log.stamp(other, 2)), ["new"]);
}

#[test]
fn resume_lines_join_the_finished_log() {
    let mut log = LogState::default();
    log.add(1, &lines(&["Done."]));
    log.stamp(doc("done"), 1);
    log.add_after(2, &lines(&["rotp/lead resumed on the new account."]));
    let again = log.again(true, 2).expect("the finished doc");
    assert_eq!(texts(&again).len(), 2);
    assert_eq!(again["phase"], "done");
}

#[test]
fn a_verifying_switch_ends_once_its_check_is_in() {
    let mut log = LogState::default();
    let shown = log.stamp(doc("verifying"), 1);
    assert_eq!(shown["phase"], "verifying");
    let check = Line::new(Stage::Verify, LineState::Done, "Sidebar check: same.");
    let done = log
        .finish("SW-1", ("done", None), 2, &[check])
        .expect("the switch doc");
    assert_eq!(done["phase"], "done");
    assert_eq!(done["steps"][0]["stage"], "verify");
    assert_eq!(log.finish("SW-7", ("done", None), 3, &[]), None);
}

#[test]
fn the_doc_follows_the_contract() {
    let mut log = LogState::default();
    let mut raw = doc("syncing");
    raw["countdownEndsAt"] = json!(60_000);
    raw["steps"] = json!([
        { "id": "close", "label": "Close Claude", "status": "done", "detail": "" },
        { "id": "sync", "label": "Sync", "status": "running", "detail": "" },
    ]);
    log.set_sync(json!({ "backupDir": "B" }));
    let shown = log.stamp(raw, 1);
    assert_eq!(shown["phase"], "sync");
    assert_eq!(shown["desktopPhase"], "syncing");
    assert_eq!(shown["desktopSteps"].as_array().unwrap().len(), 2);
    assert_eq!(shown["countdownEndsAt"], iso_from_ms(60_000));
    assert_eq!(
        shown["sync"],
        json!({ "backupDir": "B", "state": "running" })
    );
    assert_eq!(shown["threads"].as_array().unwrap().len(), 0);
    assert!(shown["blocker"].is_null());
    assert!(shown["error"].is_null());
    assert!(shown.get("log").is_none());
}

#[test]
fn a_failed_sync_step_gives_the_sync_its_error() {
    let mut log = LogState::default();
    let mut raw = doc("failed");
    raw["steps"] = json!([{ "id": "sync", "label": "Sync", "status": "failed", "detail": "boom" }]);
    let shown = log.stamp(raw, 1);
    assert_eq!(shown["sync"]["state"], "failed");
    assert_eq!(shown["sync"]["error"], "boom");
    assert_eq!(log.stamp(doc("countdown"), 2)["sync"]["state"], "pending");
}

#[test]
fn the_handover_phase_names_the_leads_it_waits_for() {
    let mut log = LogState::default();
    let found = cycle(
        CyclePhase::Asked,
        vec![
            handover("lead", HandoverState::Asked),
            handover("builder", HandoverState::Ready),
        ],
    );
    log.set_threads(threads_of(&found));
    let shown = log.stamp(doc("handover"), 1);
    assert_eq!(shown["phase"], "handover");
    assert_eq!(shown["threads"][0]["state"], "asked");
    assert_eq!(shown["threads"][1]["state"], "ready");
    assert_eq!(shown["blocker"], "Waiting for the handover of lead.");
}

#[test]
fn a_countdown_past_its_end_says_why_it_is_not_moving() {
    let mut log = LogState::default();
    let mut raw = doc("countdown");
    raw["countdownEndsAt"] = json!(1_000);
    assert!(log.stamp(raw.clone(), 999)["blocker"].is_null());
    assert!(log.stamp(raw, 1_000)["blocker"].is_string());
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
        step("sync", StepStatus::Failed, "boom"),
    ];
    let out = step_lines(&before, &after);
    assert_eq!(
        line_texts(&out),
        ["Step close: done.", "Step sync: failed."]
    );
    assert_eq!(out[0].stage, Stage::Switch);
    assert_eq!(out[0].detail.as_deref(), Some("closed"));
    assert_eq!(
        (out[1].stage, out[1].state),
        (Stage::Sync, LineState::Failed)
    );
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
        vec![handover("lead", HandoverState::Asked)],
    );
    let out = cycle_lines(None, Some(&after));
    assert_eq!(
        line_texts(&out.process),
        [
            "Handover cycle SW-2 started for hotmail: every lead is asked at once.",
            "rotp/lead was asked for its handover."
        ]
    );
    assert_eq!(out.process[1].state, LineState::Waiting);
    assert_eq!(out.resume.len(), 0);
}

#[test]
fn a_ready_lead_and_the_ready_cycle_are_logged_once() {
    let before = cycle(
        CyclePhase::Asked,
        vec![handover("lead", HandoverState::Asked)],
    );
    let after = cycle(
        CyclePhase::Ready,
        vec![handover("lead", HandoverState::Ready)],
    );
    let out = cycle_lines(Some(&before), Some(&after));
    assert_eq!(
        line_texts(&out.process),
        ["Handover received from rotp/lead.", "Every handover is in."]
    );
    assert_eq!(cycle_lines(Some(&after), Some(&after)).process.len(), 0);
}

#[test]
fn a_resume_goes_to_the_resume_lines() {
    let before = cycle(
        CyclePhase::Ready,
        vec![handover("lead", HandoverState::Ready)],
    );
    let after = cycle(
        CyclePhase::Done,
        vec![handover("lead", HandoverState::Resumed)],
    );
    let out = cycle_lines(Some(&before), Some(&after));
    assert_eq!(out.process.len(), 0);
    assert_eq!(
        line_texts(&out.resume),
        ["rotp/lead resumed on the new account."]
    );
    assert_eq!(out.resume[0].stage, Stage::Verify);
}
