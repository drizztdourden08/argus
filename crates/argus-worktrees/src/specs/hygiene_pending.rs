//! KEEP: K3, K4.
//! A thread's keep is a pending keep (I-179): it holds the worktree like an exception for at most 24 hours, the
//! Overlord gets one note and a wake with the two answers, his keep confirms it, his clear declines it, and a lapse
//! drops it. The page is still refused.

// test-tag: standard

use crate::board::Caller;
use crate::hygiene::{HygieneState, KeepInput};
use crate::requests::{RequestInput, RequestStatus};
use crate::specs::hygiene_kit::{
    DEBOUNCE, HOUR, add_overlord, kind, notes_of, state, step, through_debounce, wakes, world,
};
use crate::testkit::{Fixture, Harness, exists_itself};

fn keep_by(h: &Harness, path: &str, by: &str, from: &str) -> KeepInput {
    KeepInput {
        project: "alpha".into(),
        caller: Caller::Thread(by.into()),
        by_project: Some(from.into()),
        id: h.id_of(path),
        note: Some("the owner tests it tonight".into()),
        until_ms: None,
        asked_by: None,
        clear: false,
    }
}

/// The lead's ask, as its own board sends it.
fn ask(h: &Harness, path: &str) -> KeepInput {
    keep_by(h, path, "lead", "alpha")
}

/// The Overlord's answer, from the `portfolio` board.
fn answer(h: &Harness, path: &str) -> KeepInput {
    keep_by(h, path, "overlord", "portfolio")
}

/// A world where the lead has asked to keep `agent-a1` and the Overlord is on the board.
fn asked() -> (Fixture, Harness, String) {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    h.svc.keep(&ask(&h, &path)).unwrap();
    (fx, h, path)
}

fn is_pending(h: &Harness) -> bool {
    h.docs()["agent-a1"].hygiene.clone().unwrap().pending
}

#[test]
fn a_threads_keep_files_a_pending_keep_and_the_overlord_gets_one_note_and_one_wake() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    let filed = h.svc.keep(&ask(&h, &path)).unwrap();
    assert_eq!(
        filed.text,
        format!(
            "pending keep filed: the hygiene holds {path} until the Overlord answers (at most 24 h)"
        )
    );
    h.svc.hygiene_step("alpha");
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
    assert!(is_pending(&h));
    let id = h.id_of(&path);
    let notes = notes_of(&h, "overlord");
    assert_eq!(notes.len(), 1);
    assert!(notes[0].contains("lead of alpha"), "{}", notes[0]);
    assert!(
        notes[0].contains("the owner tests it tonight"),
        "{}",
        notes[0]
    );
    assert!(
        notes[0].contains(&format!(
            "coord worktree keep {id} --project alpha --asked-by lead --note"
        )),
        "{}",
        notes[0]
    );
    assert!(
        notes[0].contains(&format!("coord worktree keep {id} --project alpha --clear")),
        "{}",
        notes[0]
    );
    assert_eq!(wakes(&h, "worktree-report").len(), 1);
    h.svc.keep(&ask(&h, &path)).unwrap();
    assert_eq!(
        notes_of(&h, "overlord").len(),
        1,
        "a second ask is no duplicate"
    );
}

#[test]
fn a_pending_keep_stops_the_hygiene_removing_a_clean_unattached_worktree() {
    let (fx, h, path) = asked();
    step(&h, DEBOUNCE);
    step(&h, DEBOUNCE);
    h.svc.requests_idle();
    assert!(exists_itself(&path));
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
}

#[test]
fn the_overlords_keep_confirms_it_and_tells_the_thread() {
    let (fx, h, path) = asked();
    h.svc.keep(&answer(&h, &path)).unwrap();
    h.svc.hygiene_step("alpha");
    assert!(!is_pending(&h));
    step(&h, 48 * HOUR);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
    assert!(exists_itself(&path));
    let told = notes_of(&h, "lead");
    assert!(told.iter().any(|t| t.contains("confirmed")), "{told:?}");
}

