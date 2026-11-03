//! KEEP: K5 (D-11), K1.
//! Rule 2 of the stall watch, the acknowledgement ladder: a wake that asks a thread to act and that nobody
//! read is sent again at 5 minutes, becomes a `stall` line for the Overlord at 10 and reaches the owner at 30;
//! reading the inbox, or the thing it asked for, ends it. A manual clock drives every tick.

// test-tag: critical

use argus_wake::config::WakeConfig;
use argus_wake::reasons::{IMMEDIATE_REASONS, WAKE_REASONS};
use argus_wake::stalls::ACKABLE;
use argus_wake::testkit::{DAY, HOUR, MIN};
use argus_wake::wake::WakeInput;
use argus_wake::wake_policy::{WakeClass, class_of};
use serde_json::json;

use crate::stalls_world::{
    REQUEST, activity_texts, assert_to, assert_unread_assignment, assigned, dep_request, item,
    open_dep, pass, pass_quiet, pass_with, touch, working, world,
};

#[test]
fn d11_unread_dep_request_nudges_at_5_wakes_the_overlord_at_10_and_the_owner_at_30() {
    let mut rig = world(&json!({}), &[item("I-112", "todo", &json!({}))], &json!({}));
    open_dep(&rig, &json!({}));
    dep_request(&mut rig);
    pass_with(&mut rig, 4, working);
    assert_eq!(rig.wakes_of("ack-nudge").len(), 0, "not before 5 minutes");
    pass_with(&mut rig, 1, working);
    let nudges = rig.wakes_of("ack-nudge");
    assert_eq!(nudges.len(), 1);
    assert_to(&nudges[0], "rotp", "lead");
    assert_eq!(nudges[0].text, format!("2nd: {REQUEST}"));
    assert!(
        rig.wakes_of("stall").is_empty(),
        "the Overlord waits for 10"
    );
    pass_with(&mut rig, 4, working);
    assert_eq!(rig.wakes_of("stall").len(), 0);
    pass_with(&mut rig, 1, working);
    let stalls = rig.wakes_of("stall");
    assert_eq!(stalls.len(), 1);
    assert_to(&stalls[0], "portfolio", "coordinator");
    assert_eq!(
        stalls[0].text,
        "stall: rotp did not read D-11 for 10 min (it is in a turn)"
    );
    assert_eq!(activity_texts(&rig, "stall"), [stalls[0].text.clone()]);
    assert_eq!(rig.owner.notices().len(), 0, "the owner waits for 30");
    pass_with(&mut rig, 20, working);
    let notices = rig.owner.notices();
    assert_eq!(notices.len(), 1);
    assert_eq!(notices[0].title, "Unread for 30 min");
    assert_eq!(
        notices[0].text,
        "stall: rotp did not read D-11 for 30 min (it is in a turn)"
    );
    assert_eq!(
        (notices[0].project.as_str(), notices[0].thread.as_str()),
        ("rotp", "lead")
    );
    assert_eq!(notices[0].topic, "stall");
    assert_eq!(rig.wakes_of("ack-nudge").len(), 1, "the nudge goes once");
    assert_eq!(
        rig.wakes_of("stall").len(),
        1,
        "the resend is 30 minutes on"
    );
}

#[test]
fn an_inbox_read_before_5_min_stops_the_ladder() {
    let mut rig = world(&json!({}), &[item("I-112", "todo", &json!({}))], &json!({}));
    open_dep(&rig, &json!({}));
    rig.busy.set("rotp", true);
    dep_request(&mut rig);
    pass(&mut rig, 3);
    rig.service.inbox_read("rotp", "lead");
    pass_quiet(&mut rig, 26);
    assert_eq!(rig.service.stalls(None).len(), 0);
}

