//! KEEP: S2 (the wake service on in-memory storage, the owner's post to the listener).
//! The wake service end to end on in-memory storage: the owner posts the importer applies become events for
//! the listeners (coalesced, queued, replayed), a failed run wakes the lead, the page rows, the settings.

// test-tag: standard

use argus_wake::activity::Level;
use argus_wake::config::{DEFAULT_RESTART_GRACE_MS, DEFAULT_WINDOW_MS, WakeConfig};
use argus_wake::engine::AttachOptions;
use argus_wake::policy::RunView;
use argus_wake::testkit::{HOUR, MIN, Rig, SEC, Tap, applied, board, failed_run, thread, wake};
use argus_wake::time::parse_ms;
use argus_wake::wake_policy::WakePolicy;
use serde_json::{Value, json};

fn config(window_ms: i64) -> WakeConfig {
    WakeConfig {
        window_ms,
        policy: WakePolicy::off(),
        ..WakeConfig::default()
    }
}

fn alpha(tests: &[(&str, &str)], i2_thread: &str, ui_status: &str) -> Value {
    let tests: serde_json::Map<String, Value> = tests
        .iter()
        .map(|(id, status)| ((*id).to_owned(), json!({"id": id, "thread": "ui", "build": "b1", "status": status, "mode": "manual", "item": "I-1"})))
        .collect();
    json!({
        "threads": {
            "coordinator": thread("coordinator", "coordinator", &json!({})),
            "ui": thread("ui", "thread", &json!({"status": ui_status})),
            "api": thread("api", "thread", &json!({})),
        },
        "items": {"I-2": {"id": "I-2", "title": "Settings page", "thread": i2_thread}},
        "tests": tests,
        "entries": {"Q-1": {"id": "Q-1", "kind": "question", "thread": "ui"}},
    })
}

fn tests_with(done: &[(&'static str, &'static str)]) -> Vec<(&'static str, &'static str)> {
    ["T-2", "T-3", "T-4", "T-5"]
        .iter()
        .map(|id| {
            (
                *id,
                done.iter()
                    .find(|(t, _)| t == id)
                    .map_or("pending", |(_, status)| *status),
            )
        })
        .collect()
}

fn result(entry_id: &str, target: &str, outcome: &str, note: &str) -> Value {
    json!({"id": entry_id, "kind": "result", "replyTo": target, "outcome": outcome, "text": note})
}

fn rig(window_ms: i64) -> Rig {
    let rig = Rig::new(&config(window_ms), false);
    rig.boards
        .set("alpha", board(&alpha(&tests_with(&[]), "ui", "active")));
    rig
}

fn listen(rig: &mut Rig, scope: &str, thread: &str) -> Tap {
    rig.listen(scope, thread, AttachOptions::default())
}

fn post(rig: &mut Rig, entry: &Value, after: &Value, before: &Value) {
    rig.service
        .on_applied(&applied("alpha", entry, after, before));
}

const FAILED: [(&str, &str); 1] = [("T-2", "fail")];

const PARTIAL: [(&str, &str); 2] = [("T-2", "fail"), ("T-3", "partial")];

fn alpha_with(done: &[(&'static str, &'static str)]) -> Value {
    alpha(&tests_with(done), "ui", "active")
}

#[test]
fn wakes_inside_the_window_arrive_as_one_event_and_another_thread_hears_nothing() {
    let mut rig = rig(150);
    let mut ui = listen(&mut rig, "alpha", "ui");
    let mut api = listen(&mut rig, "alpha", "api");
    let note = "The error text is missing from the form";
    post(
        &mut rig,
        &result("O-1", "T-2", "fail", note),
        &alpha_with(&FAILED),
        &alpha_with(&[]),
    );
    post(
        &mut rig,
        &result("O-2", "T-3", "partial", "the icon does not change"),
        &alpha_with(&PARTIAL),
        &alpha_with(&FAILED),
    );
    rig.advance(150);
    let event = ui.events()[0].clone();
    assert_eq!(
        (event.scope.as_str(), event.thread.as_str(), event.count),
        ("alpha", "ui", 2)
    );
    assert_eq!(
        event.lines,
        [
            format!("T-2 failed: {note}"),
            "T-3 partial: the icon does not change".to_owned()
        ]
    );
    assert!(event.id.starts_with("W-") && parse_ms(&event.at).is_some());
    rig.advance(400);
    assert_eq!((ui.count(), api.count()), (1, 0), "one event, not two");
}

#[test]
fn a_failed_run_wakes_the_lead_at_once_and_leaves_a_line_in_the_activity_log() {
    let mut rig = rig(150);
    let mut star = listen(&mut rig, "alpha", "*");
    rig.service.on_run_end(&failed_run());
    assert_eq!(
        star.lines(),
        [[
            "R-6 launcher-master failed (exit 1): Node.js v24.8.0. Open its log, fix the cause and tell the owner."
        ]]
    );
    let lines = rig.activity.lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(
        (
            lines[0].kind.as_str(),
            lines[0].level,
            lines[0].project.as_deref()
        ),
        ("run-failed", Level::Error, Some("alpha"))
    );
    assert_eq!(
        lines[0].text,
        "alpha: R-6 launcher-master failed (exit 1): Node.js v24.8.0"
    );
    rig.service.on_run_end(&RunView {
        status: "stopped".to_owned(),
        ..failed_run()
    });
    rig.advance(450);
    assert_eq!(
        (star.count(), rig.activity.lines().len()),
        (1, 1),
        "a stop by someone is not a failure to act on"
    );
}

