//! KEEP: K1.
//! What stays open is not left to one passive notice: an open wake fault wakes the Overlord at once
//! (`fault-open`), the Overlord hears ONE coalesced line every 15 minutes while a fault or an open notice
//! (`worktree-stale`, `remote-sync-due`) stays open (`open-resend`), the owner hears of a fault open for an
//! hour, a fault clears by itself when its cause is gone and says why in the activity log, and a board that
//! is closed is a notice only, raised for a working thread and never for an idle one. A manual clock drives
//! every tick.

// test-tag: critical

use argus_wake::Clock;
use argus_wake::config::WakeConfig;
use argus_wake::engine::AttachOptions;
use argus_wake::faults::FaultStore;
use argus_wake::reasons::{IMMEDIATE_REASONS, WAKE_REASONS};
use argus_wake::reminders::{OpenNotice, ReminderConfig};
use argus_wake::testkit::{HOUR, MIN, Rig, SEC, Tap, presence_config, registry_config, wake_in};
use argus_wake::time::iso;
use argus_wake::wake::WakeStatus;
use argus_wake::wake_policy::{WakeClass, class_of};
use serde_json::json;

const STEP: i64 = 5 * SEC;

fn send(rig: &mut Rig, thread: &str) {
    rig.service
        .submit(&[wake_in("alpha", thread, "T-1 failed: x", "test-fail")])
        .expect("submit");
}

/// Time passes in steps; the registry and the reminders tick after each, the way the real timers do.
fn remind(rig: &mut Rig, ms: i64) {
    let mut left = ms;
    while left > 0 {
        rig.clock.advance(STEP);
        rig.service.registry_tick();
        rig.service.reminder_tick();
        left -= STEP;
    }
}

/// An Overlord that listens, and a wake for each of these threads that nobody hears: one fault each.
fn faulted(threads: &[&str]) -> (Rig, Tap) {
    let mut rig = Rig::registry(&registry_config(), false);
    let main = rig.listen("portfolio", "coordinator", AttachOptions::default());
    for thread in threads {
        send(&mut rig, thread);
    }
    rig.pass(70 * SEC, STEP);
    assert_eq!(rig.faults.list(false).expect("faults").len(), threads.len());
    (rig, main)
}

fn working(rig: &mut Rig, thread: &str) {
    let now = iso(rig.clock.now_ms());
    rig.edit_thread("alpha", thread, |t| {
        t.activity = "working".into();
        t.last_seen = now.as_str().into();
    });
    rig.service.on_board("alpha");
}

/// A rig with presence on and a `ui` thread that listens; keep the tap, a dropped one detaches.
fn presence_rig() -> (Rig, Tap) {
    let mut rig = Rig::registry_seen(&presence_config(), true, HOUR);
    let ui = rig.listen(
        "alpha",
        "ui",
        AttachOptions {
            v: 2,
            label: Some("ui".to_owned()),
        },
    );
    (rig, ui)
}

#[test]
fn the_new_reasons_are_wake_class_and_only_the_first_closes_the_window_at_once() {
    for reason in ["fault-open", "open-resend"] {
        assert!(WAKE_REASONS.contains(&reason), "{reason}");
        assert_eq!(class_of(reason), WakeClass::Wake, "{reason}");
    }
    assert!(IMMEDIATE_REASONS.contains(&"fault-open"));
    assert!(!IMMEDIATE_REASONS.contains(&"open-resend"));
}

#[test]
fn the_reminder_settings_default_to_15_minutes_and_an_hour_and_come_from_config() {
    let defaults = WakeConfig::default().reminders;
    assert_eq!(defaults.resend_ms, 15 * MIN);
    assert_eq!(defaults.owner_ms, HOUR);
    let config =
        WakeConfig::from_json(&json!({"reminders": {"resendMs": 60_000, "ownerMs": "120000"}}));
    assert_eq!(
        config.reminders,
        ReminderConfig {
            resend_ms: MIN,
            owner_ms: 2 * MIN
        }
    );
}

#[test]
fn an_open_fault_wakes_the_overlord_once_at_once_and_a_quiet_ten_minutes_wakes_nobody_again() {
    let (mut rig, mut main) = faulted(&["ui"]);
    assert_eq!(main.count(), 1, "one model turn when the fault opened");
    let line = &main.lines()[0][0];
    assert!(
        line.contains("alpha/ui: no-listener (queued for 65s"),
        "{line}"
    );
    let rows = rig.wakes_of("fault-open");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, WakeStatus::Delivered);
    assert_eq!(rows[0].thread, "coordinator");
    remind(&mut rig, 10 * MIN);
    assert_eq!(rig.wakes_of("fault-open").len(), 1);
    assert_eq!(rig.wakes_of("open-resend").len(), 0);
    assert_eq!(main.count(), 1);
}

#[test]
fn two_open_faults_are_sent_again_every_15_minutes_as_one_coalesced_line() {
    let (mut rig, _main) = faulted(&["ui", "api"]);
    assert_eq!(rig.wakes_of("fault-open").len(), 2);
    remind(&mut rig, 14 * MIN);
    assert_eq!(
        rig.wakes_of("open-resend").len(),
        0,
        "not before 15 minutes"
    );
    remind(&mut rig, 2 * MIN);
    let first = rig.wakes_of("open-resend");
    assert_eq!(first.len(), 1, "one line, not one per fault");
    assert_eq!(first[0].text, "2 faults open, oldest 15m");
    assert_eq!(
        (first[0].scope.as_str(), first[0].thread.as_str()),
        ("portfolio", "coordinator")
    );
    remind(&mut rig, 15 * MIN);
    let all = rig.wakes_of("open-resend");
    assert_eq!(all.len(), 2, "and again 15 minutes later");
    assert_eq!(all[1].text, "2 faults open, oldest 30m");
}