#[test]
fn starting_the_item_clears_the_stall_with_a_log_line() {
    let mut rig = world(&json!({}), &[item("I-112", "todo", &json!({}))], &json!({}));
    open_dep(&rig, &json!({}));
    dep_request(&mut rig);
    pass_with(&mut rig, 10, working);
    assert_eq!(rig.service.stalls(Some("rotp")).len(), 1);
    touch(&rig, "I-112", "doing");
    pass_with(&mut rig, 1, working);
    assert_eq!(rig.service.stalls(None).len(), 0);
    assert_eq!(
        activity_texts(&rig, "stall-cleared"),
        ["stall cleared: rotp did not read D-11 for 10 min (it is in a turn) (open 11m)"]
    );
}

#[test]
fn an_unread_item_is_named_by_its_reason_and_the_line_says_when_the_lead_is_idle() {
    let mut rig = world(&json!({}), &[item("I-5", "todo", &json!({}))], &json!({}));
    rig.busy.set("rotp", true);
    assigned(&mut rig, "I-5");
    pass(&mut rig, 10);
    assert_unread_assignment(&rig, "I-5");
}

#[test]
fn the_overlord_hears_the_nudge_and_the_owner_the_toast_but_never_a_stall_about_the_overlord() {
    let mut rig = world(&json!({}), &[], &json!({}));
    let wake = WakeInput::about(
        "portfolio",
        "coordinator",
        "owner-note",
        "E-4",
        "owner note: look at the bundle",
    );
    rig.service.submit(&[wake]).expect("submit");
    pass(&mut rig, 30);
    let nudges = rig.wakes_of("ack-nudge");
    assert_eq!(nudges.len(), 1);
    assert_to(&nudges[0], "portfolio", "coordinator");
    assert_eq!(rig.wakes_of("stall").len(), 0);
    let notices = rig.owner.notices();
    assert_eq!(notices.len(), 1);
    assert_eq!(
        notices[0].text,
        "stall: portfolio did not read owner-note E-4 for 30 min (it is idle)"
    );
}

#[test]
fn notices_and_fault_lines_are_never_ackable() {
    let mut rig = world(&json!({}), &[], &json!({}));
    let quiet = [
        "kanban-stale",
        "wake-fault",
        "fault-open",
        "open-resend",
        "thread-message",
        "agent-idle",
        "test-fail",
        "resume",
        "stall",
        "ack-nudge",
    ];
    for reason in quiet {
        assert!(!ACKABLE.contains(&reason), "{reason}");
    }
    let wakes: Vec<WakeInput> = quiet[..8]
        .iter()
        .map(|reason| WakeInput::about("rotp", "lead", reason, "X-1", "a line"))
        .collect();
    rig.service.submit(&wakes).expect("submit");
    pass_quiet(&mut rig, 40);
}

#[test]
fn the_new_reasons_are_wake_class_and_close_the_window_at_once() {
    for reason in ["ack-nudge", "stall"] {
        assert!(WAKE_REASONS.contains(&reason), "{reason}");
        assert!(IMMEDIATE_REASONS.contains(&reason), "{reason}");
        assert_eq!(class_of(reason), WakeClass::Wake, "{reason}");
    }
}

#[test]
fn the_stall_settings_default_and_come_from_config_and_an_invalid_value_keeps_its_default() {
    let d = WakeConfig::default().stall;
    assert!(d.enabled);
    assert_eq!(
        [
            d.tick_ms,
            d.nudge_ms,
            d.unread_ms,
            d.owner_ms,
            d.unstarted_ms
        ],
        [MIN, 5 * MIN, 10 * MIN, 30 * MIN, 30 * MIN]
    );
    assert_eq!(
        [
            d.idle_ms,
            d.silent_ms,
            d.resend_ms,
            d.horizon_ms,
            d.proc_idle_min
        ],
        [10 * MIN, HOUR, 30 * MIN, DAY, 10]
    );
    let config = json!({"stall": {
        "enabled": false, "nudgeMs": 120_000, "ownerMs": "60000", "idleMs": -5, "silentMs": "soon",
    }});
    let c = WakeConfig::from_json(&config).stall;
    assert!(!c.enabled);
    assert_eq!((c.nudge_ms, c.owner_ms), (2 * MIN, MIN));
    assert_eq!((c.idle_ms, c.silent_ms), (10 * MIN, HOUR));
}
