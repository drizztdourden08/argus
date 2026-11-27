// test-tag: critical
//! The rules service over a fake world and a paused clock.

#![allow(clippy::assert_is_empty)]

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use argus_accounts::handover::{CycleEvent, CycleMode};
use argus_accounts::rules::{CycleView, LeadKey, View};
use argus_accounts::types::{HandoverMode, Rules};
use argus_notify::Level;
use async_trait::async_trait;
use serde_json::Value;

use super::{RulesDesk, Snapshot, World};
use crate::services::accounts_switch_log::Line;

#[derive(Default)]
struct Fake {
    snapshot: Mutex<Option<Snapshot>>,
    calls: Mutex<Vec<String>>,
    docs: Mutex<Vec<Value>>,
    /// The step log, every line in order.
    lines: Mutex<Vec<String>>,
    switch_fails: Mutex<bool>,
}

fn held<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Fake {
    fn set(&self, snapshot: Snapshot) {
        *held(&self.snapshot) = Some(snapshot);
    }

    fn change(&self, f: impl FnOnce(&mut Snapshot)) {
        f(held(&self.snapshot).as_mut().expect("snapshot set"));
    }

    fn log(&self, line: impl Into<String>) {
        held(&self.calls).push(line.into());
    }

    fn calls(&self) -> Vec<String> {
        held(&self.calls).clone()
    }

    fn toasts(&self) -> Vec<String> {
        self.calls()
            .into_iter()
            .filter(|c| c.starts_with("toast"))
            .collect()
    }

    fn switched(&self) -> bool {
        self.calls().iter().any(|c| c.starts_with("switch"))
    }

    fn lines(&self) -> Vec<String> {
        held(&self.lines).clone()
    }

    fn doc_phases(&self) -> Vec<String> {
        held(&self.docs)
            .iter()
            .map(|d| d["phase"].as_str().unwrap_or_default().to_owned())
            .collect()
    }
}

#[async_trait]
impl World for Fake {
    async fn snapshot(&self) -> Snapshot {
        held(&self.snapshot).clone().expect("snapshot set")
    }
    async fn start_cycle(
        &self,
        account: &str,
        mode: CycleMode,
        ask_at: Option<f64>,
        _use_now: Option<f64>,
    ) -> Result<(), String> {
        self.log(format!("start {account} {mode:?} {ask_at:?}"));
        Ok(())
    }
    async fn ask_all(&self, _use_now: Option<f64>) -> Result<(), String> {
        self.log("ask_all");
        Ok(())
    }
    async fn cut_off_rest(&self, _use_now: Option<f64>) -> Result<(), String> {
        self.log("cut_off");
        Ok(())
    }
    async fn re_ask(&self, lead: &LeadKey) -> Result<(), String> {
        self.log(format!("re_ask {}", lead.thread));
        Ok(())
    }
    async fn add_lead(&self, lead: &LeadKey) -> Result<(), String> {
        self.log(format!("add {}", lead.thread));
        Ok(())
    }
    async fn cancel_cycle(&self) {
        self.log("cancel_cycle");
    }
    async fn spend_cycle(&self) {
        self.log("spend_cycle");
        self.change(|s| s.cycle = CycleView::None);
    }
    async fn publish(&self, doc: Value, lines: Vec<Line>) {
        held(&self.docs).push(doc);
        held(&self.lines).extend(lines.into_iter().map(|l| l.text));
    }
    async fn note(&self, lines: Vec<Line>) {
        held(&self.lines).extend(lines.into_iter().map(|l| l.text));
    }
    async fn shown(&self) -> Option<Value> {
        held(&self.docs).last().cloned()
    }
    async fn toast(&self, level: Level, title: &str, _text: &str) {
        self.log(format!("toast {level:?} {title}"));
    }
    async fn refresh(&self) {
        self.log("refresh");
    }
    async fn learn(&self, cycle_id: &str) {
        self.log(format!("learn {cycle_id}"));
    }
    async fn switch(&self, to: &str, handover: bool) -> Result<(), String> {
        self.log(format!("switch {to} {handover}"));
        if *held(&self.switch_fails) {
            return Err("boom".to_owned());
        }
        Ok(())
    }
    fn switching(&self) -> bool {
        false
    }
    fn warn(&self, what: &str, _detail: &str) {
        self.log(format!("warn {what}"));
    }
}

