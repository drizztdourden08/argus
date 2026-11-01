//! KEEP: K1.
//! The wake policy: the one table that says which reasons wake a thread and which only leave a
//! notice, a passive reason never reaching a listener (live or replayed), the 5 minute debounce
//! per thread and the 20 minute digest on a fake clock, the `wakePolicy` settings, and the notice
//! digest the prompt hook prints.

// test-tag: critical

use argus_wake::config::WakeConfig;
use argus_wake::engine::{AttachOptions, EngineConfig};
use argus_wake::notices::{digest, digest_line};
use argus_wake::reasons::WAKE_REASONS;
use argus_wake::testkit::{EngineRig, MIN, SEC, START, wake, wake_in};
use argus_wake::time::iso;
use argus_wake::wake::{WakeRow, WakeStatus};
use argus_wake::wake_policy::{
    DEFAULT_DEBOUNCE_MS, DEFAULT_DIGEST_MS, WAKE_CLASSES, WakeClass, WakePolicy, class_of,
    is_passive,
};
use serde_json::json;

const WINDOW: i64 = 15 * SEC;

const PASSIVE: [&str; 11] = [
    "lead-info",
    "wake-fault",
    "agent-stale",
    "agent-checkin",
    "kanban-stale",
    "process-alert",
    "worktree-stale",
    "worktree-review",
    "launcher-stale",
    "launcher-master-missing",
    "remote-sync-due",
];

fn policy_rig(immediate: &[&str]) -> EngineRig {
    EngineRig::with(EngineConfig {
        window_ms: WINDOW,
        immediate: immediate.iter().map(|r| (*r).to_owned()).collect(),
        policy: WakePolicy::default(),
        ..EngineConfig::default()
    })
}

#[test]
fn every_reason_has_exactly_one_class_in_the_table_and_the_table_lists_no_other() {
    for reason in WAKE_REASONS {
        let rows = WAKE_CLASSES.iter().filter(|(r, _)| *r == reason).count();
        assert_eq!(rows, 1, "{reason}");
    }
    assert_eq!(WAKE_CLASSES.len(), WAKE_REASONS.len());
}

#[test]
fn the_table_wakes_for_owner_work_messages_agents_results_and_own_faults_and_keeps_the_rest_passive()
 {
    for (reason, class) in WAKE_CLASSES {
        let expected = if PASSIVE.contains(&reason) {
            WakeClass::Passive
        } else {
            WakeClass::Wake
        };
        assert_eq!(class, expected, "{reason}");
        assert_eq!(class_of(reason), expected, "{reason}");
    }
    for reason in [
        "owner-answer",
        "owner-note",
        "owner-decision",
        "item-assigned",
        "resume",
        "portfolio-decision",
        "portfolio-note",
        "lead-message",
        "thread-message",
        "agent-idle",
        "agent-orphan",
        "test-fail",
        "test-partial",
        "test-blocked",
        "bundle-complete",
        "run-failed",
        "process-reaped",
    ] {
        assert!(!is_passive(reason), "{reason} wakes");
    }
    assert_eq!(class_of("a-reason-nobody-knows"), WakeClass::Wake);
}

#[test]
fn a_passive_reason_is_recorded_and_mirrored_but_never_reaches_a_listener_live_or_replayed() {
    let mut rig = policy_rig(&[]);
    let mut ui = rig.listen("alpha", "ui", AttachOptions::default());
    let mut star = rig.listen("alpha", "*", AttachOptions::default());
    for reason in PASSIVE {
        rig.submit(&[wake_in("alpha", "ui", &format!("{reason} notice"), reason)]);
    }
    rig.advance(30 * MIN);
    assert_eq!(ui.count(), 0);
    assert_eq!(star.count(), 0);
    assert_eq!(rig.engine.pending(), 0, "no window was opened");
    let rows = rig.store.all();
    assert_eq!(rows.len(), PASSIVE.len());
    assert!(rows.iter().all(|r| r.status == WakeStatus::Passive));
    assert!(rows.iter().all(|r| r.listeners == 0));
    let mirrored = rig.mirrored("alpha");
    assert!(mirrored.values().all(|d| d["status"] == "passive"));
    let mut late = rig.listen("alpha", "ui", AttachOptions::default());
    assert_eq!(late.count(), 0, "a notice is never replayed");
    assert_eq!(rig.engine.queued_since("alpha", &iso(0)).len(), 0);
}

#[test]
fn a_passive_reason_next_to_a_wake_leaves_the_wake_alone() {
    let mut rig = policy_rig(&[]);
    let mut ui = rig.listen("alpha", "ui", AttachOptions::default());
    rig.submit(&[
        wake_in("alpha", "ui", "I-4 is stale", "kanban-stale"),
        wake("ui", "T-1 failed: x"),
    ]);
    rig.advance(WINDOW);
    assert_eq!(ui.lines(), [["T-1 failed: x"]]);
}

#[test]
fn a_thread_is_woken_at_most_once_per_5_minutes_and_what_comes_inside_goes_in_one_digest_20_minutes_after()
 {
    let mut rig = policy_rig(&[]);
    let mut ui = rig.listen("alpha", "ui", AttachOptions::default());
    rig.submit(&[wake("ui", "first")]);
    rig.advance(WINDOW);
    assert_eq!(ui.count(), 1, "the first wake goes when its window closes");
    let woken = START + WINDOW;
    rig.advance(MIN);
    rig.submit(&[wake("ui", "second")]);
    rig.advance(5 * MIN);
    rig.submit(&[wake("ui", "third")]);
    rig.clock.set(woken + DEFAULT_DIGEST_MS - 1);
    rig.engine.tick();
    assert_eq!(ui.count(), 1, "held until the digest is due");
    rig.advance(1);
    assert_eq!(ui.count(), 2);
    let held = ui.events()[1].clone();
    assert_eq!(held.count, 2);
    assert_eq!(held.lines, ["second", "third"]);
}

