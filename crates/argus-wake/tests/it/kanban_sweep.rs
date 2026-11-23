//! KEEP: K1.
//! The Kanban sweep: the alert rules of a board's Kanban config. A board with none runs the default,
//! which tells what the stale reminders always told; the other kinds name who hears and how often.

// test-tag: critical

use argus_board::kanban_config::default_config;
use argus_wake::item_wakes::{ItemEnv, ItemTick, ItemWakeConfig, ItemWakes};
use argus_wake::memory::MemoryBoards;
use argus_wake::testkit::{DAY, HOUR, MIN, START, board, merge, never_sent};
use argus_wake::time::iso;
use serde_json::{Value, json};

fn item(id: &str, status: &str, hours_ago: i64, extra: &Value) -> Value {
    let mut record = json!({
        "id": id, "title": format!("item {id}"), "thread": "worker", "status": status,
        "createdAt": iso(START - 30 * DAY), "updatedAt": iso(START - hours_ago * HOUR),
    });
    merge(&mut record, extra);
    record
}

fn world(items: &[Value], extra: &Value) -> Value {
    let mut doc = json!({
        "threads": {
            "lead": {"id": "lead", "role": "coordinator", "status": "active"},
            "worker": {"id": "worker", "role": "thread", "status": "active"},
            "gone": {"id": "gone", "role": "thread", "status": "archived"},
        },
        "items": {}, "tests": {}, "actions": {}, "agents": {}, "entries": {},
    });
    for record in items {
        doc["items"][record["id"].as_str().expect("id")] = record.clone();
    }
    merge(&mut doc, extra);
    doc
}

fn tick(sweep: &mut ItemWakes, doc: &Value, at: i64) -> ItemTick {
    let boards = MemoryBoards::new();
    boards.set("alpha", board(doc));
    let last_sent = never_sent();
    let env = ItemEnv {
        boards: &boards,
        main: None,
        last_sent: &*last_sent,
    };
    sweep.tick(at, &env)
}

fn kanban(columns: &Value, rules: &Value) -> Value {
    json!({"kanban": {"columns": columns, "rules": rules}})
}

fn rule(id: &str, kind: &str, params: &Value, to: &str, wake: bool) -> Value {
    json!({
        "id": id, "kind": kind, "params": params,
        "alert": {"to": to, "wake": wake, "repeatMs": 12 * HOUR},
    })
}

fn column(id: &str, lanes: &[&str], rules: &[&str], limit: &Value) -> Value {
    let mut col = json!({
        "id": id, "name": id, "match": {"lanes": lanes}, "rules": rules,
    });
    if !limit.is_null() {
        col["limit"] = limit.clone();
    }
    col
}

fn fresh() -> ItemWakes {
    ItemWakes::new(ItemWakeConfig::default())
}

#[test]
fn a_board_with_no_config_and_one_with_the_default_config_tell_the_same_stale_wakes() {
    let items = [
        item("I-1", "doing", 5, &json!({})),
        item("I-2", "review", 7, &json!({})),
        item("I-3", "todo", 30, &json!({})),
        item("I-4", "doing", 2, &json!({})),
    ];
    let plain = tick(&mut fresh(), &world(&items, &json!({})), START);
    let stored = json!({"kanban": serde_json::to_value(default_config()).expect("config")});
    let same = tick(&mut fresh(), &world(&items, &stored), START);
    assert_eq!(plain.wakes.len(), 1);
    assert_eq!(plain.wakes, same.wakes);
    assert_eq!(plain.wakes[0].reason, "kanban-stale");
    assert_eq!(plain.wakes[0].thread, "worker");
    assert!(plain.wakes[0].text.starts_with("3 board items stale: "));
    assert_eq!(plain.owner.len(), 0);
}

#[test]
fn the_default_tells_the_same_thread_again_only_after_twelve_hours() {
    let doc = world(&[item("I-1", "doing", 5, &json!({}))], &json!({}));
    let mut sweep = fresh();
    assert_eq!(tick(&mut sweep, &doc, START).wakes.len(), 1);
    assert_eq!(tick(&mut sweep, &doc, START + 11 * HOUR).wakes.len(), 0);
    assert_eq!(tick(&mut sweep, &doc, START + 12 * HOUR).wakes.len(), 1);
}

