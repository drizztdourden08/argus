//! The remote sync wake: a project whose lead went 24 hours without `coord remote synced` wakes
//! that lead once, then again every 24 hours while the sync is still due; a new record ends it.
//! A manual clock drives every tick.

use argus_wake::activity::{ActivityLine, Level};
use argus_wake::config::WakeConfig;
use argus_wake::memory::{MemoryBoards, MemorySyncs};
use argus_wake::reasons::{IMMEDIATE_REASONS, WAKE_REASONS};
use argus_wake::sync_wakes::{
    SyncEnv, SyncWakeConfig, SyncWakes, ago_text, due_text, REASON, SYNC_KIND,
};
use argus_wake::testkit::{DAY, HOUR, MIN, Rig, START, board};
use argus_wake::time::iso;
use argus_wake::wake::WakeInput;
use serde_json::{Value, json};

type LastSent = Box<dyn Fn(&str, &str, &str, &str) -> i64>;

fn never() -> LastSent {
    Box::new(|_, _, _, _| 0)
}

fn project(lead_status: &str) -> Value {
    json!({
        "threads": {
            "lead": {
                "id": "lead", "role": "coordinator", "status": lead_status,
                "registeredAt": iso(START),
            },
            "worker": {"id": "worker", "role": "thread", "status": "active"},
        },
    })
}

struct World {
    boards: MemoryBoards,
    syncs: MemorySyncs,
    wakes: SyncWakes,
    last_sent: LastSent,
    events: Vec<WakeInput>,
    lines: Vec<ActivityLine>,
}

impl World {
    fn new() -> Self {
        let world = Self {
            boards: MemoryBoards::new(),
            syncs: MemorySyncs::new(),
            wakes: SyncWakes::new(SyncWakeConfig::default(), START),
            last_sent: never(),
            events: Vec::new(),
            lines: Vec::new(),
        };
        world.boards.set("alpha", board(&project("active")));
        world
    }

    fn sync_at(&self, hours: i64) {
        self.syncs
            .set("alpha", &iso(START + hours * HOUR), "alpha", "pushed 2 PRs");
    }

    fn tick_at(&mut self, now: i64) -> usize {
        let env = SyncEnv {
            boards: &self.boards,
            syncs: &self.syncs,
            last_sent: &*self.last_sent,
        };
        let out = self.wakes.tick(now, &env);
        let sent = out.wakes.len();
        self.events.extend(out.wakes);
        self.lines.extend(out.activity);
        sent
    }

    fn at(&mut self, hours: i64) -> usize {
        self.tick_at(START + hours * HOUR)
    }

    fn texts(&self) -> Vec<&str> {
        self.events.iter().map(|e| e.text.as_str()).collect()
    }

    fn lines_of(&self, kind: &str) -> Vec<&str> {
        let of_kind = |l: &&ActivityLine| l.kind == kind;
        self.lines
            .iter()
            .filter(of_kind)
            .map(|l| l.text.as_str())
            .collect()
    }
}

#[test]
fn the_reason_is_known_to_the_wake_store_and_does_not_close_the_coalescing_window() {
    assert!(WAKE_REASONS.contains(&REASON));
    assert!(!IMMEDIATE_REASONS.contains(&REASON));
}

#[test]
fn the_words_are_the_owners_text_with_a_short_age_or_never() {
    assert_eq!(
        due_text(&ago_text(None)),
        "remote sync due: last never, run the full sync, then coord remote synced"
    );
    assert_eq!(ago_text(Some(25 * HOUR)), "25h ago");
    assert_eq!(ago_text(Some(3 * DAY)), "3d ago");
    assert_eq!(ago_text(Some(40 * MIN)), "40m ago");
}

#[test]
fn a_project_that_never_synced_wakes_its_lead_at_24_h_once_and_again_at_48_h() {
    let mut world = World::new();
    assert_eq!(world.at(0), 0);
    assert_eq!(world.at(19), 0, "19 h is quiet");
    assert_eq!(world.at(20), 0, "20 h is only amber on the page");
    assert_eq!(world.tick_at(START + 24 * HOUR - MIN), 0);
    assert_eq!(world.at(24), 1, "24 h: due");
    assert_eq!(world.at(25), 0, "once");
    assert_eq!(world.at(47), 0);
    assert_eq!(world.at(48), 1, "48 h: due again");
    assert_eq!(world.at(49), 0);
    assert_eq!(world.at(72), 1, "and every 24 h after");
    let first = &world.events[0];
    assert_eq!(
        (first.scope.as_str(), first.thread.as_str(), first.reason.as_str()),
        ("alpha", "lead", "remote-sync-due")
    );
    assert_eq!(first.subject.as_deref(), Some("alpha"));
    assert_eq!(
        first.text,
        "remote sync due: last never, run the full sync, then coord remote synced"
    );
}

