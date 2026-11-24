//! KEEP: K1.
//! The registry on in-memory storage with a manual clock and fake boards: the row states (listening,
//! waking, idle, fault), a reaction seen through `lastSeen` or an acked inbox, the three ways a wake fails
//! (queued with nobody listening, delivered with no reaction, a write that throws), one fault per thread,
//! the notice for the Overlord (a passive `wake-fault`) and its one real wake (`fault-open`), a fault that
//! clears itself when the thread listens again or shows a sign of life, the re-alert for a deaf thread, and the page mirror.

// test-tag: critical

use argus_wake::board::InboxMessage;
use argus_wake::config::WakeConfig;
use argus_wake::engine::{AttachOptions, Delivery, ListenerSink};
use argus_wake::faults::{Fault, FaultStore};
use argus_wake::memory::{MemoryBoards, MemoryDocs, MemoryFaults};
use argus_wake::registry::{Registry, RegistryConfig, RegistryDeps, RowState};
use argus_wake::store::WakeStore;
use argus_wake::testkit::{MIN, Rig, SEC, registry_config, wake_in};
use argus_wake::time::iso;
use argus_wake::wake::{WakeEvent, WakeStatus};
use argus_wake::{Clock, ManualClock};
use serde_json::Value;

const STEP: i64 = 5 * SEC;

struct Throwing;

impl ListenerSink for Throwing {
    fn deliver(&mut self, _event: &WakeEvent) -> Delivery {
        Delivery::Failed("socket closed hard".to_owned())
    }

    fn replaced(&mut self) {}
}

fn rig() -> Rig {
    Rig::registry(&registry_config(), false)
}

fn labelled(label: &str) -> AttachOptions {
    AttachOptions {
        v: 2,
        label: Some(label.to_owned()),
    }
}

fn send(rig: &mut Rig, thread: &str, text: &str) {
    rig.service
        .submit(&[wake_in("alpha", thread, text, "test-fail")])
        .expect("submit");
}

fn open_faults(rig: &Rig) -> Vec<Fault> {
    rig.faults.list(false).expect("faults")
}

fn open_for(rig: &Rig, scope: &str, thread: &str) -> usize {
    open_faults(rig)
        .iter()
        .filter(|f| f.project == scope && f.thread == thread)
        .count()
}

fn fault_doc(rig: &Rig, scope: &str, id: &str) -> Value {
    rig.doc(scope, id).expect("doc")["fault"].clone()
}

fn doc_ids(rig: &Rig, scope: &str, collection: &str) -> Vec<String> {
    let mut ids: Vec<String> = rig
        .docs
        .list(scope, collection)
        .into_iter()
        .map(|d| d.id)
        .collect();
    ids.sort();
    ids
}

#[test]
fn rows_exist_for_live_threads_only_no_done_archived_or_placeholder_thread() {
    let mut rig = rig();
    let alpha: Vec<String> = rig
        .service
        .registry_rows("alpha")
        .into_iter()
        .map(|r| r.thread)
        .collect();
    assert_eq!(alpha, ["api", "lead", "ui"]);
    let main: Vec<String> = rig
        .service
        .registry_rows("portfolio")
        .into_iter()
        .map(|r| r.thread)
        .collect();
    assert_eq!(main, ["lead"]);
}

#[test]
fn a_thread_with_a_v2_listener_is_listening_and_the_lead_is_too_through_its_star_listener() {
    let mut rig = rig();
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Idle));
    let ui = rig.listen("alpha", "ui", labelled("ui session"));
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Listening));
    let row = rig
        .service
        .registry_rows("alpha")
        .into_iter()
        .find(|r| r.thread == "ui")
        .expect("row");
    assert_eq!(row.listener_label.as_deref(), Some("ui session"));
    assert!(
        row.since
            .as_deref()
            .is_some_and(|s| s.starts_with("2026-") && s.contains('T'))
    );
    let _lead = rig.listen("alpha", "*", labelled("lead session"));
    assert_eq!(rig.state("alpha", "lead"), Some(RowState::Listening));
    assert_eq!(
        rig.state("alpha", "api"),
        Some(RowState::Idle),
        "the lead listener does not make another thread listening"
    );
    rig.service.detach(&ui.attached.id);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Idle));
    let _legacy = rig.listen("alpha", "api", AttachOptions { v: 1, label: None });
    assert_eq!(
        rig.state("alpha", "api"),
        Some(RowState::Idle),
        "a legacy listener is not a registration"
    );
}