fn view(key: &str, percent: f64, handover: HandoverMode) -> View {
    View {
        key: key.to_owned(),
        label: key.to_uppercase(),
        can_receive: true,
        rules: Rules {
            auto_switch: true,
            switch_at: 98.0,
            handover,
            can_receive: true,
        },
        percent: Some(percent),
        critical: false,
        resets_at_ms: Some(4_000_000_000_000),
    }
}

fn snapshot(a: f64, b: f64, handover: HandoverMode) -> Snapshot {
    Snapshot {
        accounts: vec![view("a", a, handover), view("b", b, HandoverMode::Off)],
        active: Some("a".to_owned()),
        estimates: None,
        cycle: CycleView::None,
        plan: None,
    }
}

fn desk_with(snap: Snapshot) -> (Arc<Fake>, Arc<RulesDesk>) {
    let world = Arc::new(Fake::default());
    world.set(snap);
    let desk = RulesDesk::new(world.clone());
    (world, desk)
}

async fn settle() {
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
}

async fn wait_grace() {
    tokio::time::sleep(Duration::from_secs(61)).await;
    settle().await;
}

#[tokio::test(start_paused = true)]
async fn the_switch_point_starts_a_countdown_with_a_doc_and_one_urgent_toast() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    assert_eq!(world.doc_phases(), ["countdown"]);
    let doc = held(&world.docs).remove(0);
    assert_eq!(doc["from"], "a");
    assert_eq!(doc["to"], "b");
    assert_eq!(doc["startedBy"], "rule");
    assert!(doc["countdownEndsAt"].as_i64().unwrap() > 0);
    assert!(doc["id"].as_str().unwrap().starts_with("SW-"));
    assert_eq!(world.calls(), ["toast Urgent Switching to B in 1 minute"]);
    desk.evaluate().await;
    assert_eq!(
        world.calls().len(),
        1,
        "a running countdown starts no second one"
    );
}

#[tokio::test(start_paused = true)]
async fn after_the_grace_it_reads_the_usage_again_and_switches() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    wait_grace().await;
    let calls = world.calls();
    assert!(calls.contains(&"refresh".to_owned()));
    assert_eq!(calls.last().unwrap(), "switch b false");
}

#[tokio::test(start_paused = true)]
async fn a_rule_with_a_handover_asks_the_switch_for_one() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Auto));
    desk.evaluate().await;
    wait_grace().await;
    assert_eq!(world.calls().last().unwrap(), "switch b true");
}

#[tokio::test(start_paused = true)]
async fn a_completed_cycle_means_the_switch_skips_the_handover() {
    let mut snap = snapshot(90.0, 10.0, HandoverMode::Auto);
    snap.cycle = CycleView::Ready;
    let (world, desk) = desk_with(snap);
    desk.evaluate().await;
    wait_grace().await;
    assert_eq!(world.calls().last().unwrap(), "switch b false");
    let calls = world.calls();
    let spent = calls.iter().position(|c| c == "spend_cycle").unwrap();
    assert_eq!(calls[spent + 1], "switch b false", "closed before the switch");
    desk.evaluate().await;
    assert_eq!(
        world.toasts().len(),
        1,
        "the spent cycle does not start another countdown"
    );
}

#[tokio::test(start_paused = true)]
async fn the_grace_end_stands_down_when_the_use_dropped() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    world.change(|s| s.accounts[0].percent = Some(80.0));
    wait_grace().await;
    assert!(!world.switched());
    assert_eq!(world.doc_phases(), ["countdown", "cancelled"]);
    assert!(
        world
            .toasts()
            .contains(&"toast Info Switch stood down".to_owned())
    );
}

#[tokio::test(start_paused = true)]
async fn the_grace_end_stands_down_when_claude_changed_account() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    world.change(|s| s.active = Some("b".to_owned()));
    wait_grace().await;
    assert!(!world.switched());
}

