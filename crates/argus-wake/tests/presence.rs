//! Board presence: a thread's board tab (a page opened with `?thread=<id>`) is tracked, and the registry
//! opens a `board-closed` fault, waking the Overlord at once, when a live thread whose session is alive has
//! no open board for longer than the grace. The fault clears itself when the board opens again. Also the
//! not-listening fault for an active session with no wake-stream.

use argus_wake::Clock;
use argus_wake::engine::AttachOptions;
use argus_wake::faults::{Fault, FaultStore, NewFault};
use argus_wake::presence::{MAX_TABS_PER_THREAD, Presence};
use argus_wake::registry::{RegistryRow, RowState};
use argus_wake::testkit::{
    HOUR, Rig, SEC, START, Tap, board, merge, presence_config, registry_boards, thread,
};
use argus_wake::time::iso;
use serde_json::json;

const STEP: i64 = 5 * SEC;

fn rig() -> Rig {
    Rig::registry_seen(&presence_config(), true, HOUR)
}

fn open(rig: &Rig) -> Vec<Fault> {
    rig.faults.list(false).expect("faults")
}

fn row(rig: &mut Rig, scope: &str, thread: &str) -> RegistryRow {
    rig.service
        .registry_rows(scope)
        .into_iter()
        .find(|r| r.thread == thread)
        .expect("row")
}

fn listen(rig: &mut Rig, scope: &str, thread: &str) -> Tap {
    rig.listen(
        scope,
        thread,
        AttachOptions {
            v: 2,
            label: Some(thread.to_owned()),
        },
    )
}

fn reasons(faults: &[Fault]) -> Vec<(String, String)> {
    faults
        .iter()
        .map(|f| (f.thread.clone(), f.reason.clone()))
        .collect()
}

fn keep_alive(rig: &mut Rig, thread: &str) {
    let now = iso(rig.clock.now_ms());
    rig.edit_thread("alpha", thread, |t| t.last_seen = now.as_str().into());
    rig.service.on_board("alpha");
}

/// A listening, working thread whose board stayed closed past the grace: one fault is open.
fn closed_board_fault(rig: &mut Rig) -> Tap {
    let tap = listen(rig, "alpha", "ui");
    working(rig, "alpha", "ui");
    rig.pass(40 * SEC, STEP);
    assert_eq!(open(rig).len(), 1);
    tap
}

/// Two quiet minutes on: no fault is open and the board is not required.
fn assert_unrequired_after_two_quiet_minutes(rig: &mut Rig) {
    rig.pass(120 * SEC, STEP);
    assert_eq!(open(rig).len(), 0);
    assert!(!row(rig, "alpha", "ui").board.board_required);
}

/// A board is required only while the thread is inside a turn (its Browser pane closes by itself
/// between turns): working, seen just now.
fn working(rig: &mut Rig, scope: &str, thread: &str) {
    let now = iso(rig.clock.now_ms());
    rig.edit_thread(scope, thread, |t| {
        t.activity = "working".into();
        t.last_seen = now.as_str().into();
    });
    rig.service.on_board(scope);
}

fn digits_between(text: &str, prefix: &str, suffix: &str) -> bool {
    text.strip_prefix(prefix).is_some_and(|rest| {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        digits > 0 && rest[digits..].starts_with('s') && rest[digits + 1..].starts_with(suffix)
    })
}

#[test]
fn tabs_attach_and_detach_a_reconnecting_tab_replaces_itself_and_the_list_and_the_cap_hold() {
    let mut p = Presence::new();
    assert!(p.open_of("a", "t").is_none());
    let first = p.attach("a", "t", "x", START);
    let second = p.attach("a", "t", "y", START);
    assert_eq!(p.open_of("a", "t").map(|o| o.tabs), Some(2));
    p.detach(&first);
    p.detach(&first);
    assert_eq!(p.open_of("a", "t").map(|o| o.tabs), Some(1));
    let again = p.attach("a", "t", "y", START + SEC);
    p.detach(&second);
    assert_eq!(
        p.open_of("a", "t").map(|o| o.tabs),
        Some(1),
        "the old stream closing must not remove the new one"
    );
    let listed: Vec<(String, String, usize)> = p
        .list()
        .into_iter()
        .map(|r| (r.project, r.thread, r.tabs))
        .collect();
    assert_eq!(listed, [("a".to_owned(), "t".to_owned(), 1)]);
    p.detach(&again);
    assert!(p.open_of("a", "t").is_none());
    for i in 0..MAX_TABS_PER_THREAD + 3 {
        p.attach("b", "t", &format!("tab{i}"), START);
    }
    assert_eq!(
        p.open_of("b", "t").map(|o| o.tabs),
        Some(MAX_TABS_PER_THREAD)
    );
}