#[test]
fn the_registry_is_mirrored_live_as_docs_per_project_and_as_project_thread_on_the_portfolio_scope()
{
    let mut rig = rig();
    let _ui = rig.listen("alpha", "ui", labelled("ui session"));
    assert_eq!(doc_ids(&rig, "alpha", "registry"), ["api", "lead", "ui"]);
    let ui = rig.doc("alpha", "ui").expect("doc");
    assert_eq!(ui["thread"], "ui");
    assert_eq!(ui["project"], "alpha");
    assert_eq!(ui["session"], "ui session");
    assert_eq!(ui["role"], "thread");
    assert_eq!(ui["listening"], true);
    assert_eq!(ui["listenerLabel"], "ui session");
    assert_eq!(ui["state"], "listening");
    assert_eq!(ui["pendingWake"], Value::Null);
    assert_eq!(ui["fault"], Value::Null);
    assert_eq!(ui["activity"], "idle");
    assert!(ui["since"].is_string());
    let portfolio: Vec<String> = rig
        .docs
        .list("_portfolio", "registry")
        .into_iter()
        .map(|d| d.id)
        .collect();
    assert_eq!(portfolio, ["alpha/lead", "portfolio/lead"]);
    assert_eq!(
        rig.doc("_portfolio", "alpha/lead").expect("doc")["project"],
        "alpha"
    );
}

#[test]
fn a_delivered_wake_is_waking_until_the_thread_shows_a_sign_of_life_then_listening_again_with_no_fault()
 {
    let mut rig = rig();
    let _ui = rig.listen("alpha", "ui", AttachOptions::default());
    send(&mut rig, "ui", "T-1 failed: x");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Waking));
    let pending = rig.doc("alpha", "ui").expect("doc")["pendingWake"].clone();
    assert_eq!(
        (pending["id"].as_str(), pending["reason"].as_str()),
        (Some("W-1"), Some("test-fail"))
    );
    rig.pass(30 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Waking));
    rig.seen("alpha", "ui");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Listening));
    assert_eq!(
        rig.doc("alpha", "ui").expect("doc")["state"],
        "listening",
        "the doc follows at once"
    );
    rig.pass(5 * MIN, STEP);
    assert_eq!(open_faults(&rig).len(), 0);
    assert_eq!(rig.fault_wakes().len(), 0);
}

#[test]
fn a_last_seen_older_than_the_delivery_is_no_reaction() {
    let mut rig = rig();
    let _ui = rig.listen("alpha", "ui", AttachOptions::default());
    rig.clock.advance(20 * SEC);
    send(&mut rig, "ui", "T-1 failed: x");
    let earlier = iso(argus_wake::testkit::START + 15 * SEC);
    rig.edit_thread("alpha", "ui", |t| t.last_seen = earlier.as_str().into());
    rig.service.on_board("alpha");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Waking));
}

#[test]
fn acknowledging_the_inbox_that_was_waiting_is_a_reaction_too() {
    let mut rig = rig();
    rig.edit("alpha", |b| {
        b.inbox
            .insert("ui".to_owned(), vec![InboxMessage { id: "m-1".into() }]);
    });
    let _ui = rig.listen("alpha", "ui", AttachOptions::default());
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(10 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Waking));
    rig.edit("alpha", |b| {
        b.inbox.insert("ui".to_owned(), Vec::new());
    });
    rig.service.on_board("alpha");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Listening));
}

