//! KEEP: K1.
//! The registry rules for a hosted thread (`host` `argus`): no board-closed and no not-listening fault
//! while its child lives, `hosted-down` when the child is gone, and a desktop thread keeps both faults.

// test-tag: critical

use argus_wake::Clock;
use argus_wake::engine::AttachOptions;
use argus_wake::faults::{Fault, FaultStore};
use argus_wake::reminders::{HOSTED_DOWN, OwnerAlerts};
use argus_wake::testkit::{HOUR, Rig, SEC, START, Tap, presence_config};
use argus_wake::time::iso;

const STEP: i64 = 5 * SEC;

fn rig() -> Rig {
    Rig::registry_seen(&presence_config(), true, HOUR)
}

fn reasons(rig: &Rig) -> Vec<String> {
    rig.faults
        .list(false)
        .expect("faults")
        .into_iter()
        .map(|f| f.reason)
        .collect()
}

fn hosted(rig: &mut Rig) {
    rig.edit_thread("alpha", "ui", |t| t.host = "argus".into());
}

fn sink(rig: &mut Rig, label: &str) -> Tap {
    rig.listen(
        "alpha",
        "ui",
        AttachOptions {
            v: 2,
            label: Some(label.to_owned()),
        },
    )
}

fn seen_now(rig: &mut Rig, working: bool) {
    let now = iso(rig.clock.now_ms());
    rig.edit_thread("alpha", "ui", |t| {
        if working {
            t.activity = "working".into();
        }
        t.last_seen = now.as_str().into();
    });
    rig.service.on_board("alpha");
}

fn fault(reason: &str) -> Fault {
    Fault {
        id: 1,
        project: "portfolio".to_owned(),
        thread: "lead".to_owned(),
        since: iso(START),
        reason: reason.to_owned(),
        wake_id: None,
        detail: None,
        cleared_at: None,
        cleared_by: None,
        note: None,
    }
}

#[test]
fn a_hosted_thread_with_a_live_child_gets_no_board_or_listening_fault() {
    let mut rig = rig();
    hosted(&mut rig);
    let child = sink(&mut rig, "hosted overlord");
    seen_now(&mut rig, true);
    rig.pass(120 * SEC, STEP);
    assert_eq!(reasons(&rig).len(), 0);
    rig.service.detach(&child.attached.id);
}

#[test]
fn a_hosted_thread_whose_child_is_gone_opens_hosted_down_and_clears_when_it_is_back() {
    let mut rig = rig();
    hosted(&mut rig);
    let child = sink(&mut rig, "hosted overlord");
    rig.service.detach(&child.attached.id);
    for _ in 0..10 {
        seen_now(&mut rig, true);
        rig.pass(STEP, STEP);
    }
    assert_eq!(reasons(&rig), [HOSTED_DOWN]);
    let back = sink(&mut rig, "hosted overlord");
    rig.pass(STEP, STEP);
    assert_eq!(reasons(&rig).len(), 0);
    rig.service.detach(&back.attached.id);
}

#[test]
fn a_desktop_thread_still_gets_board_closed_and_not_listening() {
    let mut closed = rig();
    let stream = sink(&mut closed, "ui");
    seen_now(&mut closed, true);
    closed.pass(40 * SEC, STEP);
    assert_eq!(reasons(&closed), ["board-closed"]);
    closed.service.detach(&stream.attached.id);
    let mut deaf = rig();
    let _board = deaf.service.presence_attach("alpha", "ui", "t1");
    for _ in 0..14 {
        seen_now(&mut deaf, false);
        deaf.pass(STEP, STEP);
    }
    assert_eq!(reasons(&deaf), ["not-listening"]);
}

#[test]
fn a_hosted_down_fault_reaches_the_owner_at_once_and_another_waits_the_hour() {
    let mut alerts = OwnerAlerts::new(HOUR);
    let notices = alerts.due(START, &[fault(HOSTED_DOWN), fault("board-closed")]);
    assert_eq!(notices.len(), 1);
    assert_eq!(
        notices[0].text,
        "portfolio/lead: the hosted Overlord is down"
    );
}
