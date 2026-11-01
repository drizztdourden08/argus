//! Wakes about sub-agents: a quiet agent wakes the thread that owns it, and an agent on a model of opus
//! level or above wakes it for a check-in every 15 to 30 minutes (the lead's choice, default 20).

use std::collections::HashMap;

use argus_wake::agent_wakes::{AgentWakeConfig, AgentWakes, checkin_every, high_tier};
use argus_wake::board::AgentView;
use argus_wake::memory::MemoryBoards;
use argus_wake::reasons::{EXTERNAL_REASONS, WAKE_REASONS};
use argus_wake::source::{AgentLive, NoLive};
use argus_wake::testkit::{HOUR, MIN, START, board};
use argus_wake::time::iso;
use argus_wake::wake::WakeInput;
use serde_json::{Value, json};

struct Live(HashMap<String, String>);

impl AgentLive for Live {
    fn last_activity_at(&self, _scope: &str, agent: &str) -> Option<String> {
        self.0.get(agent).cloned()
    }
}

fn agent(id: &str, extra: &Value) -> Value {
    let mut record = json!({
        "id": id, "name": format!("agent {id}"), "thread": "lead", "model": "sonnet", "status": "working",
        "task": "build the thing", "startedAt": iso(START), "updatedAt": iso(START),
    });
    argus_wake::testkit::merge(&mut record, extra);
    record
}

fn view(value: &Value) -> AgentView {
    serde_json::from_value(value.clone()).expect("agent")
}

struct World {
    boards: MemoryBoards,
    json: Value,
    live: HashMap<String, String>,
    busy: Vec<String>,
    wakes: AgentWakes,
    events: Vec<WakeInput>,
}

impl World {
    fn new() -> Self {
        let json = json!({
            "threads": {
                "lead": {"id": "lead", "role": "coordinator", "status": "active"},
                "ui": {"id": "ui", "role": "thread", "status": "active"},
                "old": {"id": "old", "status": "done"},
            },
            "agents": {"A-1": agent("A-1", &json!({})), "A-2": agent("A-2", &json!({"model": "opus", "thread": "ui"}))},
        });
        let world = Self {
            boards: MemoryBoards::new(),
            json,
            live: HashMap::new(),
            busy: ["A-1", "A-2", "A-3"].map(str::to_owned).to_vec(),
            wakes: AgentWakes::new(AgentWakeConfig::default(), START),
            events: Vec::new(),
        };
        world.sync();
        world
    }

    fn sync(&self) {
        self.boards.set("alpha", board(&self.json));
    }

    fn set(&mut self, id: &str, field: &str, value: &Value) {
        self.json["agents"][id][field] = value.clone();
        self.sync();
    }

    fn add(&mut self, id: &str, extra: &Value) {
        self.json["agents"][id] = agent(id, extra);
        self.sync();
    }

    fn tick(&mut self, now: i64) -> usize {
        let out = self.wakes.tick(now, &self.boards, &Live(self.live.clone()));
        let sent = out.len();
        self.events.extend(out);
        sent
    }

    fn at(&mut self, minutes: i64) -> usize {
        let now = START + minutes * MIN;
        for id in self.busy.clone() {
            self.live.insert(id, iso(now - MIN));
        }
        self.tick(now)
    }

    fn reasons(&self, reason: &str) -> usize {
        self.events.iter().filter(|e| e.reason == reason).count()
    }
}

#[test]
fn the_reasons_are_known_to_the_wake_store_and_are_not_postable_from_outside() {
    assert!(WAKE_REASONS.contains(&"agent-stale") && WAKE_REASONS.contains(&"agent-checkin"));
    assert!(
        !EXTERNAL_REASONS.contains(&"agent-stale") && !EXTERNAL_REASONS.contains(&"agent-checkin")
    );
}

#[test]
fn high_tier_counts_opus_and_anything_newer_or_unknown_and_the_model_it_really_runs_on_wins() {
    let tier = |value: Value| high_tier(&view(&value));
    assert!(tier(json!({"model": "opus"})));
    assert!(tier(json!({"model": "fable"})));
    assert!(!tier(json!({"model": "sonnet"})));
    assert!(!tier(json!({"model": "haiku"})));
    assert!(tier(
        json!({"model": "sonnet", "modelUsed": "claude-opus-5-5"})
    ));
    assert!(!tier(json!({})));
}

#[test]
fn the_check_in_period_is_the_leads_choice_clamped_to_15_to_30_minutes_default_20() {
    let every = |value: Value| checkin_every(&view(&value), 20 * MIN);
    assert_eq!(every(json!({})), 20 * MIN);
    assert_eq!(every(json!({"checkinMin": 5})), 15 * MIN);
    assert_eq!(every(json!({"checkinMin": 25})), 25 * MIN);
    assert_eq!(every(json!({"checkinMin": 90})), 30 * MIN);
    assert_eq!(
        every(json!({"checkinMin": null})),
        15 * MIN,
        "Number(null) is 0 in Node, clamped to 15"
    );
}

#[test]
fn an_opus_agent_wakes_its_own_thread_after_the_default_20_minutes_then_every_20_and_a_sonnet_agent_never_gets_one()
 {
    let mut world = World::new();
    assert_eq!(world.at(19), 0);
    assert_eq!(world.at(20), 1);
    let first = &world.events[0];
    assert_eq!(
        (
            first.scope.as_str(),
            first.thread.as_str(),
            first.reason.as_str(),
            first.subject.as_deref()
        ),
        ("alpha", "ui", "agent-checkin", Some("A-2"))
    );
    assert!(
        first
            .text
            .contains("A-2 agent A-2 (opus) running 20m: check-in"),
        "{}",
        first.text
    );
    assert!(first.text.ends_with("Task: build the thing"));
    assert_eq!(world.at(39), 0);
    assert_eq!(world.at(40), 1);
    assert!(
        !world
            .events
            .iter()
            .any(|e| e.subject.as_deref() == Some("A-1") && e.reason == "agent-checkin")
    );
}

