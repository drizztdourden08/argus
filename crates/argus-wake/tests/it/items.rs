//! KEEP: K1.
//! Wakes about the Kanban: an item that sits on a board for far too long reminds the thread that owns it,
//! one wake per thread listing the worst, repeated every 12 hours, and the Overlord hears of projects where
//! it goes on for days. A manual clock drives every tick.

// test-tag: critical

use argus_wake::activity::Level;
use argus_wake::item_wakes::{ItemEnv, ItemWakeConfig, ItemWakes, age_text};
use argus_wake::memory::MemoryBoards;
use argus_wake::reasons::{IMMEDIATE_REASONS, WAKE_REASONS};
use argus_wake::testkit::{DAY, HOUR, MIN, START, SentLookup, board, merge, never_sent};
use argus_wake::time::iso;
use argus_wake::wake::WakeInput;
use serde_json::{Value, json};

fn item(id: &str, status: &str, minutes_ago: i64, extra: &Value) -> Value {
    let mut record = json!({
        "id": id, "title": format!("item {id}"), "thread": "worker", "status": status,
        "createdAt": iso(START - 30 * DAY), "updatedAt": iso(START - minutes_ago * MIN),
    });
    merge(&mut record, extra);
    record
}

fn hours(h: i64) -> i64 {
    h * 60
}

fn empty_board() -> Value {
    json!({
        "threads": {
            "lead": {"id": "lead", "role": "coordinator", "status": "active"},
            "worker": {"id": "worker", "role": "thread", "status": "active"},
            "other": {"id": "other", "role": "thread", "status": "active"},
            "gone": {"id": "gone", "status": "done"},
        },
        "items": {}, "tests": {}, "actions": {}, "agents": {}, "entries": {}, "handedBuild": null,
    })
}

struct World {
    boards: MemoryBoards,
    alpha: Value,
    wakes: ItemWakes,
    events: Vec<WakeInput>,
    last_sent: SentLookup,
}

impl World {
    fn new() -> Self {
        let world = Self {
            boards: MemoryBoards::new(),
            alpha: empty_board(),
            wakes: ItemWakes::new(ItemWakeConfig::default()),
            events: Vec::new(),
            last_sent: never_sent(),
        };
        world.sync();
        world
    }

    fn sync(&self) {
        self.boards.set("alpha", board(&self.alpha));
    }

    fn add(&mut self, record: &Value) {
        self.alpha["items"][record["id"].as_str().expect("id")] = record.clone();
        self.sync();
    }

    fn touch(&mut self, id: &str, at: i64) {
        self.alpha["items"][id]["updatedAt"] = json!(iso(at));
        self.sync();
    }

    fn put(&mut self, collection: &str, record: &Value) {
        self.alpha[collection][record["id"].as_str().expect("id")] = record.clone();
        self.sync();
    }

    fn fresh(&mut self) {
        self.wakes = ItemWakes::new(ItemWakeConfig::default());
    }

    fn tick_at(&mut self, now: i64) -> usize {
        let env = ItemEnv {
            boards: &self.boards,
            main: Some("coordinator".to_owned()),
            last_sent: &*self.last_sent,
        };
        let out = self.wakes.tick(now, &env);
        let sent = out.wakes.len();
        self.events.extend(out.wakes);
        sent
    }

    fn tick(&mut self) -> usize {
        self.tick_at(START)
    }

    fn at(&mut self, minutes: i64) -> usize {
        self.tick_at(START + minutes * MIN)
    }

    fn stale_ids(&self, minutes: i64) -> String {
        let mut ids: Vec<String> = self
            .wakes
            .stale_items(START + minutes * MIN, &self.boards)
            .into_iter()
            .map(|s| s.id)
            .collect();
        ids.sort();
        ids.join(",")
    }

    fn in_scope(&self, scope: &str) -> Vec<&WakeInput> {
        self.events.iter().filter(|e| e.scope == scope).collect()
    }
}

fn reminded() -> World {
    let mut world = World::new();
    world.add(&item("I-1", "doing", hours(5), &json!({})));
    assert_eq!(world.at(0), 1);
    world
}

#[test]
fn the_reason_is_known_to_the_wake_store_and_does_not_close_the_coalescing_window() {
    assert!(WAKE_REASONS.contains(&"kanban-stale"));
    assert!(!IMMEDIATE_REASONS.contains(&"kanban-stale"));
}

#[test]
fn age_text_is_minutes_under_an_hour_hours_under_two_days_then_days() {
    assert_eq!(age_text(40 * MIN), "40m");
    assert_eq!(age_text(5 * HOUR), "5h");
    assert_eq!(age_text(47 * HOUR), "47h");
    assert_eq!(age_text(2 * DAY), "2d");
    assert_eq!(age_text(9 * DAY), "9d");
}