#[test]
fn dwell_uses_the_limit_of_its_rule_and_the_lane_clock_of_the_item() {
    let cfg = kanban(
        &json!([column("doing", &["doing"], &["slow"], &Value::Null)]),
        &json!([rule("slow", "dwell", &json!({"ms": HOUR}), "lead", true)]),
    );
    let fresh_lane = item("I-1", "doing", 0, &json!({"laneAt": iso(START - 30 * MIN)}));
    let old_lane = item("I-2", "doing", 3, &json!({"laneAt": iso(START - 3 * HOUR)}));
    let out = tick(&mut fresh(), &world(&[fresh_lane, old_lane], &cfg), START);
    assert_eq!(out.wakes.len(), 1);
    assert_eq!(out.wakes[0].thread, "lead");
    assert!(out.wakes[0].text.contains("I-2 doing 3h"));
    assert!(!out.wakes[0].text.contains("I-1"));
}

#[test]
fn an_alert_that_does_not_wake_lands_as_an_inbox_notice() {
    let cfg = kanban(
        &json!([column("doing", &["doing"], &["slow"], &Value::Null)]),
        &json!([rule(
            "slow",
            "dwell",
            &json!({"ms": HOUR}),
            "item-thread",
            false
        )]),
    );
    let out = tick(
        &mut fresh(),
        &world(&[item("I-1", "doing", 3, &json!({}))], &cfg),
        START,
    );
    assert_eq!(out.wakes.len(), 1);
    assert_eq!(out.wakes[0].reason, "kanban-notice");
    assert_eq!(out.wakes[0].thread, "worker");
}

#[test]
fn assignee_gone_names_an_agent_that_stopped_and_a_thread_that_was_archived() {
    let cfg = kanban(
        &json!([column("doing", &["doing"], &["gone"], &Value::Null)]),
        &json!([rule("gone", "assignee-gone", &json!({}), "lead", true)]),
    );
    let extra = merge_all(
        &cfg,
        &json!({"agents": {
            "A-1": {"id": "A-1", "thread": "worker", "status": "done"},
            "A-2": {"id": "A-2", "thread": "worker", "status": "working"},
        }}),
    );
    let items = [
        item("I-1", "doing", 0, &json!({"assignee": "agent:A-1"})),
        item("I-2", "doing", 0, &json!({"assignee": "agent:A-2"})),
        item("I-3", "doing", 0, &json!({"assignee": "gone"})),
        item("I-4", "doing", 0, &json!({"assignee": "worker"})),
        item("I-5", "doing", 0, &json!({"assignee": "owner"})),
        item("I-6", "doing", 0, &json!({})),
    ];
    let out = tick(&mut fresh(), &world(&items, &extra), START);
    assert_eq!(out.wakes.len(), 1);
    assert_eq!(out.wakes[0].reason, "kanban-alert");
    assert_eq!(out.wakes[0].thread, "lead");
    let text = &out.wakes[0].text;
    assert!(text.starts_with("2 board items lost the assignee: "));
    assert!(text.contains("I-1 doing, assignee agent:A-1 is gone"));
    assert!(text.contains("I-3 doing, assignee gone is gone"));
}

fn merge_all(a: &Value, b: &Value) -> Value {
    let mut out = a.clone();
    merge(&mut out, b);
    out
}

#[test]
fn over_limit_fires_when_a_column_holds_more_than_its_max() {
    let limit = json!({"max": 2, "mode": "warn"});
    let cfg = kanban(
        &json!([column("doing", &["doing"], &["over"], &limit)]),
        &json!([rule("over", "over-limit", &json!({}), "lead", true)]),
    );
    let three = [
        item("I-1", "doing", 0, &json!({})),
        item("I-2", "doing", 0, &json!({})),
        item("I-3", "doing", 0, &json!({})),
    ];
    let out = tick(&mut fresh(), &world(&three, &cfg), START);
    assert_eq!(out.wakes.len(), 1);
    assert_eq!(out.wakes[0].thread, "lead");
    assert!(out.wakes[0].text.contains("doing holds 3 of 2"));
    let two = tick(&mut fresh(), &world(&three[..2], &cfg), START);
    assert_eq!(two.wakes.len(), 0);
}

