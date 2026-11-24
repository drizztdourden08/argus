//! KEEP: K1.
//! Wakes about cross-board dependencies: each history row of the ledger on the portfolio board tells one
//! side once (request, resolved, blocked, cancelled, reopened), to the thread of the item while it is live
//! and else to the project's lead, and a restart sends nothing again. A manual clock drives every tick.

// test-tag: critical

use argus_wake::dep_wakes::{DepEnv, DepWakeConfig, DepWakes};
use argus_wake::memory::MemoryBoards;
use argus_wake::reasons::{IMMEDIATE_REASONS, PORTFOLIO, WAKE_REASONS};
use argus_wake::testkit::{DAY, MIN, START, SentLookup, board, never_sent, sent_wakes};
use argus_wake::time::iso;
use argus_wake::wake::WakeInput;
use argus_wake::wake_policy::{WakeClass, class_of};
use serde_json::{Value, json};

const REASONS: [&str; 5] = [
    "dep-request",
    "dep-resolved",
    "dep-blocked",
    "dep-cancelled",
    "dep-reopened",
];

fn step(seq: u64, from: &str, to: &str, reason: Option<&str>) -> Value {
    json!({
        "seq": seq, "at": iso(START - 5 * MIN), "by": "alpha/T-2",
        "from": from, "to": to, "reason": reason,
    })
}

fn dep(history: &[Value]) -> Value {
    json!({
        "id": "D-4", "state": "open",
        "from": {"project": "alpha", "item": "I-7", "thread": "worker"},
        "to": {"project": "beta", "item": "I-12", "thread": "builder"},
        "title": "Expose the save hook", "reason": null, "history": history,
    })
}

fn project(items: &Value) -> Value {
    json!({
        "threads": {
            "lead": {"id": "lead", "role": "lead", "status": "active"},
            "worker": {"id": "worker", "role": "thread", "status": "active"},
            "builder": {"id": "builder", "role": "thread", "status": "active"},
        },
        "items": items,
    })
}

struct World {
    boards: MemoryBoards,
    alpha: Value,
    beta: Value,
    ledger: Value,
    wakes: DepWakes,
    events: Vec<WakeInput>,
    last_sent: SentLookup,
}

impl World {
    fn new(history: &[Value]) -> Self {
        let mut world = Self {
            boards: MemoryBoards::new(),
            alpha: project(&json!({"I-7": {"id": "I-7", "thread": "worker"}})),
            beta: project(&json!({"I-12": {"id": "I-12", "thread": "builder"}})),
            ledger: json!({"deps": {}}),
            wakes: DepWakes::new(DepWakeConfig::default()),
            events: Vec::new(),
            last_sent: never_sent(),
        };
        world.ledger["deps"]["D-4"] = dep(history);
        world.sync();
        world
    }

    fn sync(&self) {
        self.boards.set("alpha", board(&self.alpha));
        self.boards.set("beta", board(&self.beta));
        self.boards.set(PORTFOLIO, board(&self.ledger));
    }

    fn history(&mut self, history: &[Value]) {
        self.ledger["deps"]["D-4"]["history"] = json!(history);
        self.sync();
    }

    fn tick(&mut self) -> usize {
        let env = DepEnv {
            boards: &self.boards,
            last_sent: &*self.last_sent,
        };
        let out = self.wakes.tick(START, &env);
        let sent = out.len();
        self.events.extend(out);
        sent
    }

    fn restart(&mut self) {
        self.last_sent = sent_wakes(&self.events, START);
        self.wakes = DepWakes::new(DepWakeConfig::default());
    }

    fn lines(&self) -> Vec<(&str, &str, &str, &str, &str)> {
        self.events
            .iter()
            .map(|e| {
                (
                    e.scope.as_str(),
                    e.thread.as_str(),
                    e.reason.as_str(),
                    e.subject.as_deref().unwrap_or(""),
                    e.text.as_str(),
                )
            })
            .collect()
    }
}

fn whole_life() -> Vec<Value> {
    vec![
        step(1, "opening", "open", None),
        step(2, "open", "blocked", Some("needs the schema first")),
        step(3, "blocked", "open", None),
        step(4, "open", "done", None),
        step(5, "done", "open", None),
        step(6, "open", "cancelled", Some("not needed")),
    ]
}

#[test]
fn the_five_reasons_are_known_two_close_the_window_at_once_and_two_are_notices() {
    for reason in REASONS {
        assert!(WAKE_REASONS.contains(&reason), "{reason}");
    }
    let immediate: Vec<&str> = REASONS
        .into_iter()
        .filter(|r| IMMEDIATE_REASONS.contains(r))
        .collect();
    assert_eq!(immediate, ["dep-resolved", "dep-blocked"]);
    let passive: Vec<&str> = REASONS
        .into_iter()
        .filter(|r| class_of(r) == WakeClass::Passive)
        .collect();
    assert_eq!(passive, ["dep-cancelled", "dep-reopened"]);
}