#[test]
fn a_wake_queued_with_nobody_listening_is_waking_then_a_fault_after_no_listener_ms() {
    let mut rig = rig();
    send(&mut rig, "ui", "T-1 failed: x");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Waking));
    let row = rig.store.get("W-1").expect("read").expect("row");
    assert_eq!(row.status, WakeStatus::Queued);
    rig.pass(60 * SEC, STEP);
    assert_eq!(
        rig.state("alpha", "ui"),
        Some(RowState::Waking),
        "exactly no_listener_ms is not yet more than no_listener_ms"
    );
    rig.pass(5 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
    let fault = fault_doc(&rig, "alpha", "ui");
    assert_eq!(
        (fault["reason"].as_str(), fault["wakeId"].as_str()),
        (Some("no-listener"), Some("W-1"))
    );
    let detail = fault["detail"].as_str().expect("detail");
    assert!(detail.contains("queued for 65s, nobody listening"));
    let doc = rig.doc("alpha", "ui").expect("doc");
    assert_eq!(doc["pendingWake"], Value::Null);
    let faults = open_faults(&rig);
    let rows: Vec<(&str, &str, &str, Option<&str>)> = faults
        .iter()
        .map(|f| {
            (
                f.project.as_str(),
                f.thread.as_str(),
                f.reason.as_str(),
                f.wake_id.as_deref(),
            )
        })
        .collect();
    assert_eq!(rows, [("alpha", "ui", "no-listener", Some("W-1"))]);
    assert_eq!(
        rig.doc("_portfolio", "alpha/ui").expect("doc")["state"],
        "fault"
    );
}

#[test]
fn the_overlord_is_told_once_with_a_notice_and_woken_once_when_a_wake_failed() {
    let mut rig = rig();
    let mut main = rig.listen("portfolio", "lead", AttachOptions::default());
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(70 * SEC, STEP);
    assert_eq!(
        main.count(),
        1,
        "the notice is passive, fault-open is the one model turn"
    );
    assert_eq!(rig.wakes_of("fault-open").len(), 1);
    assert_eq!(rig.fault_wakes().len(), 1);
    let notice = rig.fault_wakes()[0].clone();
    assert_eq!(
        (notice.scope.as_str(), notice.thread.as_str()),
        ("portfolio", "lead")
    );
    assert_eq!(notice.status, WakeStatus::Passive);
    assert_eq!(
        notice.text,
        "alpha/ui: no-listener (queued for 65s, nobody listening)"
    );
}

#[test]
fn a_delivered_wake_with_no_reaction_within_reaction_ms_is_a_fault_and_the_star_listener_counts_for_the_lead()
 {
    let mut rig = rig();
    let _lead = rig.listen("alpha", "*", AttachOptions::default());
    let _main = rig.listen("portfolio", "lead", AttachOptions::default());
    send(&mut rig, "lead", "owner note");
    assert_eq!(
        rig.store.get("W-1").expect("read").expect("row").status,
        WakeStatus::Delivered
    );
    rig.pass(120 * SEC, STEP);
    assert_eq!(rig.state("alpha", "lead"), Some(RowState::Waking));
    rig.pass(5 * SEC, STEP);
    assert_eq!(rig.state("alpha", "lead"), Some(RowState::Fault));
    let fault = fault_doc(&rig, "alpha", "lead");
    assert_eq!(fault["reason"], "no-reaction");
    assert!(
        fault["detail"]
            .as_str()
            .expect("detail")
            .contains("delivered, no sign of life for 125s")
    );
}

#[test]
fn a_wake_replayed_to_a_late_listener_starts_the_reaction_clock_when_it_is_delivered() {
    let mut rig = rig();
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(50 * SEC, STEP);
    let _ui = rig.listen("alpha", "ui", AttachOptions::default());
    assert_eq!(
        rig.store.get("W-1").expect("read").expect("row").status,
        WakeStatus::Delivered
    );
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Waking));
    rig.pass(100 * SEC, STEP);
    assert_eq!(
        rig.state("alpha", "ui"),
        Some(RowState::Waking),
        "no fault: the delivery was 100 s ago, not the queueing"
    );
    rig.seen("alpha", "ui");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Listening));
    assert_eq!(open_faults(&rig).len(), 0);
}

#[test]
fn a_write_that_throws_is_a_fault_at_once() {
    let mut rig = rig();
    let _main = rig.listen("portfolio", "lead", AttachOptions::default());
    rig.service
        .attach("alpha", "ui", Box::new(Throwing), AttachOptions::default());
    send(&mut rig, "ui", "T-1 failed: x");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
    let fault = fault_doc(&rig, "alpha", "ui");
    assert_eq!(
        (fault["reason"].as_str(), fault["detail"].as_str()),
        (Some("delivery-error"), Some("socket closed hard"))
    );
    assert_eq!(rig.fault_wakes().len(), 1);
}