#[test]
fn a_synced_project_counts_from_its_last_record_and_the_text_says_how_long_ago() {
    let mut world = World::new();
    world.sync_at(0);
    assert_eq!(world.at(19), 0);
    assert_eq!(world.at(20), 0);
    assert_eq!(world.at(23), 0);
    assert_eq!(world.at(24), 1);
    assert_eq!(world.at(48), 1);
    assert_eq!(
        world.texts(),
        [
            "remote sync due: last 24h ago, run the full sync, then coord remote synced",
            "remote sync due: last 2d ago, run the full sync, then coord remote synced",
        ]
    );
}

#[test]
fn a_lead_that_registered_long_ago_and_never_synced_is_due_at_once() {
    let world_board = json!({
        "threads": {"lead": {
            "id": "lead", "role": "coordinator", "status": "active",
            "registeredAt": iso(START - 3 * DAY),
        }},
    });
    let mut world = World::new();
    world.boards.set("alpha", board(&world_board));
    assert_eq!(world.at(0), 1);
}

#[test]
fn a_new_record_clears_the_due_state_and_the_count_starts_again() {
    let mut world = World::new();
    assert_eq!(world.at(24), 1);
    world.sync_at(30);
    assert_eq!(world.at(30), 0);
    assert_eq!(world.at(48), 0, "18 h after the record");
    assert_eq!(world.at(53), 0);
    assert_eq!(world.at(54), 1, "24 h after the record: due again");
    assert_eq!(
        world.texts()[1],
        "remote sync due: last 24h ago, run the full sync, then coord remote synced"
    );
}

#[test]
fn an_archived_or_done_lead_and_a_board_with_no_lead_are_left_alone() {
    for status in ["archived", "done"] {
        let mut world = World::new();
        world.boards.set("alpha", board(&project(status)));
        assert_eq!(world.at(48), 0, "{status}");
    }
    let mut world = World::new();
    let no_lead = json!({"threads": {"worker": {"id": "worker", "role": "thread", "status": "active"}}});
    world.boards.set("alpha", board(&no_lead));
    assert_eq!(world.at(48), 0);
    let placeholder = json!({"threads": {"lead": {
        "id": "lead", "role": "coordinator", "status": "active", "placeholder": true,
    }}});
    world.boards.set("alpha", board(&placeholder));
    assert_eq!(world.at(48), 0, "a placeholder is not a registered session");
}

#[test]
fn a_lead_that_ended_gives_way_to_the_next_coordinator() {
    let mut world = World::new();
    let two = json!({"threads": {
        "old": {"id": "old", "role": "coordinator", "status": "archived"},
        "lead": {"id": "lead", "role": "coordinator", "status": "active", "registeredAt": iso(START)},
    }});
    world.boards.set("alpha", board(&two));
    assert_eq!(world.at(24), 1);
    assert_eq!(world.events[0].thread, "lead");
}

#[test]
fn the_overlords_scope_and_the_portfolio_board_are_not_projects_to_sync() {
    let mut world = World::new();
    world.boards.set("portfolio", board(&project("active")));
    world.boards.set("_portfolio", board(&project("active")));
    assert_eq!(world.at(24), 1, "only alpha");
    assert!(world.events.iter().all(|e| e.scope == "alpha"));
}

#[test]
fn each_project_is_woken_for_itself() {
    let mut world = World::new();
    world.boards.set("beta", board(&project("active")));
    world.syncs.set("beta", &iso(START + 12 * HOUR), "beta", "");
    assert_eq!(world.at(24), 1, "beta synced 12 h ago");
    assert_eq!(world.events[0].scope, "alpha");
    assert_eq!(world.at(36), 1, "beta is due now");
    assert_eq!(world.events[1].scope, "beta");
}

#[test]
fn a_restart_does_not_send_the_wake_again_at_once() {
    let mut world = World::new();
    assert_eq!(world.at(24), 1);
    let sent_at = START + 24 * HOUR;
    world.wakes = SyncWakes::new(SyncWakeConfig::default(), START + 26 * HOUR);
    world.last_sent = Box::new(move |scope, thread, reason, subject| {
        assert_eq!((scope, thread, reason, subject), ("alpha", "lead", REASON, "alpha"));
        sent_at
    });
    assert_eq!(world.at(27), 0, "the wakes table remembers the last one");
    assert_eq!(world.at(48), 1);
}