#[test]
fn an_open_board_keeps_a_listening_thread_clean_and_the_row_says_so() {
    let mut rig = rig();
    let _ui = listen(&mut rig, "alpha", "ui");
    working(&mut rig, "alpha", "ui");
    let _token = rig.service.presence_attach("alpha", "ui", "t1");
    rig.pass(120 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0);
    let r = row(&mut rig, "alpha", "ui");
    assert!(r.board.board_required);
    assert!(r.board.board_open);
    assert!(
        r.board
            .board_since
            .as_deref()
            .is_some_and(|s| s.starts_with("20"))
    );
    assert_eq!(rig.doc("alpha", "ui").expect("doc")["boardOpen"], true);
}

#[test]
fn a_live_thread_with_a_session_and_no_open_board_waits_the_grace_then_one_fault_and_one_wake_of_the_overlord()
 {
    let mut rig = rig();
    let _ui = listen(&mut rig, "alpha", "ui");
    working(&mut rig, "alpha", "ui");
    rig.pass(25 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0, "inside the 30 s grace");
    let waiting = row(&mut rig, "alpha", "ui");
    assert!(waiting.board.board_required && !waiting.board.board_open);
    rig.pass(15 * SEC, STEP);
    let faults = open(&rig);
    assert_eq!(
        reasons(&faults),
        [("ui".to_owned(), "board-closed".to_owned())]
    );
    assert_eq!(faults[0].project, "alpha");
    assert_eq!(row(&mut rig, "alpha", "ui").state, RowState::Fault);
    let wakes = rig.fault_wakes();
    assert_eq!(wakes.len(), 1);
    assert_eq!(
        (wakes[0].scope.as_str(), wakes[0].thread.as_str()),
        ("portfolio", "coordinator")
    );
    assert!(
        digits_between(
            &wakes[0].text,
            "alpha/ui: board closed, no open board tab for ",
            ""
        ),
        "{}",
        wakes[0].text
    );
    assert!(wakes[0].text.contains("coord lead"));
    rig.pass(60 * SEC, STEP);
    assert_eq!(
        rig.fault_wakes().len(),
        1,
        "one fault per thread, the Overlord is told once"
    );
}

#[test]
fn the_fault_clears_itself_by_the_registry_when_the_board_opens_again_and_closing_it_starts_a_new_grace()
 {
    let mut rig = rig();
    let _ui = closed_board_fault(&mut rig);
    let token = rig.service.presence_attach("alpha", "ui", "t1");
    assert_eq!(open(&rig).len(), 0);
    let done = rig.faults.list(true).expect("faults");
    assert_eq!(done[0].cleared_by.as_deref(), Some("registry"));
    assert_eq!(done[0].note.as_deref(), Some("the board is open again"));
    assert_eq!(row(&mut rig, "alpha", "ui").state, RowState::Listening);
    rig.service.presence_detach(&token);
    rig.pass(25 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0, "a fresh grace");
    rig.pass(10 * SEC, STEP);
    assert_eq!(open(&rig).len(), 1);
    assert_eq!(rig.faults.list(true).expect("faults").len(), 2);
}