#[test]
fn the_lead_chooses_the_period_15_and_30_minutes() {
    let mut world = World::new();
    world.set("A-2", "checkinMin", &json!(15));
    world.add("A-3", &json!({"model": "opus", "checkinMin": 30}));
    assert_eq!(world.at(15), 1);
    assert_eq!(world.events[0].subject.as_deref(), Some("A-2"));
    assert_eq!(world.at(29), 0);
    assert_eq!(world.at(30), 2, "A-2 again, A-3 for the first time");
    let mut subjects: Vec<&str> = world.events[1..]
        .iter()
        .filter_map(|e| e.subject.as_deref())
        .collect();
    subjects.sort_unstable();
    assert_eq!(subjects, ["A-2", "A-3"]);
}

#[test]
fn a_quiet_agent_wakes_its_thread_after_20_minutes_without_a_sign_of_life_and_again_after_30_more()
{
    let mut world = World::new();
    world.busy.retain(|id| id != "A-1");
    assert_eq!(world.at(19), 0);
    assert_eq!(
        world.at(21),
        2,
        "A-1 (stale) and A-2 (the opus check-in comes at 20)"
    );
    let stale = world
        .events
        .iter()
        .find(|e| e.reason == "agent-stale")
        .expect("stale");
    assert_eq!(
        (stale.thread.as_str(), stale.subject.as_deref()),
        ("lead", Some("A-1"))
    );
    assert!(
        stale
            .text
            .contains("A-1 agent A-1 (sonnet) shows no sign of life for 21m: check it out"),
        "{}",
        stale.text
    );
    assert_eq!(
        world.at(50),
        1,
        "not again before 30 minutes; only the opus check-in"
    );
    assert_eq!(world.at(51), 1, "the repeat");
}

#[test]
fn a_sign_of_life_from_the_transcript_or_an_update_resets_it() {
    let mut world = World::new();
    world.busy.retain(|id| id != "A-1");
    world.live.insert("A-1".to_owned(), iso(START + 18 * MIN));
    assert_eq!(world.at(25), 1, "only the opus check-in");
    assert_eq!(world.events[0].reason, "agent-checkin");
    world.set("A-1", "updatedAt", &json!(iso(START + 40 * MIN)));
    assert_eq!(world.reasons("agent-stale"), 0);
    assert_eq!(
        world.at(55),
        1,
        "the check-in of A-2 (40 min), A-1 is not quiet"
    );
    assert_eq!(world.reasons("agent-stale"), 0);
}

#[test]
fn a_finished_archived_or_deleted_agent_is_left_alone_and_an_owner_thread_that_ended_falls_back_to_the_lead()
 {
    let mut world = World::new();
    world.set("A-1", "status", &json!("done"));
    world.set("A-2", "archivedAt", &json!(iso(START)));
    assert_eq!(world.at(60), 0);
    world.add("A-4", &json!({"thread": "old"}));
    assert_eq!(world.at(90), 1);
    assert_eq!(world.events[0].thread, "lead");
}

#[test]
fn a_restart_does_not_fire_the_check_ins_of_old_agents_at_once_the_clock_starts_from_the_dashboard_start()
 {
    let mut world = World::new();
    let mut now = START + 5 * HOUR;
    world.wakes = AgentWakes::new(AgentWakeConfig::default(), now);
    world.set("A-2", "updatedAt", &json!(iso(now)));
    assert_eq!(
        world.tick(now),
        1,
        "only A-1: it has been quiet for five hours"
    );
    assert_eq!(world.events[0].reason, "agent-stale");
    now += 19 * MIN;
    world.set("A-1", "updatedAt", &json!(iso(now)));
    world.set("A-2", "updatedAt", &json!(iso(now)));
    assert_eq!(world.tick(now), 0);
    now += 2 * MIN;
    assert_eq!(world.tick(now), 1);
    assert_eq!(
        world.events.last().map(|e| e.reason.as_str()),
        Some("agent-checkin")
    );
}

#[test]
fn a_record_the_reconciler_flagged_gets_neither_wake() {
    let mut world = World::new();
    world.busy.clear();
    world.set("A-1", "orphanSince", &json!(iso(START)));
    world.set("A-2", "idleSince", &json!(iso(START)));
    assert_eq!(world.at(120), 0);
    let mut quiet = AgentWakes::new(AgentWakeConfig::default(), START);
    assert_eq!(
        quiet.tick(START + 2 * HOUR, &world.boards, &NoLive).len(),
        0
    );
}

#[test]
fn the_periods_come_from_the_agents_section_of_the_config() {
    let config = AgentWakeConfig::from_json(
        &json!({"tickMs": 1000, "staleWakeMs": "5000", "staleRepeatMs": -1, "checkinMs": 0}),
    );
    assert_eq!((config.tick_ms, config.stale_wake_ms), (1000, 5000));
    assert_eq!(
        config.stale_repeat_ms,
        AgentWakeConfig::default().stale_repeat_ms
    );
    assert_eq!(config.checkin_ms, AgentWakeConfig::default().checkin_ms);
}
