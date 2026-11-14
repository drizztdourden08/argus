//! The hosted Overlord's listener: K2 the wake text is the same on the hook path and the hosted
//! path, S2 a gone child leaves the wake pending and the next attach receives it.

// test-tag: critical

use std::sync::{Arc, Mutex};

use argus_wake::engine::{AttachOptions, Delivery, ListenerSink};
use argus_wake::hosted::HostedSink;
use argus_wake::line::wake_line;
use argus_wake::store::WakeStore;
use argus_wake::testkit::{EngineRig, wake};
use argus_wake::wake::{WakeEvent, WakeStatus};

fn event(lines: &[&str], count: u32) -> WakeEvent {
    WakeEvent {
        id: "W-1".to_owned(),
        scope: "portfolio".to_owned(),
        thread: "coordinator".to_owned(),
        at: "2026-11-14T10:00:00.000Z".to_owned(),
        lines: lines.iter().map(|l| (*l).to_owned()).collect(),
        count,
    }
}

type Taken = Arc<Mutex<Vec<String>>>;

fn taking(taken: &Taken) -> HostedSink {
    let taken = Arc::clone(taken);
    HostedSink::new(Box::new(move |text| {
        taken.lock().unwrap().push(text);
        true
    }))
}

fn gone() -> HostedSink {
    HostedSink::new(Box::new(|_| false))
}

#[test]
fn the_hosted_path_says_what_the_hook_listener_prints() {
    let taken = Taken::default();
    let mut sink = taking(&taken);
    for wake_event in [
        event(&["lead: look at the build"], 1),
        event(&["a", "b"], 3),
        event(&[], 1),
    ] {
        assert_eq!(sink.deliver(&wake_event), Delivery::Sent);
        let data = serde_json::to_string(&serde_json::to_value(&wake_event).unwrap()).unwrap();
        let hook_text = wake_line(&data, "coordinator");
        assert_eq!(taken.lock().unwrap().last(), Some(&hook_text));
    }
    assert_eq!(
        taken.lock().unwrap()[..2],
        [
            "WAKE coordinator: lead: look at the build. Run: coord inbox",
            "WAKE coordinator: a; b (3 wakes). Run: coord inbox",
        ]
    );
}

#[test]
fn a_sink_whose_child_is_gone_says_gone() {
    assert_eq!(gone().deliver(&event(&["x"], 1)), Delivery::Gone);
}

#[test]
fn a_wake_that_found_no_child_stays_pending_and_the_next_attach_gets_it() {
    let mut rig = EngineRig::new(15_000);
    rig.submit(&[wake("coordinator", "T-1 failed: a")]);
    rig.advance(15_000);
    let dead = rig
        .engine
        .attach("alpha", "coordinator", Box::new(gone()), AttachOptions::default());
    assert_eq!(dead.replayed, 0);
    assert_eq!(
        rig.store.get("W-1").unwrap().unwrap().status,
        WakeStatus::Queued
    );
    rig.engine.detach(&dead.id);
    let taken = Taken::default();
    let live = rig.engine.attach(
        "alpha",
        "coordinator",
        Box::new(taking(&taken)),
        AttachOptions::default(),
    );
    assert_eq!(live.replayed, 1);
    assert_eq!(
        taken.lock().unwrap()[..],
        ["WAKE coordinator: T-1 failed: a. Run: coord inbox"]
    );
    assert_eq!(
        rig.store.get("W-1").unwrap().unwrap().status,
        WakeStatus::Delivered
    );
}