#[test]
fn a_new_record_leaves_one_activity_line_and_a_due_wake_one_more() {
    let mut world = World::new();
    assert_eq!(world.at(1), 0);
    assert!(world.lines.is_empty());
    world.sync_at(2);
    world.at(2);
    world.at(3);
    assert_eq!(
        world.lines_of(SYNC_KIND),
        ["alpha: remote sync recorded by alpha: pushed 2 PRs"],
        "one line, not one per tick"
    );
    assert_eq!(world.at(26), 1);
    assert_eq!(
        world.lines_of(REASON),
        ["alpha/lead: remote sync due, last 24h ago"]
    );
    let due = world.lines.last().expect("a line");
    assert_eq!(
        (due.level, due.project.as_deref(), due.thread.as_deref()),
        (Level::Info, Some("alpha"), Some("lead"))
    );
    world.sync_at(30);
    world.at(30);
    assert_eq!(world.lines_of(SYNC_KIND).len(), 2, "a second record, a second line");
}

#[test]
fn a_record_older_than_the_start_is_not_news() {
    let mut world = World::new();
    world.syncs.set("alpha", &iso(START - 2 * HOUR), "alpha", "before the start");
    assert_eq!(world.at(1), 0);
    assert!(world.lines.is_empty(), "{:?}", world.lines);
}

#[test]
fn a_record_without_a_note_or_a_setter_still_gets_a_line() {
    let mut world = World::new();
    world.syncs.set("alpha", &iso(START + HOUR), "", "");
    world.at(2);
    assert_eq!(world.lines_of(SYNC_KIND), ["alpha: remote sync recorded"]);
}

#[test]
fn the_timing_is_read_from_the_remote_sync_section_of_the_config() {
    let set = WakeConfig::from_json(&json!({
        "remoteSync": {"tickMs": 1000, "dueMs": 3_600_000, "repeatMs": "7200000"},
    }));
    assert_eq!(
        (set.syncs.tick_ms, set.syncs.due_ms, set.syncs.repeat_ms),
        (1000, HOUR, 2 * HOUR)
    );
    let bad = WakeConfig::from_json(&json!({"remoteSync": {"dueMs": -5, "repeatMs": "soon"}}));
    assert_eq!(bad.syncs, SyncWakeConfig::default());
    assert_eq!(bad.syncs.due_ms, DAY);
    let mut world = World::new();
    world.wakes = SyncWakes::new(set.syncs, START);
    assert_eq!(world.at(1), 1, "due after an hour");
    assert_eq!(world.at(2), 0);
    assert_eq!(world.at(3), 1, "repeat after two hours");
}

#[test]
fn the_service_ticks_the_wake_logs_the_lines_and_remembers_what_it_sent() {
    let mut rig = Rig::new(&WakeConfig::default(), false);
    rig.boards.set("alpha", board(&project("active")));
    rig.service.start();
    rig.clock.advance(23 * HOUR);
    assert_eq!(rig.service.sync_tick(), 0);
    rig.clock.advance(HOUR);
    assert_eq!(rig.service.sync_tick(), 1);
    assert_eq!(rig.service.sync_tick(), 0, "once");
    let rows = rig.store.all();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        (rows[0].reason.as_str(), rows[0].thread.as_str()),
        (REASON, "lead")
    );
    assert_eq!(
        rows[0].text,
        "remote sync due: last never, run the full sync, then coord remote synced"
    );
    let lines = rig.activity.lines();
    assert!(
        lines
            .iter()
            .any(|l| l.kind == REASON && l.text == "alpha/lead: remote sync due, last never")
    );
    rig.syncs.set("alpha", &iso(START + 25 * HOUR), "lead", "pushed");
    rig.clock.advance(HOUR);
    assert_eq!(rig.service.sync_tick(), 0);
    assert!(
        rig.activity
            .lines()
            .iter()
            .any(|l| l.kind == SYNC_KIND && l.text == "alpha: remote sync recorded by lead: pushed")
    );
}

#[test]
fn the_periodic_tick_runs_the_sync_wake_on_its_own_schedule() {
    let mut rig = Rig::new(&WakeConfig::default(), false);
    rig.boards.set("alpha", board(&project("active")));
    rig.service.start();
    rig.clock.advance(DAY);
    rig.service.tick();
    let due: Vec<_> = rig
        .store
        .all()
        .into_iter()
        .filter(|r| r.reason == REASON)
        .collect();
    assert_eq!(due.len(), 1);
}