#[test]
fn a_second_failure_of_a_thread_with_an_open_fault_is_not_recorded_and_not_sent_again() {
    let mut rig = rig();
    let _main = rig.listen("portfolio", "lead", AttachOptions::default());
    send(&mut rig, "ui", "first");
    rig.pass(70 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
    assert_eq!(rig.fault_wakes().len(), 1);
    send(&mut rig, "ui", "second");
    rig.pass(5 * MIN, STEP);
    assert_eq!(open_for(&rig, "alpha", "ui"), 1);
    let recorded = rig
        .faults
        .list(true)
        .expect("faults")
        .iter()
        .filter(|f| f.project == "alpha")
        .count();
    assert_eq!(recorded, 1);
    assert_eq!(rig.fault_wakes().len(), 1, "the Overlord is not told twice");
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
}

#[test]
fn a_no_listener_fault_clears_itself_once_the_thread_listens_again() {
    let mut rig = rig();
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(70 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
    let _ui = rig.listen("alpha", "ui", AttachOptions::default());
    rig.seen("alpha", "ui");
    rig.pass(10 * SEC, STEP);
    assert_eq!(
        open_for(&rig, "alpha", "ui"),
        0,
        "a wake can reach it now: nothing stays open"
    );
    assert_ne!(rig.state("alpha", "ui"), Some(RowState::Fault));
    let cleared = rig.faults.list(true).expect("faults");
    let cleared = cleared.iter().find(|f| f.project == "alpha").expect("row");
    assert_eq!(cleared.cleared_by.as_deref(), Some("registry"));
    assert_eq!(cleared.note.as_deref(), Some("it listens again"));
}

#[test]
fn a_deaf_thread_with_an_open_fault_that_is_sent_another_wake_tells_the_overlord_again_at_most_every_10_minutes()
 {
    let mut rig = rig();
    let mut main = rig.listen("portfolio", "lead", AttachOptions::default());
    send(&mut rig, "ui", "first");
    rig.pass(70 * SEC, STEP);
    assert_eq!(rig.fault_wakes().len(), 1);
    send(&mut rig, "ui", "second");
    rig.pass(2 * MIN, STEP);
    assert_eq!(
        rig.fault_wakes().len(),
        1,
        "inside 10 minutes of the fault: no second alert"
    );
    rig.pass(10 * MIN, STEP);
    send(&mut rig, "ui", "third: the owner is waiting");
    rig.pass(5 * SEC, STEP);
    assert_eq!(
        rig.fault_wakes().len(),
        2,
        "a new wake after 10 minutes tells the Overlord again"
    );
    let last = rig.fault_wakes().last().expect("notice").clone();
    assert!(
        last.text
            .contains("alpha/ui: a new wake is waiting and nobody listens")
    );
    assert_eq!(
        main.count(),
        1,
        "told by notices, woken once when the fault opened"
    );
    send(&mut rig, "ui", "fourth");
    rig.pass(5 * SEC, STEP);
    assert_eq!(
        rig.fault_wakes().len(),
        2,
        "and then not again within the next 10 minutes"
    );
    assert_eq!(open_for(&rig, "alpha", "ui"), 1, "still one fault, not two");
}

#[test]
fn only_clear_fault_clears_it_records_who_and_why_and_a_new_failure_opens_a_new_fault() {
    let mut rig = rig();
    let _main = rig.listen("portfolio", "lead", AttachOptions::default());
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(70 * SEC, STEP);
    let note = "ui session had crashed, restarted its stream";
    let cleared = rig
        .service
        .clear_fault("alpha", "ui", "lead", note)
        .expect("cleared");
    assert_eq!(cleared.cleared_by.as_deref(), Some("lead"));
    assert_eq!(cleared.note.as_deref(), Some(note));
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Idle));
    assert_eq!(fault_doc(&rig, "alpha", "ui"), Value::Null);
    assert_eq!(open_faults(&rig).len(), 0);
    assert!(
        rig.service
            .clear_fault("alpha", "ui", "lead", "nothing left to clear")
            .is_none()
    );
    assert_eq!(rig.service.faults(true).len(), 1);
    assert_eq!(rig.service.faults(false).len(), 0);
    send(&mut rig, "ui", "again");
    rig.pass(70 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
    assert_eq!(rig.service.faults(true).len(), 2);
    assert_eq!(rig.fault_wakes().len(), 2);
}

#[test]
fn a_fault_notice_for_an_overlord_nobody_hears_is_never_queued_so_it_raises_no_fault_of_its_own() {
    let mut rig = rig();
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(70 * SEC, STEP);
    assert_eq!(rig.fault_wakes().len(), 1);
    let id = rig.fault_wakes()[0].id.clone();
    assert_eq!(
        rig.store.get(&id).expect("read").expect("row").status,
        WakeStatus::Passive
    );
    rig.pass(70 * SEC, STEP);
    assert_ne!(rig.state("portfolio", "lead"), Some(RowState::Fault));
    assert_eq!(
        rig.fault_wakes().len(),
        1,
        "no new wake-fault for the Overlord"
    );
    let open: Vec<String> = open_faults(&rig)
        .iter()
        .map(|f| format!("{}/{}", f.project, f.thread))
        .collect();
    assert_eq!(open, ["alpha/ui"]);
}

#[test]
fn a_wake_for_a_thread_that_is_gone_done_or_archived_leaves_nothing_pending_and_no_fault() {
    let mut rig = rig();
    send(&mut rig, "api", "T-1 failed: x");
    rig.edit_thread("alpha", "api", |t| t.status = "done".into());
    rig.service.on_board("alpha");
    rig.pass(5 * MIN, STEP);
    assert_eq!(open_faults(&rig).len(), 0);
    assert_eq!(rig.service.registry().pending().len(), 0);
}

#[test]
fn timeouts_come_from_the_options_a_short_reaction_and_no_listener_ms_fault_sooner() {
    let mut config: WakeConfig = registry_config();
    config.registry.reaction_ms = 10 * SEC;
    config.registry.no_listener_ms = 7 * SEC;
    let mut rig = Rig::registry(&config, false);
    send(&mut rig, "ui", "T-1 failed: x");
    rig.pass(10 * SEC, STEP);
    assert_eq!(rig.state("alpha", "ui"), Some(RowState::Fault));
}

#[test]
fn the_overlord_is_found_by_the_id_of_its_board_not_by_the_key_portfolio() {
    let boards = MemoryBoards::new().with_overlord("overlord-7");
    for (scope, board) in argus_wake::testkit::registry_boards(MIN) {
        boards.set(
            if scope == "portfolio" {
                "overlord-7"
            } else {
                scope
            },
            board,
        );
    }
    let clock: std::sync::Arc<dyn Clock> = std::sync::Arc::new(ManualClock::at(0));
    let registry = Registry::new(
        RegistryDeps {
            boards: std::sync::Arc::new(boards),
            faults: Box::new(MemoryFaults::new()),
            docs: std::sync::Arc::new(MemoryDocs::new()),
            clock,
        },
        RegistryConfig::default(),
    );
    assert_eq!(registry.main_thread(), "lead");
    assert!(registry.is_main("overlord-7", "lead"));
    assert!(!registry.is_main("portfolio", "lead"));
}

#[test]
fn a_thread_with_only_an_open_resend_queued_is_not_due() {
    let mut rig = rig();
    rig.service
        .submit(&[wake_in("alpha", "ui", "2 faults open", "open-resend")])
        .expect("submit");
    assert_eq!(rig.service.due().len(), 0);
}

#[test]
fn an_open_resend_beside_a_new_wake_leaves_the_thread_due_with_the_new_wake_only() {
    let mut rig = rig();
    rig.service
        .submit(&[wake_in("alpha", "ui", "2 faults open", "open-resend")])
        .expect("submit");
    rig.service
        .submit(&[wake_in("alpha", "ui", "T-1 stalled", "stall")])
        .expect("submit");
    let due = rig.service.due();
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].thread, "ui");
    assert_eq!(due[0].count, 1);
}