#[test]
fn each_transition_tells_its_side_once_with_the_words_of_the_contract() {
    let mut world = World::new(&whole_life());
    assert_eq!(world.tick(), 6);
    assert_eq!(
        world.lines(),
        [
            (
                "beta",
                "builder",
                "dep-request",
                "D-4#1",
                "dep D-4: alpha/I-7 needs your I-12 \"Expose the save hook\""
            ),
            (
                "alpha",
                "worker",
                "dep-blocked",
                "D-4#2",
                "beta/I-12 is blocked on you (D-4): needs the schema first"
            ),
            (
                "alpha",
                "worker",
                "dep-reopened",
                "D-4#3",
                "beta/I-12 is open again (D-4)"
            ),
            (
                "alpha",
                "worker",
                "dep-resolved",
                "D-4#4",
                "beta/I-12 is done (D-4): resume I-7"
            ),
            (
                "alpha",
                "worker",
                "dep-reopened",
                "D-4#5",
                "beta/I-12 is open again (D-4)"
            ),
            (
                "beta",
                "builder",
                "dep-cancelled",
                "D-4#6",
                "alpha/I-7 cancelled its request I-12 (D-4)"
            ),
        ]
    );
    assert_eq!(world.tick(), 0, "every row once");
}

#[test]
fn dropped_and_gone_resolve_the_request_like_done_and_name_their_state() {
    let mut world = World::new(&[
        step(1, "open", "dropped", None),
        step(2, "open", "gone", None),
    ]);
    assert_eq!(world.tick(), 2);
    let texts: Vec<&str> = world.events.iter().map(|e| e.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "beta/I-12 is dropped (D-4): resume I-7",
            "beta/I-12 is gone (D-4): resume I-7"
        ]
    );
    assert!(world.events.iter().all(|e| e.reason == "dep-resolved"));
}

#[test]
fn a_new_row_wakes_once_and_a_restart_does_not_send_the_old_rows_again() {
    let mut life = whole_life();
    let mut world = World::new(&life[..3]);
    assert_eq!(world.tick(), 3);
    world.restart();
    assert_eq!(world.tick(), 0, "the wakes table knows all three");
    life.truncate(4);
    world.history(&life);
    assert_eq!(world.tick(), 1, "only the new row");
    assert_eq!(world.events[3].subject.as_deref(), Some("D-4#4"));
    assert_eq!(world.events[3].reason, "dep-resolved");
}

#[test]
fn a_row_that_the_table_cannot_tell_about_any_more_or_that_says_nothing_is_left_out() {
    let old = json!({
        "seq": 1, "at": iso(START - 8 * DAY), "from": "opening", "to": "open",
    });
    let mut world = World::new(&[
        old,
        step(2, "opening", "opening", None),
        step(3, "open", "open", None),
        step(0, "open", "done", None),
        json!({"at": iso(START), "from": "open", "to": "done"}),
    ]);
    assert_eq!(world.tick(), 0);
    world.ledger["deps"]["D-4"]["to"]["item"] = json!("");
    world.history(&[step(4, "opening", "open", None)]);
    assert_eq!(world.tick(), 0, "the words need both items");
}

fn first_heard(world: &World) -> (&str, &str) {
    (
        world.events[0].scope.as_str(),
        world.events[0].thread.as_str(),
    )
}

#[test]
fn the_thread_of_the_item_hears_it_while_live_else_the_lead_and_a_paused_one_waits_for_its_resume()
{
    let mut world = World::new(&[step(1, "open", "done", None)]);
    world.alpha["threads"]["worker"]["status"] = json!("done");
    world.sync();
    assert_eq!(world.tick(), 1);
    assert_eq!(
        first_heard(&world),
        ("alpha", "lead"),
        "the thread ended: its lead hears"
    );

    let mut world = World::new(&[step(1, "open", "done", None)]);
    world.alpha["threads"]["worker"]["status"] = json!("paused");
    world.sync();
    assert_eq!(world.tick(), 0, "paused: nobody is woken");
    world.alpha["threads"]["worker"]["status"] = json!("active");
    world.sync();
    assert_eq!(world.tick(), 1, "resumed: it hears now");
    assert_eq!(world.events[0].thread, "worker");

    let mut world = World::new(&[step(1, "opening", "open", None)]);
    world.beta["items"] = json!({});
    world.beta["threads"]["builder"]["status"] = json!("archived");
    world.sync();
    assert_eq!(world.tick(), 1);
    assert_eq!(
        first_heard(&world),
        ("beta", "lead"),
        "the item is gone and its thread ended: the lead hears"
    );
}

#[test]
fn nothing_is_sent_without_a_ledger_a_board_of_the_side_or_when_dependencies_are_off() {
    let mut world = World::new(&[step(1, "open", "done", None)]);
    world.boards.set(PORTFOLIO, board(&json!({})));
    assert_eq!(world.tick(), 0, "no ledger");
    world.sync();
    world.alpha["threads"] = json!({});
    world.sync();
    assert_eq!(world.tick(), 0, "no thread and no lead to hear");

    let mut world = World::new(&[step(1, "open", "done", None)]);
    world.wakes = DepWakes::new(DepWakeConfig::from_json(&json!({"enabled": false})));
    assert_eq!(world.tick(), 0, "deps.enabled is false");
    assert_eq!(
        DepWakeConfig::from_json(&json!({})),
        DepWakeConfig::default()
    );
}
