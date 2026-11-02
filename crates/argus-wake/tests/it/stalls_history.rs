//! KEEP: K5 (D-11).
//! Rule 2 starts with the service: a wake made before it has no read stamp to compare with, so it is history and
//! never unread, while a wake made after it is watched as before. A manual clock drives every tick.

// test-tag: critical

use argus_wake::testkit::{MIN, Rig, START};
use argus_wake::wake::WakeInput;
use serde_json::json;

use crate::stalls_world::{item, pass, world};

fn assigned(rig: &mut Rig, id: &str) {
    let wake = WakeInput::about(
        "rotp",
        "lead",
        "item-assigned",
        id,
        format!("{id} ({id} title) was assigned to you by the owner"),
    );
    rig.service.submit(&[wake]).expect("submit");
}

fn busy_world() -> Rig {
    let items = [
        item("I-5", "todo", &json!({})),
        item("I-6", "todo", &json!({})),
    ];
    let rig = world(&json!({}), &items, &json!({}));
    rig.busy.set("rotp", true);
    rig
}

#[test]
fn an_ackable_wake_older_than_the_service_never_raises_an_unread_stall() {
    let mut rig = busy_world();
    rig.clock.set(START - 20 * MIN);
    assigned(&mut rig, "I-5");
    rig.clock.set(START);
    pass(&mut rig, 40);
    assert_eq!(rig.wakes_of("ack-nudge").len(), 0);
    assert_eq!(rig.wakes_of("stall").len(), 0);
    assert_eq!(rig.owner.notices().len(), 0);
}

#[test]
fn an_ackable_wake_made_after_the_service_started_still_raises_one() {
    let mut rig = busy_world();
    rig.clock.set(START - 20 * MIN);
    assigned(&mut rig, "I-5");
    rig.clock.set(START);
    pass(&mut rig, 5);
    assigned(&mut rig, "I-6");
    pass(&mut rig, 10);
    let stalls = rig.wakes_of("stall");
    assert_eq!(stalls.len(), 1);
    assert_eq!(
        stalls[0].text,
        "stall: rotp did not read item-assigned I-6 for 10 min (it is idle)"
    );
}