#[test]
fn a_reload_inside_the_grace_does_not_fault() {
    let mut rig = rig();
    let _ui = listen(&mut rig, "alpha", "ui");
    working(&mut rig, "alpha", "ui");
    let token = rig.service.presence_attach("alpha", "ui", "t1");
    rig.pass(10 * SEC, STEP);
    rig.service.presence_detach(&token);
    rig.pass(20 * SEC, STEP);
    let _again = rig.service.presence_attach("alpha", "ui", "t2");
    rig.pass(120 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0);
    assert_eq!(rig.faults.list(true).expect("faults").len(), 0);
}

#[test]
fn only_a_thread_whose_session_is_alive_needs_its_board() {
    let mut rig = rig();
    rig.pass(120 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0);
    assert!(
        !row(&mut rig, "alpha", "ui").board.board_required,
        "no listener and the last sign of life was an hour ago"
    );
    assert!(!row(&mut rig, "alpha", "lead").board.board_required);
    let a_minute_ago = iso(rig.clock.now_ms() - 60 * SEC);
    rig.edit_thread("alpha", "ui", |t| {
        t.last_seen = a_minute_ago.as_str().into();
    });
    rig.service.on_board("alpha");
    assert!(
        !row(&mut rig, "alpha", "ui").board.board_required,
        "a heartbeat a minute ago is not a turn: the pane closes between turns"
    );
    working(&mut rig, "alpha", "ui");
    assert!(row(&mut rig, "alpha", "ui").board.board_required);
    rig.pass(40 * SEC, STEP);
    assert_eq!(open(&rig).len(), 1);
    rig.edit_thread("alpha", "ui", |t| t.status = "done".into());
    rig.service.on_board("alpha");
    assert!(
        !rig.service
            .registry_rows("alpha")
            .iter()
            .any(|r| r.thread == "ui" && r.board.board_required)
    );
}

#[test]
fn the_lead_is_covered_by_its_star_listener_and_the_overlord_itself_is_shown_on_the_page_only() {
    let mut rig = rig();
    let _lead = listen(&mut rig, "alpha", "*");
    let _main = listen(&mut rig, "portfolio", "coordinator");
    working(&mut rig, "alpha", "lead");
    working(&mut rig, "portfolio", "coordinator");
    rig.pass(40 * SEC, STEP);
    let mut faults: Vec<String> = open(&rig)
        .iter()
        .map(|f| format!("{}/{}", f.project, f.thread))
        .collect();
    faults.sort();
    assert_eq!(faults, ["alpha/lead", "portfolio/coordinator"]);
    assert_eq!(
        rig.fault_wakes().len(),
        1,
        "the lead is reported to the Overlord; the Overlord is not"
    );
    assert_eq!(
        row(&mut rig, "portfolio", "coordinator")
            .fault
            .expect("fault")
            .reason,
        "board-closed"
    );
}

#[test]
fn without_a_presence_tracker_nothing_is_required_or_faulted() {
    let mut rig = Rig::registry_seen(&presence_config(), false, HOUR);
    let _ui = listen(&mut rig, "alpha", "ui");
    assert_unrequired_after_two_quiet_minutes(&mut rig);
}

#[test]
fn an_active_session_with_no_wake_stream_for_a_minute_is_a_not_listening_fault_and_clears_when_the_stream_starts()
 {
    let mut rig = rig();
    keep_alive(&mut rig, "ui");
    let _board = rig.service.presence_attach("alpha", "ui", "t1");
    assert!(row(&mut rig, "alpha", "ui").flags.deaf);
    for _ in 0..11 {
        rig.pass(STEP, STEP);
        keep_alive(&mut rig, "ui");
    }
    assert_eq!(open(&rig).len(), 0, "inside the minute");
    for _ in 0..3 {
        rig.pass(STEP, STEP);
        keep_alive(&mut rig, "ui");
    }
    assert_eq!(
        reasons(&open(&rig)),
        [("ui".to_owned(), "not-listening".to_owned())]
    );
    let wakes = rig.fault_wakes();
    assert_eq!(wakes.len(), 1);
    assert!(
        digits_between(
            &wakes[0].text,
            "alpha/ui: not listening, no wake-stream for ",
            " while the session is active"
        ),
        "{}",
        wakes[0].text
    );
    let stream = listen(&mut rig, "alpha", "ui");
    assert_eq!(open(&rig).len(), 0);
    assert_eq!(
        rig.faults.list(true).expect("faults")[0]
            .cleared_by
            .as_deref(),
        Some("registry")
    );
    assert!(!row(&mut rig, "alpha", "ui").flags.deaf);
    rig.service.detach(&stream.attached.id);
}