#[test]
fn a_column_in_alert_mode_tells_the_lead_even_with_no_rule() {
    let limit = json!({"max": 1, "mode": "alert"});
    let cfg = kanban(
        &json!([column("doing", &["doing"], &[], &limit)]),
        &json!([]),
    );
    let items = [
        item("I-1", "doing", 0, &json!({})),
        item("I-2", "doing", 0, &json!({})),
    ];
    let out = tick(&mut fresh(), &world(&items, &cfg), START);
    assert_eq!(out.wakes.len(), 1);
    assert_eq!(out.wakes[0].thread, "lead");
    assert_eq!(out.wakes[0].reason, "kanban-alert");
}

#[test]
fn unassigned_waits_for_its_time_in_the_column() {
    let cfg = kanban(
        &json!([column("todo", &["todo"], &["loose"], &Value::Null)]),
        &json!([rule(
            "loose",
            "unassigned",
            &json!({"ms": 2 * HOUR}),
            "lead",
            true
        )]),
    );
    let items = [
        item("I-1", "todo", 3, &json!({"laneAt": iso(START - 3 * HOUR)})),
        item("I-2", "todo", 1, &json!({"laneAt": iso(START - HOUR)})),
        item(
            "I-3",
            "todo",
            5,
            &json!({"laneAt": iso(START - 5 * HOUR), "assignee": "worker"}),
        ),
    ];
    let out = tick(&mut fresh(), &world(&items, &cfg), START);
    assert_eq!(out.wakes.len(), 1);
    let text = &out.wakes[0].text;
    assert!(text.starts_with("1 board item without an assignee: "));
    assert!(text.contains("I-1 todo, no assignee for 3h"));
}

#[test]
fn an_alert_to_the_owner_is_a_notice_for_him_and_no_wake() {
    let cfg = kanban(
        &json!([column("doing", &["doing"], &["gone"], &Value::Null)]),
        &json!([rule("gone", "assignee-gone", &json!({}), "owner", true)]),
    );
    let doc = world(
        &[item("I-1", "doing", 0, &json!({"assignee": "gone"}))],
        &cfg,
    );
    let mut sweep = fresh();
    let out = tick(&mut sweep, &doc, START);
    assert_eq!(out.wakes.len(), 0);
    assert_eq!(out.owner.len(), 1);
    assert_eq!(out.owner[0].project, "alpha");
    assert_eq!(out.owner[0].topic, "kanban");
    assert_eq!(tick(&mut sweep, &doc, START + HOUR).owner.len(), 0);
    assert_eq!(tick(&mut sweep, &doc, START + 12 * HOUR).owner.len(), 1);
}

#[test]
fn an_alert_to_a_named_thread_goes_to_that_thread_and_a_paused_one_hears_nothing() {
    let cfg = kanban(
        &json!([column("doing", &["doing"], &["gone"], &Value::Null)]),
        &json!([rule("gone", "assignee-gone", &json!({}), "worker", true)]),
    );
    let item_gone = item("I-1", "doing", 0, &json!({"assignee": "gone"}));
    let doc = world(&[item_gone], &cfg);
    assert_eq!(tick(&mut fresh(), &doc, START).wakes[0].thread, "worker");
    let paused = merge_all(&doc, &json!({"threads": {"worker": {"status": "paused"}}}));
    assert_eq!(tick(&mut fresh(), &paused, START).wakes.len(), 0);
}

#[test]
fn a_damaged_config_falls_back_to_the_default() {
    let doc = world(
        &[item("I-1", "doing", 5, &json!({}))],
        &json!({"kanban": {"columns": []}}),
    );
    let out = tick(&mut fresh(), &doc, START);
    assert_eq!(out.wakes.len(), 1);
    assert_eq!(out.wakes[0].reason, "kanban-stale");
}
