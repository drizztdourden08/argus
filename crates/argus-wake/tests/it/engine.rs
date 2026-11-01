//! KEEP: K1.
//! The wake engine on in-memory storage and a manual clock: coalescing windows, the 8 lines and "+N more",
//! listener matching (thread, `*`), queued events and their one-time replay inside 24 hours, recovery of
//! windows left open, the page mirror (last 50), and the listener registry (newer replaces older, legacy).

// test-tag: critical

use std::sync::Arc;

use argus_wake::engine::{AttachOptions, EngineConfig, WakeEngine, channel};
use argus_wake::store::{MIRROR_KEEP, WakeStore};
use argus_wake::testkit::{DAY, EngineRig, HOUR, START, Tap, wake};
use argus_wake::time::iso;
use argus_wake::wake::{WakeInput, WakeStatus};

fn status(rig: &EngineRig, id: &str) -> WakeStatus {
    rig.store.get(id).expect("read").expect("row").status
}

fn listen(rig: &mut EngineRig, thread: &str) -> Tap {
    rig.listen("alpha", thread, AttachOptions::default())
}

fn v1() -> AttachOptions {
    AttachOptions { v: 1, label: None }
}

fn listeners_of_row(rig: &EngineRig, id: &str) -> u32 {
    rig.store.get(id).expect("read").expect("row").listeners
}

fn stamped(wakes: &[(&str, &str)]) -> Vec<WakeInput> {
    wakes
        .iter()
        .map(|(thread, text)| wake(thread, text))
        .collect()
}

fn kanban(thread: &str, text: &str, subject: &str) -> WakeInput {
    WakeInput::about("alpha", thread, "kanban-stale", subject, text)
}

#[test]
fn wakes_inside_one_window_become_one_event_and_nothing_is_sent_before_it_closes() {
    let mut rig = EngineRig::new(15_000);
    let mut ui = listen(&mut rig, "ui");
    rig.submit(&stamped(&[("ui", "T-1 failed: a")]));
    rig.advance(5000);
    rig.submit(&stamped(&[
        ("ui", "T-2 failed: b"),
        ("ui", "T-3 failed: c"),
    ]));
    rig.advance(9000);
    assert_eq!(ui.count(), 0, "window of 15 s not over yet");
    rig.advance(1000);
    assert_eq!(ui.count(), 1);
    let event = ui.events()[0].clone();
    assert_eq!(
        (
            event.id.as_str(),
            event.scope.as_str(),
            event.thread.as_str(),
            event.count
        ),
        ("W-1", "alpha", "ui", 3)
    );
    assert_eq!(
        event.lines,
        ["T-1 failed: a", "T-2 failed: b", "T-3 failed: c"]
    );
    assert_eq!(
        event.at,
        iso(START),
        "the event is stamped with its first wake"
    );
    assert_eq!(status(&rig, "W-1"), WakeStatus::Delivered);
    assert_eq!(status(&rig, "W-2"), WakeStatus::Coalesced);
    assert_eq!(status(&rig, "W-3"), WakeStatus::Coalesced);
    rig.advance(60_000);
    assert_eq!(ui.count(), 1);
}

#[test]
fn a_window_is_per_project_and_thread() {
    let mut rig = EngineRig::new(15_000);
    let mut all = listen(&mut rig, "*");
    let mut other = rig.listen("beta", "*", AttachOptions::default());
    let mut beta = wake("ui", "for beta");
    beta.scope = "beta".to_owned();
    let mut wakes = stamped(&[("ui", "for ui"), ("api", "for api"), ("ui", "for ui again")]);
    wakes.push(beta);
    rig.submit(&wakes);
    rig.advance(15_000);
    let seen: Vec<(String, Vec<String>)> = all
        .events()
        .iter()
        .map(|e| (e.thread.clone(), e.lines.clone()))
        .collect();
    assert_eq!(
        seen,
        [
            (
                "ui".to_owned(),
                vec!["for ui".to_owned(), "for ui again".to_owned()]
            ),
            ("api".to_owned(), vec!["for api".to_owned()])
        ]
    );
    assert_eq!(other.lines(), [["for beta"]]);
}

