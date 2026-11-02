//! KEEP: K1.
//! Rule 3 of the stall watch: a dependency that was read but stays `open` for 30 minutes while its item is still
//! `todo` wakes the Overlord, and again every 30 minutes, until the item is started. A manual clock drives every
//! tick.

// test-tag: critical

use argus_wake::testkit::{MIN, Rig, START};
use argus_wake::time::iso;
use serde_json::json;

use crate::stalls_world::{activity_texts, item, open_dep, pass, touch, world};

fn acked_world() -> Rig {
    let rig = world(&json!({}), &[item("I-112", "todo", &json!({}))], &json!({}));
    open_dep(&rig, &json!({"ackedAt": iso(START + MIN)}));
    rig.busy.set("rotp", true);
    rig
}

#[test]
fn an_acked_dep_open_30_min_with_its_item_todo_wakes_the_overlord_then_every_30_min() {
    let mut rig = acked_world();
    pass(&mut rig, 29);
    assert!(rig.wakes_of("stall").is_empty(), "not before 30 minutes");
    pass(&mut rig, 1);
    let first = rig.wakes_of("stall");
    assert_eq!(first.len(), 1);
    assert_eq!(
        (first[0].scope.as_str(), first[0].thread.as_str()),
        ("portfolio", "coordinator")
    );
    assert_eq!(
        first[0].text,
        "stall: D-11 open 30 min, rotp I-112 still todo"
    );
    assert_eq!(
        first[0].subject.as_deref(),
        Some(format!("rotp:unstarted:D-11:{START}").as_str())
    );
    pass(&mut rig, 29);
    assert_eq!(
        rig.wakes_of("stall").len(),
        1,
        "not again before 30 minutes"
    );
    pass(&mut rig, 1);
    let all = rig.wakes_of("stall");
    assert_eq!(all.len(), 2);
    assert_eq!(
        all[1].text,
        "stall: D-11 open 60 min, rotp I-112 still todo"
    );
    assert_eq!(activity_texts(&rig, "stall").len(), 1, "raised once");
}

#[test]
fn the_item_leaving_todo_clears_it() {
    let mut rig = acked_world();
    pass(&mut rig, 30);
    assert_eq!(rig.service.stalls(None).len(), 1);
    touch(&rig, "I-112", "doing");
    pass(&mut rig, 1);
    assert!(rig.service.stalls(None).is_empty());
    assert_eq!(
        activity_texts(&rig, "stall-cleared"),
        ["stall cleared: D-11 open 30 min, rotp I-112 still todo (open 31m)"]
    );
    pass(&mut rig, 29);
    assert_eq!(
        rig.wakes_of("stall").len(),
        1,
        "nothing more once it cleared"
    );
}
