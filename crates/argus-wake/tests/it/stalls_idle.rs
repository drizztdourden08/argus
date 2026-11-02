//! KEEP: K1, and the acceptance line: an idle lead with an unblocked `todo` item gives a stall line within 11
//! minutes, and none once a blocker is recorded.
//! Rule 4 of the stall watch: a live lead with no turn, no running agent and no running process for 10 minutes,
//! while an item is `todo` or `doing` and nothing blocks it. A manual clock drives every tick.

// test-tag: critical

use serde_json::{Value, json};

use crate::stalls_world::{activity_texts, item, pass, pass_with, working, world};

const LINE: &str = "stall: rotp idle 10 min, 1 open items without a blocker (I-4)";

fn ask(kind: &str, status: &str) -> Value {
    json!({"E-1": {"id": "E-1", "item": "I-4", "kind": kind, "status": status}})
}

#[test]
fn an_idle_lead_with_an_unblocked_todo_item_stalls_within_11_min() {
    let mut rig = world(&json!({}), &[item("I-4", "todo", &json!({}))], &json!({}));
    pass(&mut rig, 9);
    assert!(rig.wakes_of("stall").is_empty(), "not before 10 minutes");
    pass(&mut rig, 1);
    let stalls = rig.wakes_of("stall");
    assert_eq!(stalls.len(), 1);
    assert_eq!(stalls[0].text, LINE);
    assert_eq!(
        (stalls[0].scope.as_str(), stalls[0].thread.as_str()),
        ("portfolio", "coordinator")
    );
    let open = rig.service.stalls(Some("rotp"));
    assert_eq!(
        (
            open[0].kind.as_str(),
            open[0].lead.as_str(),
            open[0].subject.as_str()
        ),
        ("idle", "lead", "lead")
    );
}

#[test]
fn a_blocker_a_dep_wait_or_an_open_ask_keeps_it_quiet() {
    let cases = [
        (json!({"blocker": "needs the schema"}), json!({})),
        (json!({"waitsOn": {"state": "open"}}), json!({})),
        (json!({"requestedBy": {"state": "blocked"}}), json!({})),
        (json!({}), ask("question", "open")),
        (json!({}), ask("decision", "pending")),
    ];
    for (extra, entries) in cases {
        let mut rig = world(
            &json!({}),
            &[item("I-4", "todo", &extra)],
            &json!({"entries": entries}),
        );
        pass(&mut rig, 15);
        assert!(rig.wakes_of("stall").is_empty(), "{extra} {entries}");
        assert!(rig.service.stalls(None).is_empty(), "{extra} {entries}");
    }
}

#[test]
fn a_turn_an_agent_or_a_busy_process_is_not_idle() {
    let agent = json!({"agents": {
        "A-1": {"id": "A-1", "thread": "lead", "status": "working", "item": "I-4"},
    }});
    let none = json!({});
    for case in ["turn", "agent", "process"] {
        let extra = if case == "agent" { &agent } else { &none };
        let mut rig = world(&json!({}), &[item("I-4", "todo", &json!({}))], extra);
        rig.busy.set("rotp", case == "process");
        pass_with(&mut rig, 15, |rig| {
            if case == "turn" {
                working(rig);
            }
        });
        assert!(rig.wakes_of("stall").is_empty(), "{case}");
    }
}

#[test]
fn the_lead_turning_active_clears_it_with_a_log_line() {
    let mut rig = world(&json!({}), &[item("I-4", "todo", &json!({}))], &json!({}));
    pass(&mut rig, 10);
    assert_eq!(rig.service.stalls(None).len(), 1);
    pass_with(&mut rig, 1, working);
    assert!(rig.service.stalls(None).is_empty());
    assert_eq!(
        activity_texts(&rig, "stall-cleared"),
        [format!(
            "stall cleared: {} (open 11m)",
            &LINE["stall: ".len()..]
        )]
    );
}
