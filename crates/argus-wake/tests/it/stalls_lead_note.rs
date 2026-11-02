//! I-65: a `coord lead` note (`lead-info`, `lead-message`) nobody read is sent again at 3 minutes and
//! reaches the Overlord at 5, sooner than the other wakes of rule 2; reading the inbox ends it.

// test-tag: critical

use argus_wake::stalls::{ACKABLE, LEAD_NOTES};
use argus_wake::testkit::{MIN, Rig};
use argus_wake::wake::WakeInput;
use serde_json::json;

use crate::stalls_world::{assert_to, pass, pass_quiet, world};

const NOTE: &str = "archipelia lead: run 11 result, can you rejoin?";

fn lead_note(rig: &mut Rig, reason: &str) {
    let wake = WakeInput::about("rotp", "lead", reason, "M-1", NOTE);
    rig.service.submit(&[wake]).expect("submit");
}

#[test]
fn an_unread_lead_note_nudges_at_3_and_wakes_the_overlord_at_5() {
    for reason in LEAD_NOTES {
        assert!(ACKABLE.contains(&reason), "{reason}");
        let mut rig = world(&json!({}), &[], &json!({}));
        lead_note(&mut rig, reason);
        pass_quiet(&mut rig, 2);
        pass(&mut rig, 1);
        let nudges = rig.wakes_of("ack-nudge");
        assert_eq!(nudges.len(), 1, "{reason}");
        assert_to(&nudges[0], "rotp", "lead");
        assert_eq!(nudges[0].text, format!("2nd: {NOTE}"));
        pass(&mut rig, 1);
        assert_eq!(rig.wakes_of("stall").len(), 0, "the Overlord waits for 5");
        pass(&mut rig, 1);
        let stalls = rig.wakes_of("stall");
        assert_eq!(stalls.len(), 1, "{reason}");
        assert_to(&stalls[0], "portfolio", "coordinator");
        assert_eq!(
            stalls[0].text,
            format!("stall: rotp did not read the lead note \"{NOTE}\" for 5 min (it is idle)")
        );
        assert_eq!(rig.wakes_of("ack-nudge").len(), 1, "the nudge goes once");
    }
}

#[test]
fn reading_the_inbox_before_3_min_ends_the_lead_note_ladder() {
    let mut rig = world(&json!({}), &[], &json!({}));
    lead_note(&mut rig, "lead-info");
    rig.clock.advance(MIN);
    rig.service.inbox_read("rotp", "lead");
    pass_quiet(&mut rig, 40);
}