#[test]
fn an_event_carries_at_most_8_lines_and_then_plus_n_more() {
    let mut rig = EngineRig::new(15_000);
    let mut ui = listen(&mut rig, "ui");
    let texts: Vec<String> = (1..=11).map(|i| format!("line {i}")).collect();
    let wakes: Vec<WakeInput> = texts.iter().map(|t| wake("ui", t)).collect();
    rig.submit(&wakes);
    rig.advance(15_000);
    let event = ui.events()[0].clone();
    assert_eq!(event.count, 11);
    assert_eq!(event.lines.len(), 9);
    assert_eq!(&event.lines[7..], ["line 8", "+3 more"]);
}

#[test]
fn the_window_length_comes_from_the_configuration_and_0_sends_at_once() {
    let mut rig = EngineRig::new(200);
    let mut a = listen(&mut rig, "ui");
    rig.submit(&stamped(&[("ui", "x")]));
    rig.advance(199);
    assert_eq!(a.count(), 0);
    rig.advance(1);
    assert_eq!(a.count(), 1);
    let mut instant = EngineRig::new(0);
    let mut b = listen(&mut instant, "ui");
    instant.submit(&stamped(&[("ui", "x"), ("ui", "y")]));
    assert_eq!(b.lines(), [["x", "y"]]);
}

#[test]
fn a_listener_for_one_thread_does_not_get_another_and_star_gets_all() {
    let mut rig = EngineRig::new(15_000);
    let mut ui = listen(&mut rig, "ui");
    let mut api = listen(&mut rig, "api");
    let mut star = listen(&mut rig, "*");
    rig.submit(&stamped(&[("ui", "hello ui")]));
    rig.advance(15_000);
    assert_eq!((ui.count(), api.count(), star.count()), (1, 0, 1));
    assert_eq!(rig.mirrored("alpha")["W-1"]["listeners"], 2);
    let mut stats = rig.engine.listener_stats();
    stats.sort_by(|a, b| a.thread.cmp(&b.thread));
    let rows: Vec<(&str, usize)> = stats.iter().map(|s| (s.thread.as_str(), s.count)).collect();
    assert_eq!(rows, [("*", 1), ("api", 1), ("ui", 1)]);
    for tap in [&ui, &star, &api] {
        rig.engine.detach(&tap.attached.id);
    }
    assert_eq!(rig.engine.listener_stats().len(), 0);
}

#[test]
fn with_no_listener_the_event_stays_queued_and_goes_once_to_the_first_listener_that_connects() {
    let mut rig = EngineRig::new(15_000);
    rig.submit(&stamped(&[
        ("ui", "T-1 failed: a"),
        ("ui", "T-2 failed: b"),
    ]));
    rig.advance(15_000);
    assert_eq!(status(&rig, "W-1"), WakeStatus::Queued);
    assert_eq!(rig.mirrored("alpha")["W-1"]["listeners"], 0);
    assert_eq!(rig.mirrored("alpha")["W-2"]["status"], "coalesced");
    let mut wrong = listen(&mut rig, "api");
    assert_eq!(wrong.count(), 0, "another thread does not take it");
    rig.advance(2 * HOUR);
    let mut late = listen(&mut rig, "ui");
    assert_eq!(late.attached.replayed, 1);
    let event = late.events()[0].clone();
    assert_eq!((event.id.as_str(), event.count), ("W-1", 2));
    assert_eq!(event.lines, ["T-1 failed: a", "T-2 failed: b"]);
    assert_eq!(status(&rig, "W-1"), WakeStatus::Delivered);
    assert_eq!(rig.mirrored("alpha")["W-1"]["listeners"], 1);
    assert_eq!(
        rig.mirrored("alpha")["W-2"]["listeners"],
        1,
        "a coalesced row shows what its event got"
    );
    assert_eq!(listen(&mut rig, "ui").count(), 0, "replayed only once");
}