#[test]
fn the_overlords_clear_declines_it_and_the_hygiene_judges_the_worktree_again() {
    let (fx, h, path) = asked();
    let mut clear = answer(&h, &path);
    clear.clear = true;
    h.svc.keep(&clear).unwrap();
    step(&h, 1);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert!(through_debounce(&h, &path));
    let told = notes_of(&h, "lead");
    assert!(told.iter().any(|t| t.contains("declined")), "{told:?}");
}

#[test]
fn a_pending_keep_lapses_after_24_hours_and_the_thread_is_told() {
    let (fx, h, path) = asked();
    step(&h, 24 * HOUR - 1);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
    assert_eq!(notes_of(&h, "lead").len(), 0);
    step(&h, 1);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
    let told = notes_of(&h, "lead");
    assert!(told.iter().any(|t| t.contains("lapsed")), "{told:?}");
    step(&h, HOUR);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
}

/// The Overlord's keep of agent 1's worktree that ends in an hour.
fn timed_keep(h: &Harness, path: &str) -> KeepInput {
    let mut keep = answer(h, path);
    keep.until_ms = Some(h.clock_now() + HOUR);
    keep.asked_by = Some("lead".into());
    keep
}

#[test]
fn an_ended_keep_holds_one_more_hour_and_its_asker_is_told_once() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    h.svc.keep(&timed_keep(&h, &path)).unwrap();
    step(&h, HOUR - 1);
    assert_eq!(notes_of(&h, "lead").len(), 0);
    step(&h, 1);
    let (kept, why) = state(&h, "agent-a1").unwrap();
    assert_eq!(kept, HygieneState::Exception);
    assert!(why.unwrap_or_default().contains("the keep ended"));
    step(&h, HOUR / 2);
    let told = notes_of(&h, "lead");
    assert_eq!(told.len(), 1, "{told:?}");
    assert!(told[0].contains("ended") && told[0].contains(&path));
    step(&h, HOUR / 2);
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Removing));
    assert_eq!(notes_of(&h, "lead").len(), 1);
}

#[test]
fn a_removal_request_for_a_kept_worktree_is_refused_until_the_keep_is_cleared() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    h.svc.keep(&answer(&h, &path)).unwrap();
    let mut input = RequestInput::new("alpha", Caller::Thread("lead".into()));
    input.targets = vec![path.clone()];
    h.svc.request(&input).unwrap();
    h.svc.requests_idle();
    let list = h.svc.request_list("alpha");
    assert_eq!(list[0].status, RequestStatus::Refused);
    let said = &list[0].results[0].text;
    assert!(said.contains("on the keep list"), "{said}");
    assert!(said.contains("--clear"), "{said}");
    assert!(exists_itself(&path));
}

#[test]
fn the_page_is_refused_and_a_thread_clears_only_its_own_pending_keep() {
    let (fx, h) = world(|_| {});
    let path = fx.path("agent-a1");
    add_overlord(&h);
    let mut page = ask(&h, &path);
    page.caller = Caller::Page;
    assert!(h.svc.keep(&page).is_err());
    assert_eq!(notes_of(&h, "overlord").len(), 0);
    h.svc.keep(&ask(&h, &path)).unwrap();
    let mut other = keep_by(&h, &path, "other", "beta");
    other.clear = true;
    assert!(h.svc.keep(&other).is_err());
    let mut own = ask(&h, &path);
    own.clear = true;
    h.svc.keep(&own).unwrap();
    h.svc.hygiene_step("alpha");
    assert_eq!(
        state(&h, "agent-a1").map(|s| s.0),
        Some(HygieneState::Removing)
    );
    h.svc.keep(&ask(&h, &path)).unwrap();
    h.svc.keep(&answer(&h, &path)).unwrap();
    assert!(
        h.svc.keep(&own).is_err(),
        "a confirmed keep is not withdrawn"
    );
    h.svc.hygiene_step("alpha");
    assert_eq!(kind(&h, "agent-a1"), Some(HygieneState::Exception));
}