#[test]
fn the_owner_is_told_once_when_a_fault_has_been_open_an_hour() {
    let (mut rig, _main) = faulted(&["ui"]);
    remind(&mut rig, 59 * MIN);
    assert_eq!(rig.owner.notices().len(), 0, "not before the hour");
    remind(&mut rig, 2 * MIN);
    let notices = rig.owner.notices();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].title, "Wake fault open for 1h");
    assert!(
        notices[0].text.starts_with("alpha/ui: "),
        "{}",
        notices[0].text
    );
    assert_eq!(
        (notices[0].project.as_str(), notices[0].thread.as_str()),
        ("alpha", "ui")
    );
    remind(&mut rig, 30 * MIN);
    assert_eq!(rig.owner.notices().len(), 1, "once, not every tick");
}

#[test]
fn a_fault_clears_by_itself_when_the_lead_shows_signs_of_life_and_the_log_says_why() {
    let mut rig = Rig::registry(&registry_config(), false);
    let _lead = rig.listen("alpha", "*", AttachOptions::default());
    let _main = rig.listen("portfolio", "coordinator", AttachOptions::default());
    send(&mut rig, "lead");
    rig.pass(125 * SEC, STEP);
    let open = rig.faults.list(false).expect("faults");
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].reason, "no-reaction");
    rig.seen("alpha", "lead");
    assert_eq!(rig.faults.list(false).expect("faults").len(), 0);
    let done = rig.faults.list(true).expect("faults");
    assert_eq!(done[0].cleared_by.as_deref(), Some("registry"));
    assert_eq!(
        done[0].note.as_deref(),
        Some("the thread shows signs of life after the wake")
    );
    let logged = rig
        .activity
        .lines()
        .iter()
        .any(|l| {
            l.text
                == "alpha/lead: wake fault (no-reaction) cleared by itself: the thread shows signs of life after the wake"
        });
    assert!(logged, "{:?}", rig.activity.lines());
    remind(&mut rig, 20 * MIN);
    assert_eq!(
        rig.wakes_of("open-resend").len(),
        0,
        "nothing is open any more"
    );
}

#[test]
fn an_idle_thread_with_its_board_closed_is_never_faulted() {
    let (mut rig, _ui) = presence_rig();
    remind(&mut rig, 10 * MIN);
    assert_eq!(rig.faults.list(true).expect("faults").len(), 0);
    assert_eq!(rig.wakes_of("wake-fault").len(), 0);
    assert_eq!(rig.wakes_of("fault-open").len(), 0);
}

#[test]
fn a_board_closed_fault_of_a_working_thread_is_a_notice_and_the_reminder_wakes_if_it_stays_open() {
    let (mut rig, _ui) = presence_rig();
    working(&mut rig, "ui");
    rig.pass(40 * SEC, STEP);
    let open = rig.faults.list(false).expect("faults");
    assert_eq!(open.len(), 1);
    assert_eq!(open[0].reason, "board-closed");
    let notices = rig.fault_wakes();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].status, WakeStatus::Passive);
    assert_eq!(rig.wakes_of("fault-open").len(), 0, "the notice only");
    for _ in 0..16 {
        working(&mut rig, "ui");
        remind(&mut rig, MIN);
    }
    let lines = rig.wakes_of("open-resend");
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].text.starts_with("1 fault open, oldest "),
        "{}",
        lines[0].text
    );
    rig.edit_thread("alpha", "ui", |t| t.activity = "idle".into());
    rig.service.on_board("alpha");
    assert_eq!(rig.faults.list(false).expect("faults").len(), 0);
    let done = rig.faults.list(true).expect("faults");
    assert_eq!(
        done[0].note.as_deref(),
        Some("the thread is idle, its board is not required")
    );
}

fn stale_worktree_rig() -> Rig {
    let rig = Rig::registry(&registry_config(), false);
    rig.notices.set(vec![OpenNotice {
        reason: "worktree-stale".to_owned(),
        project: "alpha".to_owned(),
        subject: "agent-wt".to_owned(),
    }]);
    rig
}

#[test]
fn open_notices_join_the_same_line_every_15_minutes_until_they_are_resolved() {
    let mut rig = stale_worktree_rig();
    remind(&mut rig, 16 * MIN);
    let first = rig.wakes_of("open-resend");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].text, "1 worktree-stale open");
    rig.notices.set(Vec::new());
    remind(&mut rig, 30 * MIN);
    assert_eq!(
        rig.wakes_of("open-resend").len(),
        1,
        "resolved: no more lines"
    );
}

#[test]
fn a_remote_sync_that_stays_due_is_sent_again_with_the_other_open_notices() {
    let mut rig = stale_worktree_rig();
    rig.clock.advance(25 * HOUR);
    rig.service.reminder_tick();
    assert_eq!(
        rig.wakes_of("open-resend").len(),
        0,
        "the clock starts here"
    );
    rig.clock.advance(15 * MIN);
    rig.service.reminder_tick();
    let lines = rig.wakes_of("open-resend");
    assert_eq!(lines.len(), 1);
    assert_eq!(
        lines[0].text,
        "1 remote-sync-due open; 1 worktree-stale open"
    );
    rig.syncs
        .set("alpha", &iso(rig.clock.now_ms()), "alpha", "pushed");
    rig.clock.advance(15 * MIN);
    rig.service.reminder_tick();
    let after = rig.wakes_of("open-resend");
    assert_eq!(after.len(), 2);
    assert_eq!(
        after[1].text, "1 worktree-stale open",
        "a sync record ends its notice"
    );
}
