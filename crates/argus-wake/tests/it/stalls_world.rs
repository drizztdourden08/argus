//! KEEP: K1.
//! The world the stall tests share: project `rotp` with a live lead and its items, the Overlord's board
//! `portfolio`, the ledger `_portfolio`, and a manual clock that moves a minute at a time with a stall tick after
//! each, the way the real timer does.

use argus_wake::Clock;
use argus_wake::testkit::{MIN, Rig, START, board, by_id, merge, registry_config, thread};
use argus_wake::time::iso;
use argus_wake::wake::{WakeInput, WakeRow};
use serde_json::{Value, json};

/// The words of the wake that asks `rotp` for `I-112`.
pub const REQUEST: &str = "dep D-11: archipelia/I-124 needs your I-112 \"Expose the hook\"";

/// A work item of the lead, written at `START`.
pub fn item(id: &str, status: &str, extra: &Value) -> Value {
    let mut record = json!({
        "id": id, "title": format!("{id} title"), "status": status, "thread": "lead",
        "createdAt": iso(START), "updatedAt": iso(START),
    });
    merge(&mut record, extra);
    record
}

fn rotp(lead: &Value, items: &[Value], extra: &Value) -> Value {
    let mut lead_fields = json!({"lastSeen": iso(START)});
    merge(&mut lead_fields, lead);
    let mut value = json!({
        "threads": {"lead": thread("lead", "lead", &lead_fields)},
        "items": by_id(items),
        "inbox": {},
    });
    merge(&mut value, extra);
    value
}

/// Project `rotp` with an idle lead last seen at `START` and these items, and the Overlord's board.
pub fn world(lead: &Value, items: &[Value], extra: &Value) -> Rig {
    let rig = Rig::new(&registry_config(), false);
    rig.boards.set("rotp", board(&rotp(lead, items, extra)));
    let overlord = thread("lead", "lead", &json!({}));
    rig.boards.set(
        "portfolio",
        board(&json!({"threads": {"lead": overlord}, "inbox": {}})),
    );
    rig
}

/// `rotp` with `I-12` doing since `START` and a lead that is busy, so that only rule 5 can speak.
pub fn silent_world() -> Rig {
    let rig = world(&json!({}), &[item("I-12", "doing", &json!({}))], &json!({}));
    rig.busy.set("rotp", true);
    rig
}

/// The ledger holds `D-11`: `archipelia/I-124` asks `rotp/I-112`, open since `START`.
pub fn open_dep(rig: &Rig, extra: &Value) {
    let mut dep = json!({
        "id": "D-11", "state": "open", "openedAt": iso(START), "title": "Expose the hook",
        "from": {"project": "archipelia", "item": "I-124", "thread": "worker"},
        "to": {"project": "rotp", "item": "I-112", "thread": "lead"},
        "history": [],
    });
    merge(&mut dep, extra);
    rig.boards
        .set("_portfolio", board(&json!({"deps": {"D-11": dep}})));
}

/// The wake that asks the lead of `rotp` for `I-112`.
///
/// # Panics
/// When the wake cannot be recorded.
pub fn dep_request(rig: &mut Rig) {
    let wake = WakeInput::about("rotp", "lead", "dep-request", "D-11#1", REQUEST);
    rig.service.submit(&[wake]).expect("submit");
}

/// Time passes a minute at a time; `before` runs after each minute and before the stall tick.
pub fn pass_with(rig: &mut Rig, minutes: i64, before: impl Fn(&Rig)) {
    for _ in 0..minutes {
        rig.clock.advance(MIN);
        before(rig);
        rig.service.stall_tick();
    }
}

/// Time passes a minute at a time with a stall tick after each.
pub fn pass(rig: &mut Rig, minutes: i64) {
    pass_with(rig, minutes, |_| {});
}

/// The same, and the reminder tick of the Overlord's coalesced line after each stall tick.
pub fn pass_reminding(rig: &mut Rig, minutes: i64) {
    for _ in 0..minutes {
        pass(rig, 1);
        rig.service.reminder_tick();
    }
}

/// The lead of `rotp` is in a turn right now.
pub fn working(rig: &Rig) {
    let now = iso(rig.clock.now_ms());
    rig.edit_thread("rotp", "lead", |t| {
        t.activity = "working".into();
        t.last_seen = now.as_str().into();
    });
}

/// An item of `rotp` is written right now, and given a status.
pub fn touch(rig: &Rig, item: &str, status: &str) {
    let now = iso(rig.clock.now_ms());
    rig.edit("rotp", |b| {
        if let Some(i) = b.items.get_mut(item) {
            i.status = status.into();
            i.rec.updated_at = now.as_str().into();
        }
    });
}

/// The wake is addressed to `thread` of `scope`.
pub fn assert_to(wake: &WakeRow, scope: &str, thread: &str) {
    assert_eq!((wake.scope.as_str(), wake.thread.as_str()), (scope, thread));
}

/// The texts of the activity lines of one kind.
pub fn activity_texts(rig: &Rig, kind: &str) -> Vec<String> {
    rig.activity
        .lines()
        .into_iter()
        .filter(|line| line.kind == kind)
        .map(|line| line.text)
        .collect()
}

/// The owner assigns `id` to the lead of `rotp`.
///
/// # Panics
/// When the wake cannot be recorded.
pub fn assigned(rig: &mut Rig, id: &str) {
    let wake = WakeInput::about(
        "rotp",
        "lead",
        "item-assigned",
        id,
        format!("{id} ({id} title) was assigned to you by the owner"),
    );
    rig.service.submit(&[wake]).expect("submit");
}

/// The one stall line says the idle lead did not read the assignment of `id` for 10 minutes.
pub fn assert_unread_assignment(rig: &Rig, id: &str) {
    let stalls = rig.wakes_of("stall");
    assert_eq!(stalls.len(), 1);
    assert_eq!(
        stalls[0].text,
        format!("stall: rotp did not read item-assigned {id} for 10 min (it is idle)")
    );
}

/// A pass of `minutes` raises no ack nudge, no stall wake and no owner notice.
pub fn pass_quiet(rig: &mut Rig, minutes: i64) {
    pass(rig, minutes);
    assert_eq!(rig.wakes_of("ack-nudge").len(), 0);
    assert_eq!(rig.wakes_of("stall").len(), 0);
    assert_eq!(rig.owner.notices().len(), 0);
}
