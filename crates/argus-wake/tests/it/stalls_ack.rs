//! KEEP: K1.
//! The Overlord's ack of a stall (`coord stall ack`): an acked stall stops its 30 minute resend and leaves the 15
//! minute `open-resend` count but stays on the page until it clears, and a new occurrence after a clear is a new
//! stall that an old ack does not cover. A manual clock drives every tick.

// test-tag: critical

use serde_json::Value;

use crate::stalls_world::{activity_texts, pass, pass_reminding, silent_world, touch};

const BY: &str = "portfolio/coordinator";

#[test]
fn an_acked_stall_stops_its_resends_and_leaves_the_open_resend_line() {
    let mut rig = silent_world();
    pass_reminding(&mut rig, 74);
    assert_eq!(rig.wakes_of("stall").len(), 1);
    assert_eq!(rig.wakes_of("open-resend").len(), 0);
    pass_reminding(&mut rig, 1);
    let lines = rig.wakes_of("open-resend");
    assert_eq!(lines.len(), 1, "15 minutes after the stall was raised");
    assert_eq!(lines[0].text, "1 stall open");
    assert_eq!(
        (lines[0].scope.as_str(), lines[0].thread.as_str()),
        ("portfolio", "coordinator")
    );
    let acked = rig
        .service
        .ack_stalls("rotp", BY, "I asked the lead about I-12");
    assert_eq!(acked.len(), 1);
    assert_eq!(acked[0].acked_by.as_deref(), Some(BY));
    assert_eq!(
        acked[0].ack_note.as_deref(),
        Some("I asked the lead about I-12")
    );
    let doc = rig
        .docs
        .get("_portfolio", "stalls", &acked[0].id)
        .expect("doc");
    assert_eq!(doc["ackedBy"], Value::from(BY));
    assert_eq!(rig.service.ack_stalls("rotp", BY, "again").len(), 0);
    pass_reminding(&mut rig, 60);
    assert_eq!(rig.wakes_of("stall").len(), 1, "no resend once acked");
    assert_eq!(rig.wakes_of("open-resend").len(), 1, "and no count");
    assert_eq!(
        rig.service.stalls(Some("rotp")).len(),
        1,
        "still on the page"
    );
}

#[test]
fn a_new_occurrence_after_a_clear_wakes_again() {
    let mut rig = silent_world();
    assert_eq!(
        rig.service.ack_stalls("rotp", BY, "nothing is open").len(),
        0
    );
    pass(&mut rig, 60);
    let first = rig.service.stalls(None);
    assert_eq!(first.len(), 1);
    rig.service.ack_stalls("rotp", BY, "I looked at I-12");
    pass(&mut rig, 1);
    touch(&rig, "I-12", "doing");
    pass(&mut rig, 1);
    assert_eq!(rig.service.stalls(None).len(), 0);
    assert_eq!(activity_texts(&rig, "stall-cleared").len(), 1);
    pass(&mut rig, 59);
    let rows = rig.wakes_of("stall");
    assert_eq!(rows.len(), 2, "the old ack does not cover the new stall");
    assert_ne!(rows[0].subject, rows[1].subject);
    let again = rig.service.stalls(None);
    assert_eq!(again.len(), 1);
    assert_ne!(again[0].id, first[0].id);
    assert!(again[0].acked_at.is_none());
}