#[test]
fn each_status_has_its_own_threshold_doing_4_h_review_6_h_todo_24_h() {
    let mut world = World::new();
    world.add(&item("I-1", "doing", 234, &json!({})));
    world.add(&item("I-2", "review", 342, &json!({})));
    world.add(&item("I-3", "todo", 1410, &json!({})));
    assert_eq!(world.stale_ids(0), "");
    assert_eq!(world.stale_ids(12), "I-1", "doing is over 4 h");
    assert_eq!(world.stale_ids(24), "I-1,I-2", "review is over 6 h");
    assert_eq!(world.stale_ids(36), "I-1,I-2,I-3", "todo is over 24 h");
    assert_eq!(world.at(36), 1);
    assert!(world.events[0].text.starts_with("3 board items stale: "));
}

#[test]
fn done_dropped_archived_and_deleted_items_are_left_alone() {
    let mut world = World::new();
    world.add(&item("I-1", "done", hours(50), &json!({})));
    world.add(&item("I-2", "dropped", hours(50), &json!({})));
    world.add(&item(
        "I-3",
        "doing",
        hours(50),
        &json!({"archivedAt": iso(START)}),
    ));
    world.add(&item(
        "I-4",
        "doing",
        hours(50),
        &json!({"deletedAt": iso(START)}),
    ));
    assert_eq!(world.tick(), 0);
}

#[test]
fn the_wake_goes_to_the_owning_thread_once_per_tick_for_all_its_items_worst_first_and_says_what_to_do()
 {
    let mut world = World::new();
    world.add(&item("I-12", "doing", hours(5), &json!({})));
    world.add(&item("I-15", "todo", hours(48), &json!({})));
    world.add(&item("I-18", "review", hours(9), &json!({})));
    world.add(&item(
        "I-20",
        "doing",
        hours(6),
        &json!({"thread": "other"}),
    ));
    assert_eq!(world.tick(), 2);
    let mine = world
        .events
        .iter()
        .find(|e| e.thread == "worker")
        .expect("worker");
    assert_eq!(
        (
            mine.scope.as_str(),
            mine.reason.as_str(),
            mine.subject.as_deref()
        ),
        ("alpha", "kanban-stale", Some("worker"))
    );
    assert!(mine.text.starts_with("3 board items stale: I-15 todo 2d with no change, I-18 review 9h, I-12 doing 5h. Per item: start it "), "{}", mine.text);
    assert!(mine.text.contains("update it ") && mine.text.contains("drop or close it "));
    assert!(mine.text.ends_with("Keep the board true."));
    let other = world
        .events
        .iter()
        .find(|e| e.thread == "other")
        .expect("other");
    assert!(
        other
            .text
            .starts_with("1 board item stale: I-20 doing 6h with no change.")
    );
}

#[test]
fn many_items_the_list_is_cut_to_fit_the_300_characters_of_a_wake_with_the_rest_counted() {
    let mut world = World::new();
    for n in 1..=12 {
        world.add(&item(&format!("I-{n}"), "todo", hours(30 + n), &json!({})));
    }
    world.tick();
    let text = &world.events[0].text;
    assert!(
        text.chars().count() <= 300,
        "{} characters",
        text.chars().count()
    );
    assert!(
        text.starts_with("12 board items stale: I-12 todo 42h with no change, "),
        "{text}"
    );
    assert!(text.contains(" more). Per item"), "{text}");
    assert!(text.ends_with("Keep the board true."));
}

#[test]
fn it_repeats_every_12_hours_while_the_item_stays_that_way_and_not_sooner() {
    let mut world = reminded();
    assert_eq!(world.at(11 * 60 + 54), 0);
    assert_eq!(world.at(hours(12)), 1);
    assert_eq!(world.at(23 * 60 + 54), 0);
    assert_eq!(world.at(hours(24)), 1);
}

#[test]
fn a_note_or_a_status_change_counts_as_an_update_the_item_is_quiet_again_only_after_its_threshold()
{
    let mut world = reminded();
    world.touch("I-1", START + 11 * HOUR);
    assert_eq!(world.at(hours(12)), 0, "updated an hour ago");
    assert_eq!(
        world.at(hours(15)),
        1,
        "quiet for 4 h again and the repeat period is over"
    );
}