#[test]
fn a_session_with_no_recent_heartbeat_is_not_deaf() {
    let mut rig = rig();
    rig.pass(300 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0);
    assert!(!row(&mut rig, "alpha", "ui").flags.deaf);
}

#[test]
fn a_retired_project_is_never_watched_even_when_its_board_changes() {
    let mut rig = rig();
    let mut watched = thread("x", "thread", &json!({}));
    merge(&mut watched, &json!({"lastSeen": iso(rig.clock.now_ms())}));
    rig.boards.set(
        "old",
        board(&json!({"threads": {"x": watched}, "inbox": {}})),
    );
    rig.boards.retire("old");
    for _ in 0..30 {
        rig.clock.advance(10 * SEC);
        let now = iso(rig.clock.now_ms());
        rig.edit_thread("old", "x", |t| t.last_seen = now.as_str().into());
        rig.service.on_board("old");
    }
    assert_eq!(open(&rig).iter().filter(|f| f.project == "old").count(), 0);
}

#[test]
fn after_a_dashboard_restart_nothing_is_raised_during_the_restart_grace() {
    let mut config = presence_config();
    config.registry.restart_grace_ms = 90 * SEC;
    let mut rig = Rig::registry_seen(&config, true, HOUR);
    let _ui = listen(&mut rig, "alpha", "ui");
    working(&mut rig, "alpha", "ui");
    rig.pass(85 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0, "inside the grace");
    rig.pass(15 * SEC, STEP);
    assert_eq!(
        open(&rig).len(),
        1,
        "then the board that stayed closed is reported"
    );
}

#[test]
fn a_thread_in_a_turn_with_no_listener_is_busy_not_not_listening() {
    let mut rig = rig();
    let _board = rig.service.presence_attach("alpha", "ui", "busy-tab");
    let now = iso(rig.clock.now_ms());
    rig.edit_thread("alpha", "ui", |t| {
        t.activity = "working".into();
        t.last_seen = now.as_str().into();
    });
    rig.service.on_board("alpha");
    let busy = row(&mut rig, "alpha", "ui");
    assert_eq!(busy.state, RowState::Busy);
    assert!(!busy.flags.deaf);
    let stream = listen(&mut rig, "alpha", "ui");
    assert_eq!(row(&mut rig, "alpha", "ui").state, RowState::Listening);
    rig.service.detach(&stream.attached.id);
    assert_eq!(row(&mut rig, "alpha", "ui").state, RowState::Busy);
    rig.pass(190 * SEC, STEP);
    assert_eq!(
        row(&mut rig, "alpha", "ui").state,
        RowState::Idle,
        "a turn that went quiet for minutes is idle again"
    );
}

#[test]
fn an_idle_thread_with_its_board_closed_is_never_required_or_faulted() {
    let mut rig = rig();
    let _ui = listen(&mut rig, "alpha", "ui");
    keep_alive(&mut rig, "ui");
    assert_unrequired_after_two_quiet_minutes(&mut rig);
}

#[test]
fn a_working_thread_unseen_for_over_three_minutes_is_out_of_its_turn() {
    let mut rig = rig();
    let _ui = listen(&mut rig, "alpha", "ui");
    working(&mut rig, "alpha", "ui");
    assert!(row(&mut rig, "alpha", "ui").board.board_required);
    rig.clock.advance(181 * SEC);
    rig.service.on_board("alpha");
    assert!(!row(&mut rig, "alpha", "ui").board.board_required);
}

