//! KEEP: K1.
//! Rule 5 of the stall watch: an item `doing` for 60 minutes with no board write, no agent on it and no blocker
//! is a stall for the Overlord. A manual clock drives every tick.

// test-tag: critical

use serde_json::json;

use crate::stalls_world::{assert_to, item, pass, silent_world, world};

#[test]
fn a_doing_item_60_min_without_a_write_or_agent_stalls() {
    let mut rig = silent_world();
    pass(&mut rig, 59);
    assert_eq!(rig.wakes_of("stall").len(), 0, "not before 60 minutes");
    pass(&mut rig, 1);
    let stalls = rig.wakes_of("stall");
    assert_eq!(stalls.len(), 1);
    assert_to(&stalls[0], "portfolio", "coordinator");
    assert_eq!(stalls[0].text, "stall: rotp/K-12 doing 60 min, no agent");
    let open = rig.service.stalls(Some("rotp"));
    assert_eq!(
        (open[0].kind.as_str(), open[0].subject.as_str()),
        ("silent", "I-12")
    );
}

#[test]
fn an_agent_on_it_or_a_blocker_keeps_it_quiet() {
    let agent = json!({"agents": {
        "A-1": {"id": "A-1", "thread": "lead", "status": "working", "item": "I-12"},
    }});
    let cases = [
        (json!({}), agent),
        (json!({"blocker": "needs the schema"}), json!({})),
    ];
    for (extra, project) in cases {
        let mut rig = world(&json!({}), &[item("I-12", "doing", &extra)], &project);
        rig.busy.set("rotp", true);
        pass(&mut rig, 65);
        assert_eq!(rig.wakes_of("stall").len(), 0, "{extra} {project}");
        assert_eq!(rig.service.stalls(None).len(), 0, "{extra} {project}");
    }
}