#[test]
fn the_page_rows_are_newest_first_filtered_by_thread_and_the_limit_is_clamped() {
    let mut rig = rig(0);
    for (thread, text) in [("ui", "one"), ("api", "two"), ("ui", "three")] {
        rig.service.submit(&[wake(thread, text)]).expect("submit");
    }
    let ids = |rig: &Rig, limit: Option<usize>, thread: Option<&str>| -> Vec<String> {
        rig.service
            .list("alpha", limit, thread)
            .expect("list")
            .into_iter()
            .map(|d| d.id)
            .collect()
    };
    assert_eq!(ids(&rig, None, None), ["W-3", "W-2", "W-1"]);
    assert_eq!(ids(&rig, None, Some("ui")), ["W-3", "W-1"]);
    assert_eq!(ids(&rig, None, Some("*")), ["W-3", "W-2", "W-1"]);
    assert_eq!(ids(&rig, Some(0), None), ["W-3"], "a limit under 1 is 1");
    assert_eq!(ids(&rig, Some(2), None), ["W-3", "W-2"]);
}

#[test]
fn a_store_that_fails_does_not_break_the_reminder_ticks() {
    let mut value = alpha(&[], "ui", "active");
    value["items"]["I-1"] = json!({"id": "I-1", "title": "t", "thread": "ui", "status": "doing", "createdAt": "2026-09-01T00:00:00.000Z", "updatedAt": "2026-09-01T00:00:00.000Z"});
    let mut rig = rig(0);
    rig.boards.set("alpha", board(&value));
    rig.store.set_failing(true);
    rig.clock.advance(HOUR);
    assert_eq!(
        rig.service.item_tick(),
        2,
        "the reminder and the digest are made, and not sent"
    );
    assert_eq!(rig.service.agent_tick(), 0);
    rig.service.tick();
    assert_eq!(rig.store.all().len(), 0);
}

#[test]
fn the_service_says_when_it_has_work_next() {
    let mut rig = rig(150);
    rig.service.start();
    assert_eq!(
        rig.service.until_next_ms(),
        5 * SEC,
        "the registry check is the first job"
    );
    rig.service.submit(&[wake("ui", "x")]).expect("submit");
    assert_eq!(rig.service.until_next_ms(), 150, "a window closes first");
    rig.advance(150);
    assert_eq!(rig.service.until_next_ms(), 5 * SEC - 150);
    rig.clock.advance(10 * MIN);
    assert_eq!(rig.service.until_next_ms(), 0, "overdue jobs are due now");
}

#[test]
fn the_settings_come_from_config_json_and_an_invalid_value_keeps_its_default() {
    let defaults = WakeConfig::default();
    assert_eq!(defaults.window_ms, DEFAULT_WINDOW_MS);
    assert_eq!(defaults.registry.restart_grace_ms, DEFAULT_RESTART_GRACE_MS);
    let set = WakeConfig::from_json(&json!({
        "wake": {"windowMs": 150, "noListenerMs": 300, "reactionMs": 60000, "tickMs": 50, "aliveMs": 1, "boardGraceMs": "7000"},
        "agents": {"staleWakeMs": 1000},
        "items": {"repeatMs": 2000},
    }));
    assert_eq!(set.window_ms, 150);
    assert_eq!(
        (
            set.registry.no_listener_ms,
            set.registry.reaction_ms,
            set.registry.tick_ms,
            set.registry.alive_ms
        ),
        (300, 60_000, 50, 1)
    );
    assert_eq!(set.registry.board_grace_ms, 7000);
    assert_eq!(
        (set.agents.stale_wake_ms, set.items.repeat_ms),
        (1000, 2000)
    );
    let bad = WakeConfig::from_json(
        &json!({"wake": {"windowMs": -5, "tickMs": 5, "noListenerMs": 0, "reactionMs": null, "boardGraceMs": ""}}),
    );
    assert_eq!(bad.window_ms, DEFAULT_WINDOW_MS);
    assert_eq!(
        bad.registry,
        WakeConfig::default().registry,
        "tickMs under 20, a zero, null and an empty string all keep the default"
    );
    assert_eq!(
        WakeConfig::from_json(&json!({"wake": {"windowMs": 0}})).window_ms,
        0,
        "0 closes at once"
    );
}

#[test]
fn a_board_with_odd_field_types_still_reads() {
    let odd = board(&json!({
        "threads": {"ui": {"id": 7, "role": ["coordinator"], "status": "active", "placeholder": 0}, "bad": "text"},
        "items": [],
        "tests": {"T-1": {"id": "T-1", "thread": 3, "mode": "manual"}, "T-2": 5},
        "handedBuild": 9,
    }));
    assert_eq!(odd.threads.len(), 1);
    assert!(odd.threads["ui"].is_live());
    assert_eq!(odd.coordinator(), None);
    assert_eq!(odd.tests.len(), 1);
}