#[test]
fn a_star_listener_that_connects_late_gets_the_queued_events_of_every_thread_oldest_first() {
    let mut rig = EngineRig::new(15_000);
    rig.submit(&stamped(&[("ui", "for ui")]));
    rig.advance(1000);
    rig.submit(&stamped(&[("api", "for api")]));
    rig.advance(20_000);
    let mut star = listen(&mut rig, "*");
    let seen: Vec<(String, String)> = star
        .events()
        .iter()
        .map(|e| (e.thread.clone(), e.lines[0].clone()))
        .collect();
    assert_eq!(
        seen,
        [
            ("ui".to_owned(), "for ui".to_owned()),
            ("api".to_owned(), "for api".to_owned())
        ]
    );
}

#[test]
fn an_event_older_than_24_hours_is_not_replayed() {
    let mut rig = EngineRig::new(15_000);
    rig.submit(&stamped(&[("ui", "old")]));
    rig.advance(15_000);
    rig.advance(DAY);
    let mut late = listen(&mut rig, "ui");
    assert_eq!(late.count(), 0);
    assert_eq!(status(&rig, "W-1"), WakeStatus::Queued);
}

#[test]
fn a_listener_that_fails_to_take_a_replayed_event_leaves_it_queued() {
    let mut rig = EngineRig::new(15_000);
    rig.submit(&stamped(&[("ui", "x")]));
    rig.advance(15_000);
    let (sink, rx) = channel();
    drop(rx);
    let gone = rig
        .engine
        .attach("alpha", "ui", Box::new(sink), AttachOptions::default());
    assert_eq!(gone.replayed, 0);
    assert_eq!(status(&rig, "W-1"), WakeStatus::Queued);
    rig.engine.detach(&gone.id);
    assert_eq!(listen(&mut rig, "ui").count(), 1);
}

#[test]
fn windows_left_open_when_the_server_stopped_are_closed_at_the_next_start_as_one_event_per_thread()
{
    let mut rig = EngineRig::new(15_000);
    rig.submit(&stamped(&[("ui", "a"), ("ui", "b"), ("api", "c")]));
    let config = EngineConfig {
        window_ms: 15_000,
        ..EngineConfig::default()
    };
    let mut second = WakeEngine::new(
        Box::new(rig.store.clone()),
        Arc::new(rig.docs.clone()),
        Arc::new(rig.clock.clone()),
        config,
    );
    assert_eq!(second.recover().expect("recover"), 2);
    assert_eq!(status(&rig, "W-1"), WakeStatus::Queued);
    assert_eq!(status(&rig, "W-2"), WakeStatus::Coalesced);
    let (sink, mut rx) = channel();
    second.attach("alpha", "ui", Box::new(sink), AttachOptions::default());
    let mut lines = Vec::new();
    while let Ok(message) = rx.try_recv() {
        if let argus_wake::engine::ListenerMessage::Wake(event) = message {
            lines.push(event.lines);
        }
    }
    assert_eq!(lines, [["a", "b"]]);
}

#[test]
fn stop_closes_the_open_windows_delivering_to_whoever_still_listens() {
    let mut rig = EngineRig::new(15_000);
    let mut ui = listen(&mut rig, "ui");
    rig.submit(&stamped(&[("ui", "a")]));
    rig.engine.stop();
    assert_eq!(ui.count(), 1);
    assert_eq!(rig.engine.pending(), 0);
}

#[test]
fn the_page_mirror_keeps_the_last_50_wakes_of_a_project_and_drops_the_older_ones() {
    let mut rig = EngineRig::new(0);
    let _ui = listen(&mut rig, "ui");
    for i in 1..=MIRROR_KEEP + 5 {
        rig.submit(&stamped(&[("ui", &format!("wake {i}"))]));
    }
    let docs = rig.mirrored("alpha");
    assert_eq!(docs.len(), MIRROR_KEEP);
    assert!(
        !docs.contains_key("W-5")
            && docs.contains_key("W-6")
            && docs.contains_key(&format!("W-{}", MIRROR_KEEP + 5))
    );
    let mut keys: Vec<&str> = docs["W-6"]
        .as_object()
        .expect("doc")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "at",
            "count",
            "id",
            "listeners",
            "reason",
            "status",
            "subject",
            "text",
            "thread"
        ]
    );
    assert!(rig.mirrored("beta").is_empty());
}

