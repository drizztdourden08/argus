// test-tag: critical
use super::*;

const T0: i64 = 1_000_000;

fn key(project: &str, thread: &str) -> ThreadKey {
    ThreadKey::new(project, thread)
}

fn target(project: &str, thread: &str) -> RelayTarget {
    RelayTarget {
        ward: Ward {
            key: key(project, thread),
            session_id: Some(format!("cli-{thread}")),
        },
        since_ms: T0,
    }
}

fn live(project: &str, thread: &str, in_turn: bool, main: bool) -> RelayLive {
    RelayLive {
        key: key(project, thread),
        in_turn,
        main,
    }
}

fn relay() -> Relay {
    Relay::new(RelaySettings {
        enabled: true,
        headless: false,
    })
}

fn relay_headless() -> Relay {
    Relay::new(RelaySettings::default())
}

fn via_of(actions: &[Action]) -> Vec<ThreadKey> {
    actions
        .iter()
        .filter_map(|a| match a {
            Action::Note { via, .. } => Some(via.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn nothing_happens_before_the_first_delay() {
    let mut relay = relay();
    let out = relay.step(
        T0 + FIRST_AFTER_MS - 1,
        &[target("a", "ui")],
        &[live("a", "lead", false, false)],
    );
    assert_eq!(out.len(), 0);
}

#[test]
fn the_overlord_is_preferred_when_idle() {
    let mut relay = relay();
    let all = [
        live("a", "lead", false, false),
        live("portfolio", "coordinator", false, true),
    ];
    let out = relay.step(T0 + FIRST_AFTER_MS, &[target("a", "ui")], &all);
    assert_eq!(via_of(&out), [key("portfolio", "coordinator")]);
    let last = relay.last("a", "ui").expect("last");
    assert_eq!(last.kind, AttemptKind::Note);
    assert_eq!(last.via.as_deref(), Some("coordinator"));
}

#[test]
fn a_thread_in_a_turn_is_chosen_only_when_no_idle_one_exists() {
    let busy_main = live("portfolio", "coordinator", true, true);
    let idle = live("a", "lead", false, false);
    let mut relay = relay();
    let out = relay.step(
        T0 + FIRST_AFTER_MS,
        &[target("a", "ui")],
        &[busy_main.clone(), idle],
    );
    assert_eq!(via_of(&out), [key("a", "lead")]);
    let mut relay = self::relay();
    let out = relay.step(T0 + FIRST_AFTER_MS, &[target("a", "ui")], &[busy_main]);
    assert_eq!(via_of(&out), [key("portfolio", "coordinator")]);
}

#[test]
fn the_target_is_never_its_own_relay() {
    let mut relay = relay();
    let only = [live("a", "ui", false, false)];
    let out = relay.step(T0 + FIRST_AFTER_MS, &[target("a", "ui")], &only);
    assert_eq!(via_of(&out).len(), 0);
}

#[test]
fn the_second_note_goes_to_a_different_relay() {
    let mut relay = relay();
    let all = [
        live("portfolio", "coordinator", false, true),
        live("a", "lead", false, false),
    ];
    let now = T0 + FIRST_AFTER_MS;
    let first = relay.step(now, &[target("a", "ui")], &all);
    assert_eq!(via_of(&first), [key("portfolio", "coordinator")]);
    assert_eq!(
        relay
            .step(now + AGAIN_AFTER_MS - 1, &[target("a", "ui")], &all)
            .len(),
        0
    );
    let second = relay.step(now + AGAIN_AFTER_MS, &[target("a", "ui")], &all);
    assert_eq!(via_of(&second), [key("a", "lead")]);
}

#[test]
fn headless_comes_first_when_on() {
    let mut relay = relay_headless();
    let all = [live("a", "lead", false, false)];
    let due = [target("a", "ui")];
    let now = T0 + FIRST_AFTER_MS;
    let out = relay.step(now, &due, &all);
    assert_eq!(out, [Action::Headless(due[0].ward.clone())]);
    assert_eq!(
        relay.last("a", "ui").expect("last").kind,
        AttemptKind::Headless
    );
    assert_eq!(relay.step(now + AGAIN_AFTER_MS - 1, &due, &all).len(), 0);
    let next = relay.step(now + AGAIN_AFTER_MS, &due, &all);
    assert_eq!(via_of(&next), [key("a", "lead")]);
    assert_eq!(relay.last("a", "ui").expect("last").kind, AttemptKind::Note);
}

#[test]
fn headless_then_two_notes_then_the_notice() {
    let mut relay = relay_headless();
    let all = [live("a", "lead", false, false)];
    let due = [target("a", "ui")];
    let now = T0 + FIRST_AFTER_MS;
    assert_eq!(relay.step(now, &due, &all).len(), 1);
    assert_eq!(
        via_of(&relay.step(now + AGAIN_AFTER_MS, &due, &all)).len(),
        1
    );
    assert_eq!(
        via_of(&relay.step(now + 2 * AGAIN_AFTER_MS, &due, &all)).len(),
        1
    );
    let spent = now + 3 * AGAIN_AFTER_MS;
    assert_eq!(relay.step(spent, &due, &all).len(), 0);
    let out = relay.step(spent + NOTICE_AFTER_MS, &due, &all);
    assert_eq!(out, [Action::Notice(due[0].ward.clone())]);
}

#[test]
fn headless_off_skips_the_resume_and_the_notice_still_comes() {
    let mut relay = Relay::new(RelaySettings {
        enabled: true,
        headless: false,
    });
    let due = [target("a", "ui")];
    let now = T0 + FIRST_AFTER_MS;
    assert_eq!(relay.step(now, &due, &[]).len(), 0);
    let later = relay.step(now + NOTICE_AFTER_MS, &due, &[]);
    assert_eq!(later, [Action::Notice(due[0].ward.clone())]);
}

#[test]
fn the_ten_minute_notice_happens_once() {
    let mut relay = relay();
    let due = [target("a", "ui")];
    let now = T0 + FIRST_AFTER_MS;
    assert_eq!(relay.step(now, &due, &[]).len(), 0);
    assert_eq!(relay.step(now + NOTICE_AFTER_MS - 1, &due, &[]).len(), 0);
    let at = now + NOTICE_AFTER_MS;
    assert_eq!(
        relay.step(at, &due, &[]),
        [Action::Notice(due[0].ward.clone())]
    );
    assert_eq!(relay.step(at + 60_000, &due, &[]).len(), 0);
    assert_eq!(relay.step(at + QUIET_MS - 1, &due, &[]).len(), 0);
    assert_eq!(
        relay.last("a", "ui").expect("last").kind,
        AttemptKind::Notified
    );
}

#[test]
fn a_target_no_longer_due_is_forgotten() {
    let mut relay = relay_headless();
    let due = [target("a", "ui")];
    relay.step(T0 + FIRST_AFTER_MS, &due, &[]);
    assert!(relay.last("a", "ui").is_some());
    relay.step(T0 + FIRST_AFTER_MS + 1, &[], &[]);
    assert_eq!(relay.last("a", "ui"), None);
}

#[test]
fn several_targets_for_one_relay_fold_into_one_note() {
    let mut relay = relay();
    let due = [target("a", "ui"), target("b", "api")];
    let all = [live("portfolio", "coordinator", false, true)];
    let out = relay.step(T0 + FIRST_AFTER_MS, &due, &all);
    assert_eq!(out.len(), 1);
    let Action::Note { via, targets } = &out[0] else {
        panic!("a note");
    };
    assert_eq!(via, &key("portfolio", "coordinator"));
    assert_eq!(targets.len(), 2);
}

#[test]
fn a_disabled_relay_does_nothing() {
    let mut relay = Relay::new(RelaySettings {
        enabled: false,
        headless: true,
    });
    let out = relay.step(T0 + FIRST_AFTER_MS, &[target("a", "ui")], &[]);
    assert_eq!(out.len(), 0);
}

#[test]
fn the_settings_default_to_on_and_read_booleans() {
    let on = RelaySettings::from_json(&serde_json::json!({}));
    assert_eq!(on, RelaySettings::default());
    let off = RelaySettings::from_json(&serde_json::json!({"enabled": false, "headless": false}));
    assert_eq!(
        off,
        RelaySettings {
            enabled: false,
            headless: false
        }
    );
}
