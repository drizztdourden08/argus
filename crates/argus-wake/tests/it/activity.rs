//! KEEP: K2.
//! A fault store that writes one activity line for every fault that opens or clears.

// test-tag: critical

use std::sync::Arc;

use argus_wake::activity::{ActivityFaults, Level, cleared_line, fault_line};
use argus_wake::faults::{ClearFault, Fault, FaultStore, NewFault};
use argus_wake::memory::{MemoryActivity, MemoryFaults};

fn open(scope: &str, thread: &str, reason: &str, detail: Option<&str>) -> NewFault {
    NewFault {
        scope: scope.to_owned(),
        thread: thread.to_owned(),
        since: "2026-10-25T12:00:00Z".to_owned(),
        reason: reason.to_owned(),
        wake_id: None,
        detail: detail.map(str::to_owned),
    }
}

fn clear(scope: &str, thread: &str, by: &str, note: &str) -> ClearFault {
    ClearFault {
        scope: scope.to_owned(),
        thread: thread.to_owned(),
        at: "2026-10-25T12:05:00Z".to_owned(),
        by: by.to_owned(),
        note: note.to_owned(),
    }
}

type Line = (Level, String, Option<String>, Option<String>);

fn line(level: Level, text: &str, project: &str, thread: &str) -> Line {
    (
        level,
        text.to_owned(),
        Some(project.to_owned()),
        Some(thread.to_owned()),
    )
}

fn setup() -> (ActivityFaults, MemoryActivity) {
    let log = MemoryActivity::new();
    (
        ActivityFaults::new(Box::new(MemoryFaults::new()), Arc::new(log.clone())),
        log,
    )
}

fn written(log: &MemoryActivity) -> Vec<Line> {
    log.lines()
        .into_iter()
        .map(|l| (l.level, l.text, l.project, l.thread))
        .collect()
}

#[test]
fn a_wake_fault_that_opens_and_one_that_the_registry_clears_are_one_line_each() {
    let (faults, log) = setup();
    let detail = "no wake-stream for 600s while the session is active";
    faults
        .open(&open("rotp", "lead", "not-listening", Some(detail)))
        .expect("open");
    let second = faults
        .open(&open("rotp", "lead", "no-listener", None))
        .expect("open");
    assert!(second.is_none(), "one open per thread");
    faults
        .clear(&clear("rotp", "lead", "registry", "it listens again"))
        .expect("clear");
    assert_eq!(
        written(&log),
        [
            line(
                Level::Warn,
                "rotp/lead: its wake listener is not running (no wake-stream for 600s while the session is active)",
                "rotp",
                "lead"
            ),
            line(
                Level::Info,
                "rotp/lead: wake fault (not-listening) cleared by itself: it listens again",
                "rotp",
                "lead"
            ),
        ]
    );
}

#[test]
fn a_wake_fault_that_the_overlord_clears_names_who_and_why() {
    let (faults, log) = setup();
    let detail = "queued for 61s, nobody listening";
    faults
        .open(&open("brock", "ui", "no-listener", Some(detail)))
        .expect("open");
    faults
        .clear(&clear(
            "brock",
            "ui",
            "coordinator",
            "restarted its listener",
        ))
        .expect("clear");
    assert_eq!(
        written(&log),
        [
            line(
                Level::Warn,
                "brock/ui: a wake found nobody listening (queued for 61s, nobody listening)",
                "brock",
                "ui"
            ),
            line(
                Level::Info,
                "brock/ui: wake fault (no-listener) cleared by coordinator: restarted its listener",
                "brock",
                "ui"
            ),
        ]
    );
    assert!(log.lines().iter().all(|l| l.kind == "wake-fault"));
}

#[test]
fn every_reason_has_its_words_and_an_unknown_one_is_named() {
    let fault = |reason: &str, detail: Option<&str>| Fault {
        id: 1,
        project: "p".to_owned(),
        thread: "t".to_owned(),
        since: String::new(),
        reason: reason.to_owned(),
        wake_id: None,
        detail: detail.map(str::to_owned),
        cleared_at: None,
        cleared_by: Some("coordinator".to_owned()),
        note: None,
    };
    assert_eq!(
        fault_line(&fault("no-reaction", None)),
        "p/t: woken, but no sign of life"
    );
    assert_eq!(
        fault_line(&fault("delivery-error", Some("socket closed"))),
        "p/t: a wake could not be written to its listener (socket closed)"
    );
    assert_eq!(
        fault_line(&fault("board-closed", None)),
        "p/t: its board is closed"
    );
    assert_eq!(
        fault_line(&fault("something-new", None)),
        "p/t: wake fault something-new"
    );
    assert_eq!(
        cleared_line(&fault("no-reaction", None)),
        "p/t: wake fault (no-reaction) cleared by coordinator"
    );
}