#[test]
fn what_touches_the_item_counts_a_test_a_to_do_a_finished_agent_a_note() {
    let mut world = World::new();
    for n in 1..=4 {
        world.add(&item(&format!("I-{n}"), "doing", hours(20), &json!({})));
    }
    let hour_ago = iso(START - HOUR);
    world.put("tests", &json!({"id": "T-1", "item": "I-1", "status": "pass", "mode": "auto", "updatedAt": hour_ago}));
    world.put(
        "actions",
        &json!({"id": "H-1", "item": "I-2", "status": "done", "updatedAt": hour_ago}),
    );
    world.put(
        "agents",
        &json!({"id": "A-1", "item": "I-3", "status": "done", "updatedAt": hour_ago}),
    );
    assert_eq!(world.stale_ids(0), "I-4");
    world.put("entries", &json!({"id": "E-1", "item": "I-4", "kind": "log", "status": "applied", "createdAt": iso(START - 30 * MIN)}));
    assert_eq!(world.stale_ids(0), "");
    assert_eq!(
        world.stale_ids(hours(5)),
        "I-1,I-2,I-3,I-4",
        "quiet again, 4 h after the last sign"
    );
}

#[test]
fn an_item_waiting_on_someone_else_is_skipped_an_open_question_a_handed_manual_test_an_owner_to_do_a_live_sub_agent()
 {
    let mut world = World::new();
    for n in 1..=6 {
        world.add(&item(&format!("I-{n}"), "doing", hours(30), &json!({})));
    }
    let old = iso(START - 30 * HOUR);
    world.put("entries", &json!({"id": "Q-1", "item": "I-1", "kind": "question", "status": "open", "createdAt": old}));
    world.put("entries", &json!({"id": "Q-2", "item": "I-6", "kind": "question", "status": "answered", "createdAt": old}));
    world.alpha["handedBuild"] = json!("bundle-4");
    world.put("tests", &json!({"id": "T-1", "item": "I-2", "mode": "manual", "status": "pending", "build": "bundle-4", "updatedAt": old}));
    world.put("tests", &json!({"id": "T-2", "item": "I-6", "mode": "manual", "status": "pending", "build": "bundle-3", "updatedAt": old}));
    world.put(
        "actions",
        &json!({"id": "H-1", "item": "I-3", "status": "pending", "updatedAt": old}),
    );
    world.put(
        "agents",
        &json!({"id": "A-1", "item": "I-4", "status": "working", "updatedAt": old}),
    );
    world.put(
        "agents",
        &json!({"id": "A-2", "item": "I-5", "status": "done", "updatedAt": old}),
    );
    assert_eq!(world.tick(), 1);
    assert!(
        world.events[0]
            .text
            .starts_with("2 board items stale: I-5 doing 30h with no change, I-6 doing 30h."),
        "{}",
        world.events[0].text
    );
}

#[test]
fn a_paused_thread_is_left_alone_and_an_unpaused_one_is_reminded_again() {
    let mut world = World::new();
    world.add(&item("I-1", "doing", hours(30), &json!({})));
    world.alpha["threads"]["worker"]["status"] = json!("paused");
    world.sync();
    assert_eq!(world.tick(), 0);
    world.alpha["threads"]["worker"]["status"] = json!("active");
    world.sync();
    assert_eq!(world.tick(), 1);
}

#[test]
fn an_item_of_an_ended_or_unknown_thread_goes_to_the_lead_of_the_project() {
    let mut world = World::new();
    world.add(&item("I-1", "doing", hours(30), &json!({"thread": "gone"})));
    world.add(&item(
        "I-2",
        "doing",
        hours(31),
        &json!({"thread": "nobody"}),
    ));
    assert_eq!(world.tick(), 1);
    assert_eq!(world.events[0].thread, "lead");
    assert!(
        world.events[0]
            .text
            .starts_with("2 board items stale: I-2 doing 31h with no change, I-1 doing 30h"),
        "{}",
        world.events[0].text
    );
}

#[test]
fn no_lead_to_ask_no_wake() {
    let mut world = World::new();
    world.add(&item("I-1", "doing", hours(30), &json!({"thread": "gone"})));
    world.alpha["threads"]["lead"]["role"] = json!("thread");
    world.sync();
    assert_eq!(world.tick(), 0);
}

#[test]
fn a_restart_does_not_remind_again_at_once_when_the_wakes_table_knows_the_last_one() {
    let mut world = World::new();
    world.add(&item("I-1", "doing", hours(30), &json!({})));
    let reminded = START - 2 * HOUR;
    world.last_sent = Box::new(
        move |scope: &str, thread: &str, reason: &str, subject: &str| {
            if (scope, thread, reason, subject) == ("alpha", "worker", "kanban-stale", "worker") {
                reminded
            } else {
                0
            }
        },
    );
    assert_eq!(
        world.tick(),
        0,
        "reminded two hours ago, before the restart"
    );
    assert_eq!(world.at(hours(10)), 1, "twelve hours after that one");
    world.last_sent = never_sent();
    world.events.clear();
    world.fresh();
    assert_eq!(
        world.tick(),
        1,
        "nothing known: the first reminder comes with the first tick"
    );
    assert_eq!(world.tick(), 0, "and not twice");
}