#[test]
fn an_owner_reason_inside_the_debounce_goes_when_the_5_minutes_end_not_with_the_digest() {
    let mut rig = policy_rig(&["owner-answer"]);
    let mut ui = rig.listen("alpha", "ui", AttachOptions::default());
    rig.submit(&[wake("ui", "T-1 failed: x")]);
    rig.advance(WINDOW);
    let woken = START + WINDOW;
    rig.advance(MIN);
    rig.submit(&[wake_in("alpha", "ui", "owner answered Q-3", "owner-answer")]);
    assert_eq!(
        ui.count(),
        1,
        "not at once: the thread was woken a minute ago"
    );
    rig.clock.set(woken + DEFAULT_DEBOUNCE_MS - 1);
    rig.engine.tick();
    assert_eq!(ui.count(), 1);
    rig.advance(1);
    assert_eq!(ui.lines()[1], ["owner answered Q-3"]);
}

#[test]
fn after_the_debounce_a_wake_goes_on_the_usual_window_and_the_debounce_is_per_thread() {
    let mut rig = policy_rig(&[]);
    let mut ui = rig.listen("alpha", "ui", AttachOptions::default());
    let mut api = rig.listen("alpha", "api", AttachOptions::default());
    rig.submit(&[wake("ui", "first")]);
    rig.advance(WINDOW);
    rig.submit(&[wake("api", "other thread")]);
    rig.advance(WINDOW);
    assert_eq!(api.count(), 1, "another thread is not held");
    rig.advance(6 * MIN);
    rig.submit(&[wake("ui", "later")]);
    rig.advance(WINDOW);
    assert_eq!(ui.lines(), [["first"], ["later"]]);
}

#[test]
fn the_policy_comes_from_the_wake_policy_section_and_0_turns_the_debounce_off() {
    let defaults = WakeConfig::default().policy;
    assert_eq!(
        (defaults.debounce_ms, defaults.digest_ms),
        (5 * MIN, 20 * MIN)
    );
    let set =
        WakeConfig::from_json(&json!({"wakePolicy": {"debounceMs": 60_000, "digestMs": "900000"}}));
    assert_eq!(
        (set.policy.debounce_ms, set.policy.digest_ms),
        (MIN, 15 * MIN)
    );
    let bad = WakeConfig::from_json(&json!({"wakePolicy": {"debounceMs": -1, "digestMs": "soon"}}));
    assert_eq!(bad.policy, WakePolicy::default());
    let off = WakePolicy::from_json(&json!({"debounceMs": 0}));
    assert_eq!(off.held_until(Some(START), START + 1, false), None);
    assert_eq!(WakePolicy::off().held_until(Some(START), START, true), None);
    let on = WakePolicy::default();
    assert_eq!(on.held_until(None, START, false), None, "never woken: free");
    assert_eq!(
        on.held_until(Some(START), START + MIN, false),
        Some(START + 20 * MIN)
    );
    assert_eq!(
        on.held_until(Some(START), START + MIN, true),
        Some(START + 5 * MIN)
    );
    assert_eq!(on.held_until(Some(START), START + 5 * MIN, false), None);
}

fn row(n: u64, reason: &str, text: &str, at_ms: i64, status: WakeStatus) -> WakeRow {
    let input = wake_in("alpha", "lead", text, reason);
    let mut row = WakeRow::queued(n, &input, &iso(at_ms));
    row.status = status;
    row
}

/// Five notices and one delivered wake after the last turn at `START`, one notice before it.
fn digest_rows() -> Vec<WakeRow> {
    let passive = WakeStatus::Passive;
    [
        (
            6,
            "worktree-review",
            "3 worktrees need a decision",
            6,
            passive,
        ),
        (5, "test-fail", "T-1 failed", 5, WakeStatus::Delivered),
        (4, "kanban-stale", "I-4 sits in doing", 4, passive),
        (3, "kanban-stale", "I-5 sits in todo", 3, passive),
        (2, "process-alert", "vite stale 3h", 2, passive),
        (1, "kanban-stale", "before the last turn", -1, passive),
    ]
    .into_iter()
    .map(|(n, reason, text, minutes, status)| row(n, reason, text, START + minutes * MIN, status))
    .collect()
}

#[test]
fn the_digest_counts_the_notices_since_the_last_turn_by_kind_and_names_the_newest_three() {
    let rows = digest_rows();
    let got = digest(&rows, START);
    assert_eq!(got.count, 4);
    let kinds: Vec<(&str, u32)> = got
        .kinds
        .iter()
        .map(|k| (k.reason.as_str(), k.count))
        .collect();
    assert_eq!(
        kinds,
        [
            ("kanban-stale", 2),
            ("process-alert", 1),
            ("worktree-review", 1)
        ]
    );
    let newest: Vec<&str> = got.newest.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(newest, ["W-6", "W-4", "W-3"]);
    assert_eq!(
        got.line.as_deref(),
        Some(
            "Board notices since your last turn: 4 (kanban-stale 2, process-alert 1, worktree-review 1). Newest: \"3 worktrees need a decision\"; \"I-4 sits in doing\"; \"I-5 sits in todo\". Not wakes: the board header lists them."
        )
    );
    assert_eq!(digest(&rows, START + 10 * MIN).line, None);
    assert_eq!(digest_line(0, &[], &[]), None);
}