#[tokio::test(start_paused = true)]
async fn a_cancel_ends_the_countdown_and_holds_the_rule_off() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    assert!(desk.cancel().await);
    assert!(!desk.cancel().await);
    assert_eq!(world.doc_phases(), ["countdown", "cancelled"]);
    assert!(world.calls().contains(&"cancel_cycle".to_owned()));
    wait_grace().await;
    desk.evaluate().await;
    assert!(!world.switched());
    assert_eq!(
        world.doc_phases().len(),
        2,
        "no new countdown while held off"
    );
    world.change(|s| s.accounts[0].percent = Some(90.0));
    desk.evaluate().await;
    world.change(|s| s.accounts[0].percent = Some(99.0));
    desk.evaluate().await;
    assert_eq!(
        world.doc_phases().len(),
        3,
        "the hold-off ended five points under"
    );
}

#[tokio::test(start_paused = true)]
async fn switch_now_ends_the_countdown_without_a_hold_off() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    assert!(desk.end().await);
    wait_grace().await;
    assert!(!world.switched());
    assert_eq!(
        world.doc_phases(),
        ["countdown"],
        "the switch writes the doc itself"
    );
    desk.evaluate().await;
    assert_eq!(
        world.doc_phases().len(),
        2,
        "no hold-off: the rule may count down again"
    );
}

#[tokio::test(start_paused = true)]
async fn a_failed_switch_holds_the_rule_off() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    *held(&world.switch_fails) = true;
    desk.evaluate().await;
    wait_grace().await;
    assert!(
        world
            .calls()
            .contains(&"warn rule switch failed".to_owned())
    );
    desk.evaluate().await;
    assert_eq!(
        world.doc_phases(),
        ["countdown", "failed"],
        "a failed switch never leaves the countdown on screen"
    );
    let failed = held(&world.docs).last().cloned().expect("the failed doc");
    assert_eq!(failed["error"], "boom");
    assert!(
        world
            .toasts()
            .contains(&"toast Urgent The automatic switch failed".to_owned())
    );
    assert_eq!(world.lines().last().unwrap(), "The switch failed: boom.");
}

#[tokio::test(start_paused = true)]
async fn handover_on_at_the_point_asks_every_lead_before_the_countdown() {
    let (world, desk) = desk_with(snapshot(100.0, 10.0, HandoverMode::Auto));
    desk.evaluate().await;
    assert_eq!(
        world.calls(),
        [
            "start a Now None",
            "toast Urgent Switching to B in 1 minute"
        ]
    );
    assert_eq!(world.doc_phases(), ["countdown"]);
    wait_grace().await;
    assert_eq!(world.calls().last().unwrap(), "switch b true");
}

#[tokio::test(start_paused = true)]
async fn the_step_log_names_the_countdown_and_its_end() {
    let (world, desk) = desk_with(snapshot(98.0, 10.0, HandoverMode::Off));
    desk.evaluate().await;
    wait_grace().await;
    let lines = world.lines();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("Countdown started: the switch to B"));
    assert_eq!(lines[1], "The countdown ended: the switch to B starts.");
}

#[tokio::test(start_paused = true)]
async fn no_target_is_told_once() {
    let (world, desk) = desk_with(snapshot(99.0, 99.0, HandoverMode::Off));
    desk.evaluate().await;
    desk.evaluate().await;
    assert_eq!(world.toasts(), ["toast Info A has nowhere to switch"]);
    assert!(held(&world.docs).is_empty());
}

#[tokio::test(start_paused = true)]
async fn the_fixed_ask_point_opens_a_cycle_and_asks() {
    let (world, desk) = desk_with(snapshot(92.0, 10.0, HandoverMode::At(90.0)));
    desk.evaluate().await;
    assert_eq!(world.calls(), ["start a Fixed Some(90.0)", "ask_all"]);
}

#[tokio::test(start_paused = true)]
async fn a_ready_event_learns_once_and_starts_the_countdown() {
    let mut snap = snapshot(90.0, 10.0, HandoverMode::Auto);
    snap.cycle = CycleView::Ready;
    let (world, desk) = desk_with(snap);
    let event = CycleEvent {
        cycle: "SW-4".to_owned(),
        account: "a".to_owned(),
        use_at_ready: Some(90.0),
        cut_off: Vec::new(),
    };
    desk.on_event(event.clone()).await;
    desk.on_event(event).await;
    let learned = world.calls().iter().filter(|c| *c == "learn SW-4").count();
    assert_eq!(learned, 1);
    assert_eq!(world.doc_phases(), ["countdown"]);
}