#[test]
fn the_overlord_hears_once_a_day_of_a_project_with_items_quiet_for_3_days_and_not_before() {
    let mut world = World::new();
    world.add(&item("I-1", "todo", hours(71), &json!({})));
    world.add(&item("I-2", "todo", hours(40), &json!({})));
    world.tick();
    assert_eq!(world.in_scope("portfolio").len(), 0);
    for (id, minutes, status) in [
        ("I-4", hours(216), "todo"),
        ("I-5", hours(96), "doing"),
        ("I-6", hours(84), "review"),
        ("I-7", 4608, "todo"),
        ("I-8", 4464, "todo"),
    ] {
        world.add(&item(id, status, minutes, &json!({})));
    }
    world.events.clear();
    world.fresh();
    world.tick();
    let digest = world.in_scope("portfolio");
    assert_eq!(digest.len(), 1);
    assert_eq!(
        (
            digest[0].thread.as_str(),
            digest[0].reason.as_str(),
            digest[0].subject.as_deref()
        ),
        ("coordinator", "kanban-stale", Some("alpha"))
    );
    assert_eq!(
        digest[0].text,
        "alpha: 5 items stale > 3 days (oldest I-4 todo 9d)"
    );
    world.at(hours(12));
    assert_eq!(
        world.in_scope("alpha").len(),
        2,
        "the lead is reminded again at 12 h"
    );
    assert_eq!(world.in_scope("portfolio").len(), 1, "the Overlord is not");
    world.at(23 * 60 + 54);
    assert_eq!(world.in_scope("portfolio").len(), 1);
    world.at(hours(24));
    assert_eq!(world.in_scope("portfolio").len(), 2, "daily");
}

#[test]
fn a_digest_is_one_activity_line_and_the_reminders_to_the_threads_are_not() {
    let mut world = World::new();
    world.add(&item("I-1", "todo", hours(120), &json!({"thread": "lead"})));
    let env = ItemEnv {
        boards: &world.boards,
        main: Some("coordinator".to_owned()),
        last_sent: &*world.last_sent,
    };
    let out = world.wakes.tick(START, &env);
    assert_eq!(out.wakes.len(), 2);
    assert_eq!(out.activity.len(), 1);
    let line = &out.activity[0];
    assert_eq!(
        (line.kind.as_str(), line.level, line.project.as_deref()),
        ("kanban-stale", Level::Info, Some("alpha"))
    );
    assert_eq!(
        line.text,
        "kanban digest: alpha: 1 item stale > 3 days (oldest I-1 todo 5d)"
    );
}

#[test]
fn items_waiting_on_the_owner_are_not_in_the_digest_and_the_main_board_is_reminded_like_any_other()
{
    let mut world = World::new();
    world.add(&item("I-1", "todo", hours(100), &json!({})));
    world.put("entries", &json!({"id": "Q-1", "item": "I-1", "kind": "question", "status": "open", "createdAt": iso(START - 100 * HOUR)}));
    let mut portfolio = empty_board();
    portfolio["items"]["I-1"] = item("I-1", "todo", hours(100), &json!({}));
    world.boards.set("portfolio", board(&portfolio));
    assert_eq!(world.tick(), 1);
    assert_eq!(
        (
            world.events[0].scope.as_str(),
            world.events[0].thread.as_str()
        ),
        ("portfolio", "worker")
    );
}

#[test]
fn config_the_thresholds_the_repeat_and_the_escalation_come_from_the_items_section() {
    let mut world = World::new();
    world.add(&item("I-1", "doing", hours(2), &json!({})));
    world.add(&item("I-2", "todo", hours(5), &json!({})));
    world.add(&item("I-3", "review", hours(3), &json!({})));
    let config = json!({"staleDoingMs": HOUR, "staleReviewMs": 2 * HOUR, "staleTodoMs": 4 * HOUR, "repeatMs": 2 * HOUR, "escalateMs": 3 * HOUR});
    world.wakes = ItemWakes::new(ItemWakeConfig::from_json(&config));
    assert_eq!(
        world.tick(),
        2,
        "the lead of worker, and the digest for the Overlord"
    );
    assert!(
        world.in_scope("alpha")[0]
            .text
            .starts_with("3 board items stale")
    );
    assert_eq!(
        world.in_scope("portfolio")[0].text,
        "alpha: 2 items stale > 3h (oldest I-2 todo 5h)"
    );
    assert_eq!(world.at(hours(1)), 0);
    assert_eq!(world.at(hours(2)), 1);
}