#[test]
fn rows_older_than_a_week_are_pruned_with_their_mirror_docs() {
    let mut rig = EngineRig::new(0);
    rig.submit(&stamped(&[("ui", "old")]));
    rig.advance(8 * DAY);
    assert_eq!(rig.store.prune(START + 8 * DAY).expect("prune"), ["alpha"]);
    rig.engine.remirror("alpha");
    assert!(rig.mirrored("alpha").is_empty());
}

#[test]
fn a_newer_v2_listener_for_the_same_project_and_thread_key_replaces_the_older_one() {
    let mut rig = EngineRig::new(15_000);
    let mut first = listen(&mut rig, "ui");
    let mut star = listen(&mut rig, "*");
    let mut other = rig.listen("beta", "ui", AttachOptions::default());
    assert_eq!(rig.engine.listener_total("alpha"), 2);
    let mut second = rig.listen(
        "alpha",
        "ui",
        AttachOptions {
            v: 2,
            label: Some("ui again".to_owned()),
        },
    );
    assert_eq!(first.replaced(), 1);
    assert_eq!(
        rig.engine.listener_total("alpha"),
        2,
        "one ui listener and the star one"
    );
    assert!(rig.engine.has_listener("alpha", "ui"));
    assert!(!rig.engine.has_listener("alpha", "api"));
    rig.submit(&stamped(&[("ui", "x")]));
    rig.advance(15_000);
    assert_eq!(
        (first.count(), second.count(), star.count(), other.count()),
        (0, 1, 1, 0)
    );
    assert_eq!(
        listeners_of_row(&rig, "W-1"),
        2,
        "two distinct keys matched: ui and *"
    );
    let mut open: Vec<(String, Option<String>, u8)> = rig
        .engine
        .listeners_of("alpha")
        .into_iter()
        .map(|l| (l.thread, l.label, l.v))
        .collect();
    open.sort();
    assert_eq!(
        open,
        [
            ("*".to_owned(), None, 2),
            ("ui".to_owned(), Some("ui again".to_owned()), 2)
        ]
    );
}

#[test]
fn a_legacy_listener_is_served_live_but_never_counted_replayed_to_or_replacing() {
    let mut rig = EngineRig::new(15_000);
    let mut legacy = rig.listen("alpha", "ui", v1());
    let mut modern = listen(&mut rig, "ui");
    assert_eq!(
        legacy.replaced(),
        0,
        "a v2 listener does not replace a legacy one"
    );
    assert_eq!(rig.engine.listener_total("alpha"), 2);
    let _again = rig.listen("alpha", "ui", v1());
    assert_eq!(
        rig.engine.listener_total("alpha"),
        3,
        "legacy ones pile up, none replaces another"
    );
    rig.submit(&stamped(&[("ui", "a")]));
    rig.advance(15_000);
    assert_eq!(
        (legacy.count(), modern.count()),
        (1, 1),
        "it still hears the live event"
    );
    assert_eq!(
        listeners_of_row(&rig, "W-1"),
        1,
        "only the v2 listener counts"
    );
    rig.engine.detach(&modern.attached.id);
    rig.submit(&stamped(&[("ui", "b")]));
    rig.advance(15_000);
    assert_eq!(legacy.count(), 2);
    let row = rig.store.get("W-2").expect("read").expect("row");
    assert_eq!(
        (row.status, row.listeners),
        (WakeStatus::Queued, 0),
        "heard only by legacy: nobody counted, stays queued"
    );
    let mut late = rig.listen("alpha", "ui", v1());
    assert_eq!(
        (late.attached.replayed, late.count()),
        (0, 0),
        "no replay to a legacy listener"
    );
    assert_eq!(status(&rig, "W-2"), WakeStatus::Queued);
    assert_eq!(
        listen(&mut rig, "ui").count(),
        1,
        "the next v2 listener gets it"
    );
    assert_eq!(status(&rig, "W-2"), WakeStatus::Delivered);
}