#[test]
fn a_board_closed_fault_clears_by_the_registry_when_the_thread_goes_idle() {
    let mut rig = rig();
    let _ui = closed_board_fault(&mut rig);
    rig.edit_thread("alpha", "ui", |t| t.activity = "idle".into());
    rig.service.on_board("alpha");
    assert_eq!(open(&rig).len(), 0);
    let done = rig.faults.list(true).expect("faults");
    assert_eq!(done[0].reason, "board-closed");
    assert_eq!(done[0].cleared_by.as_deref(), Some("registry"));
    assert_eq!(
        done[0].note.as_deref(),
        Some("the thread is idle, its board is not required")
    );
    assert!(row(&mut rig, "alpha", "ui").fault.is_none());
    assert_eq!(rig.fault_wakes().len(), 1, "clearing wakes nobody");
    working(&mut rig, "alpha", "ui");
    rig.pass(25 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0, "a new turn starts a fresh grace");
    rig.pass(10 * SEC, STEP);
    assert_eq!(open(&rig).len(), 1);
}

#[test]
fn a_stale_board_closed_fault_of_an_idle_lead_is_cleared_on_start() {
    let mut rig = Rig::new(&presence_config(), true);
    for (scope, board) in registry_boards(HOUR) {
        rig.boards.set(scope, board);
    }
    for thread in ["lead", "ui"] {
        let stale = NewFault {
            scope: "alpha".to_owned(),
            thread: thread.to_owned(),
            since: iso(START - HOUR),
            reason: "board-closed".to_owned(),
            wake_id: None,
            detail: Some("no open board tab for 40s".to_owned()),
        };
        rig.faults.open(&stale).expect("fault");
    }
    assert_eq!(open(&rig).len(), 2);
    rig.service.start();
    assert_eq!(open(&rig).len(), 0);
    let done = rig.faults.list(true).expect("faults");
    assert!(
        done.iter()
            .all(|f| f.cleared_by.as_deref() == Some("registry"))
    );
}

/// The turn ends: the stop hook marks the thread idle and stamps it.
fn turn_ends(rig: &mut Rig) {
    let now = iso(rig.clock.now_ms());
    rig.edit_thread("alpha", "ui", |t| {
        t.activity = "idle".into();
        t.last_seen = now.as_str().into();
    });
    rig.service.on_board("alpha");
}

/// A listening thread in a turn with its board open for 20 s: the token of that tab.
fn turn_with_open_board(rig: &mut Rig) -> argus_wake::presence::PresenceToken {
    let _ui = listen(rig, "alpha", "ui");
    working(rig, "alpha", "ui");
    let token = rig.service.presence_attach("alpha", "ui", "t1");
    rig.pass(20 * SEC, STEP);
    token
}

fn assert_never_faulted(rig: &Rig) {
    assert_eq!(open(rig).len(), 0);
    assert_eq!(rig.faults.list(true).expect("faults").len(), 0);
}

#[test]
fn a_turn_that_ends_and_then_its_board_closing_never_faults() {
    let mut rig = rig();
    let token = turn_with_open_board(&mut rig);
    turn_ends(&mut rig);
    assert!(!row(&mut rig, "alpha", "ui").board.board_required);
    rig.service.presence_detach(&token);
    rig.pass(180 * SEC, STEP);
    assert_never_faulted(&rig);
}

#[test]
fn a_board_that_closed_less_than_the_grace_before_the_turn_ended_never_faults() {
    let mut rig = rig();
    let token = turn_with_open_board(&mut rig);
    rig.service.presence_detach(&token);
    rig.pass(25 * SEC, STEP);
    assert_eq!(open(&rig).len(), 0);
    turn_ends(&mut rig);
    rig.pass(180 * SEC, STEP);
    assert_never_faulted(&rig);
}

#[test]
fn a_thread_mid_turn_with_its_board_closed_past_the_grace_raises_exactly_one_fault() {
    let mut rig = rig();
    let token = turn_with_open_board(&mut rig);
    rig.service.presence_detach(&token);
    rig.pass(40 * SEC, STEP);
    let faults = open(&rig);
    assert_eq!(
        reasons(&faults),
        [("ui".to_owned(), "board-closed".to_owned())]
    );
    assert_eq!(rig.faults.list(true).expect("faults").len(), 1);
}