#[test]
fn an_immediate_reason_closes_the_window_at_once_with_whatever_was_waiting_and_a_result_still_waits()
 {
    let config = EngineConfig {
        window_ms: 15_000,
        immediate: vec!["owner-decision".to_owned()],
        ..EngineConfig::default()
    };
    let mut rig = EngineRig::with(config);
    let mut ui = listen(&mut rig, "ui");
    rig.submit(&stamped(&[("ui", "T-1 failed: a")]));
    rig.advance(1000);
    assert_eq!(ui.count(), 0, "a result waits for the window");
    rig.submit(&[WakeInput::about(
        "alpha",
        "ui",
        "owner-decision",
        "Q-1",
        "Decided: yes",
    )]);
    assert_eq!(ui.count(), 1, "the decision does not wait");
    assert_eq!(
        ui.events()[0].count,
        2,
        "and takes the result that was waiting with it"
    );
    rig.submit(&stamped(&[("ui", "T-2 failed: b")]));
    rig.advance(15_000);
    assert_eq!(ui.count(), 2, "a later result opens a new window");
}

#[test]
fn last_at_is_when_the_newest_wake_of_a_reason_went_to_a_thread_and_0_when_none() {
    let mut rig = EngineRig::new(15_000);
    let last = |rig: &EngineRig, scope: &str, subject: Option<&str>| {
        rig.store
            .last_at(scope, "ui", "kanban-stale", subject)
            .expect("read")
    };
    assert_eq!(last(&rig, "alpha", Some("ui")), 0);
    rig.submit(&[kanban("ui", "a", "ui")]);
    rig.advance(2 * HOUR);
    rig.submit(&[
        kanban("ui", "b", "ui"),
        kanban("ui", "c", "other"),
        wake("ui", "d"),
    ]);
    assert_eq!(last(&rig, "alpha", Some("ui")), START + 2 * HOUR);
    assert_eq!(last(&rig, "alpha", None), START + 2 * HOUR, "any subject");
    assert_eq!(last(&rig, "alpha", Some("nobody")), 0);
    assert_eq!(last(&rig, "beta", Some("ui")), 0);
}

#[test]
fn texts_over_300_characters_are_cut_with_an_ellipsis() {
    let mut rig = EngineRig::new(0);
    let mut ui = listen(&mut rig, "ui");
    rig.submit(&stamped(&[
        ("ui", &"x".repeat(400)),
        ("", "no thread"),
        ("ui", ""),
    ]));
    let line = ui.events()[0].lines[0].clone();
    assert_eq!(line.chars().count(), 300);
    assert!(line.ends_with('…'));
    assert_eq!(ui.count(), 1, "a wake with no thread or no text is skipped");
}

#[test]
fn the_listener_cap_counts_a_replacing_listener_as_free() {
    let mut rig = EngineRig::new(15_000);
    for i in 0..argus_wake::engine::MAX_LISTENERS_PER_SCOPE {
        let _tap = listen(&mut rig, &format!("t{i}"));
    }
    assert!(rig.engine.listener_cap_reached("alpha", "extra", 2));
    assert!(
        !rig.engine.listener_cap_reached("alpha", "t3", 2),
        "it replaces the open one"
    );
    assert!(rig.engine.listener_cap_reached("alpha", "t3", 1));
    assert!(!rig.engine.listener_cap_reached("beta", "t3", 2));
}

#[test]
fn a_listener_is_listed_with_its_thread_since_label_and_version() {
    let mut rig = EngineRig::new(15_000);
    let options = AttachOptions {
        v: 2,
        label: Some("ui session".to_owned()),
    };
    let _ui = rig.listen("alpha", "ui", options);
    let open = rig.engine.listeners_of("alpha");
    assert_eq!(open.len(), 1);
    assert_eq!(
        (open[0].thread.as_str(), open[0].label.as_deref(), open[0].v),
        ("ui", Some("ui session"), 2)
    );
    assert_eq!(open[0].since, iso(START));
    let json = serde_json::to_value(&open[0]).expect("json");
    let mut keys: Vec<&str> = json
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["label", "since", "thread", "v"]);
}
