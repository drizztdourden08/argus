//! The switching rules of the accounts service. After every usage poll and every handover cycle
//! event the pure rules of `argus_accounts::rules` say what to do; this carries it out: it opens
//! and asks the handover cycle, runs the one-minute countdown (the `switch` state doc, one urgent
//! toast), re-checks at its end and calls the switch, keeps the hold-offs a cancel leaves and
//! teaches the estimator from each finished cycle.
//!
//! The outside world is the [`World`] trait, so tests fake the boards, the toasts and the
//! switch; the clock is the tokio clock, so tests pause it.

mod live;

use std::sync::{Arc, Weak};
use std::time::Duration;

use argus_accounts::PollResult;
use argus_accounts::handover::{CycleEvent, CycleMode};
use argus_accounts::rules::{
    Countdown, CountdownReason, CycleView, Decision, Estimates, Facts, GRACE_MS, HoldOff, LeadKey,
    StandDownReason, View, Window, decide,
};
use argus_accounts::types::HandoverMode;
use argus_core::clock::now_ms;
use argus_notify::Level;
use async_trait::async_trait;
use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard, broadcast, watch};
use tokio::task::JoinHandle;
use tokio::time::Instant;

use super::AccountsService;
use super::tasks::Timer;
use crate::Services;

const NAME: &str = "accounts";

/// What the rules read of the world in one go.
#[derive(Clone)]
pub(crate) struct Snapshot {
    /// Every account with its usage.
    pub accounts: Vec<View>,
    /// The account Claude runs as.
    pub active: Option<String>,
    /// The estimator's word on the active account.
    pub estimates: Option<Estimates>,
    /// The cycle of the active account.
    pub cycle: CycleView,
    /// The id of the last cycle, whatever it is for.
    pub cycle_id: Option<String>,
}

/// The outside world of the rules.
#[async_trait]
pub(crate) trait World: Send + Sync {
    /// The accounts, the active one, the estimates and the cycle now.
    async fn snapshot(&self) -> Snapshot;
    /// Opens a handover cycle for `account`.
    async fn start_cycle(
        &self,
        account: &str,
        mode: CycleMode,
        ask_at: Option<f64>,
        use_now: Option<f64>,
    ) -> Result<(), String>;
    /// The ask point: every lead owes its handover.
    async fn ask_all(&self, use_now: Option<f64>) -> Result<(), String>;
    /// The switch point came first.
    async fn cut_off_rest(&self, use_now: Option<f64>) -> Result<(), String>;
    /// Asks a lead for a fresh estimate.
    async fn re_ask(&self, lead: &LeadKey) -> Result<(), String>;
    /// Adds a lead that started working after the cycle did.
    async fn add_lead(&self, lead: &LeadKey) -> Result<(), String>;
    /// Calls the handover cycle off.
    async fn cancel_cycle(&self);
    /// Writes the `switch` state doc, with `lines` added to its step log.
    async fn publish(&self, doc: Value, lines: Vec<String>);
    /// Adds steps to the step log of the switch.
    async fn note(&self, lines: Vec<String>);
    /// The `switch` state doc as it is shown now.
    async fn shown(&self) -> Option<Value>;
    /// Shows a toast.
    async fn toast(&self, level: Level, title: &str, text: &str);
    /// Reads every account's usage now.
    async fn refresh(&self);
    /// Teaches the estimator from a finished cycle.
    async fn learn(&self, cycle_id: &str);
    /// Switches the Desktop app to `to`; returns when the switch is over.
    async fn switch(&self, to: &str, handover: bool) -> Result<(), String>;
    /// Whether a switch runs now, whoever started it.
    fn switching(&self) -> bool;
    /// Logs something that went wrong.
    fn warn(&self, what: &str, detail: &str);
}

/// The tokio clock read as epoch milliseconds, so a paused clock moves the rules' time too.
struct Clock {
    wall_ms: i64,
    at: Instant,
}

impl Clock {
    fn start() -> Self {
        Clock {
            wall_ms: now_ms(),
            at: Instant::now(),
        }
    }

    fn now_ms(&self) -> i64 {
        self.wall_ms + i64::try_from(self.at.elapsed().as_millis()).unwrap_or(i64::MAX)
    }
}

struct Running {
    countdown: Countdown,
    id: String,
    task: JoinHandle<()>,
}

#[derive(Default)]
struct State {
    countdown: Option<Running>,
    hold_offs: Vec<HoldOff>,
    told: Option<Window>,
    /// The cycle a switch was already made from; it does not start another countdown.
    spent: Option<String>,
    learned: Option<String>,
    switching: bool,
}

/// A switch to make once the state lock is free.
struct Go {
    from: HoldOff,
    to: String,
    handover: bool,
    /// The id of the countdown the switch comes from.
    id: String,
    /// The name of the target, for the owner.
    label: String,
}

/// The rules over a world.
pub(crate) struct RulesDesk {
    world: Arc<dyn World>,
    clock: Clock,
    state: Mutex<State>,
    me: Weak<RulesDesk>,
}

fn switch_doc(id: &str, from: &str, to: &str, phase: &str) -> Value {
    json!({
        "id": id, "from": from, "to": to, "startedBy": "rule", "phase": phase,
        "countdownEndsAt": null, "steps": [], "error": null,
    })
}

fn label_of(snapshot: &Snapshot, key: &str) -> String {
    snapshot
        .accounts
        .iter()
        .find(|v| v.key == key)
        .map_or_else(|| key.to_owned(), |v| v.label.clone())
}

fn view_of<'a>(snapshot: &'a Snapshot, key: &str) -> Option<&'a View> {
    snapshot.accounts.iter().find(|v| v.key == key)
}

impl RulesDesk {
    pub(crate) fn new(world: Arc<dyn World>) -> Arc<Self> {
        Arc::new_cyclic(|me| RulesDesk {
            world,
            clock: Clock::start(),
            state: Mutex::default(),
            me: me.clone(),
        })
    }

    /// Reads the world, decides and carries the decisions out. Called after every poll and
    /// every cycle event.
    pub(crate) async fn evaluate(&self) {
        let go = {
            let mut state = self.state.lock().await;
            if state.switching || self.world.switching() {
                return;
            }
            let snapshot = self.world.snapshot().await;
            let decisions = self.decisions(&state, &snapshot);
            self.apply(&mut state, &snapshot, decisions).await
        };
        if let Some(go) = go {
            self.switch(go).await;
        }
    }

    fn cycle_view(state: &State, snapshot: &Snapshot) -> CycleView {
        let spent = state.spent.is_some() && snapshot.cycle_id == state.spent;
        if spent && snapshot.cycle == CycleView::Ready {
            CycleView::None
        } else {
            snapshot.cycle
        }
    }

    fn decisions(&self, state: &State, snapshot: &Snapshot) -> Vec<Decision> {
        decide(&Facts {
            accounts: &snapshot.accounts,
            active: snapshot.active.as_deref(),
            estimates: snapshot.estimates.as_ref(),
            cycle: Self::cycle_view(state, snapshot),
            hold_offs: &state.hold_offs,
            countdown: state.countdown.as_ref().map(|r| &r.countdown),
            told: state.told.as_ref(),
            now_ms: self.clock.now_ms(),
        })
    }

    async fn apply(
        &self,
        state: &mut State,
        snapshot: &Snapshot,
        decisions: Vec<Decision>,
    ) -> Option<Go> {
        let active = snapshot.active.clone().unwrap_or_default();
        let use_now = view_of(snapshot, &active).and_then(|v| v.percent);
        let mut go = None;
        for decision in decisions {
            if let Decision::Switch { to } = decision {
                go = Self::begin_switch(state, snapshot, &active, to);
            } else if !self
                .carry_out(state, snapshot, (&active, use_now), decision)
                .await
            {
                break;
            }
        }
        go
    }

    async fn ask_leads(&self, leads: &[LeadKey]) {
        for lead in leads {
            self.note("estimate wake", self.world.re_ask(lead).await);
        }
    }

    async fn add_leads(&self, leads: &[LeadKey]) {
        for lead in leads {
            self.note("lead not added", self.world.add_lead(lead).await);
        }
    }

    /// Does one decision other than a switch; false when the rest of the decisions must wait.
    async fn carry_out(
        &self,
        state: &mut State,
        snapshot: &Snapshot,
        (active, use_now): (&str, Option<f64>),
        decision: Decision,
    ) -> bool {
        match decision {
            Decision::ClearHoldOff { account } => {
                state.hold_offs.retain(|h| h.account != account);
            }
            Decision::StartEstimates(leads) => self.ask_leads(&leads).await,
            Decision::AddLeads(leads) => self.add_leads(&leads).await,
            Decision::StartCycle { mode, ask_at } => {
                let started = self.world.start_cycle(active, mode, ask_at, use_now).await;
                let ok = started.is_ok();
                self.failed("The handover cycle did not start", started)
                    .await;
                // At the switch point the countdown goes on even when the cycle did not start.
                return ok || mode == CycleMode::Now;
            }
            Decision::AskAll => {
                let asked = self.world.ask_all(use_now).await;
                self.failed("The leads were not asked for their handover", asked)
                    .await;
            }
            Decision::CutOffRest => {
                let cut = self.world.cut_off_rest(use_now).await;
                self.failed("The leads not ready were not cut off", cut)
                    .await;
            }
            Decision::StartCountdown { to, reason } => {
                self.start_countdown(state, snapshot, &to, reason).await;
            }
            Decision::Switch { .. } => {}
            Decision::StandDown { reason } => self.stand_down(state, snapshot, reason).await,
            Decision::NotifyNoTarget { window } => {
                self.tell_no_target(snapshot, active).await;
                state.told = Some(window);
            }
        }
        true
    }

    fn note(&self, what: &str, result: Result<(), String>) {
        if let Err(e) = result {
            self.world.warn(what, &e);
        }
    }

    /// A failed step of the process: a log line and a step-log line.
    async fn failed(&self, what: &str, result: Result<(), String>) {
        if let Err(e) = result {
            self.world.warn(what, &e);
            self.world.note(vec![format!("{what}: {e}.")]).await;
        }
    }

    async fn start_countdown(
        &self,
        state: &mut State,
        snapshot: &Snapshot,
        to: &str,
        reason: CountdownReason,
    ) {
        let from = snapshot.active.clone().unwrap_or_default();
        let ends_at_ms = self.clock.now_ms() + GRACE_MS;
        let id = format!("SW-{}", self.clock.now_ms());
        let mut doc = switch_doc(&id, &from, to, "countdown");
        doc["countdownEndsAt"] = json!(ends_at_ms);
        let label = label_of(snapshot, to);
        let line = format!(
            "Countdown started: the switch to {label} begins in 1 minute ({}).",
            reason_text(reason)
        );
        self.world.publish(doc, vec![line]).await;
        let text = countdown_text(reason, &label_of(snapshot, &from));
        self.world
            .toast(
                Level::Urgent,
                &format!("Switching to {label} in 1 minute"),
                &text,
            )
            .await;
        let task = self.spawn_wait();
        state.countdown = Some(Running {
            countdown: Countdown {
                from,
                to: to.to_owned(),
                ends_at_ms,
            },
            id,
            task,
        });
    }

    /// The countdown task, started from a plain function so the async calls do not form a cycle.
    fn spawn_wait(&self) -> JoinHandle<()> {
        tokio::spawn(wait_out(self.me.clone()))
    }

    fn begin_switch(state: &mut State, snapshot: &Snapshot, from: &str, to: String) -> Option<Go> {
        let id = state.countdown.take()?.id;
        let view = view_of(snapshot, from)?;
        let cycle = Self::cycle_view(state, snapshot);
        if cycle == CycleView::Ready {
            state.spent.clone_from(&snapshot.cycle_id);
        }
        state.switching = true;
        Some(Go {
            from: HoldOff::after_cancel(view),
            label: label_of(snapshot, &to),
            to,
            handover: view.rules.handover != HandoverMode::Off && cycle != CycleView::Ready,
            id,
        })
    }

    async fn stand_down(&self, state: &mut State, snapshot: &Snapshot, reason: StandDownReason) {
        let Some(running) = state.countdown.take() else {
            return;
        };
        let Countdown { from, to, .. } = running.countdown;
        let mut doc = switch_doc(&running.id, &from, &to, "cancelled");
        let line = format!("Stood down: {}.", reason.text());
        doc["error"] = json!(line);
        self.world.publish(doc, vec![line]).await;
        let text = format!(
            "The switch to {} stood down: {}.",
            label_of(snapshot, &to),
            reason.text()
        );
        self.world
            .toast(Level::Info, "Switch stood down", &text)
            .await;
    }

    async fn tell_no_target(&self, snapshot: &Snapshot, active: &str) {
        let label = label_of(snapshot, active);
        let text = format!(
            "{label} is nearly spent and no other account has room. Open the Accounts tab to resume here when it resets."
        );
        self.world
            .toast(
                Level::Info,
                &format!("{label} has nowhere to switch"),
                &text,
            )
            .await;
    }

    /// Makes the switch; a failed one holds the rule off so it does not start again at once.
    async fn switch(&self, go: Go) {
        let line = format!("The countdown ended: the switch to {} starts.", go.label);
        self.world.note(vec![line]).await;
        let result = self.world.switch(&go.to, go.handover).await;
        if let Err(e) = &result {
            self.tell_failure(&go, e).await;
        }
        let mut state = self.state.lock().await;
        state.switching = false;
        if result.is_err() {
            state.hold_offs.retain(|h| h.account != go.from.account);
            state.hold_offs.push(go.from);
        }
    }

    /// A failed switch never leaves the countdown on screen: the doc fails with the reason (when
    /// the switch did not write it already) and an urgent toast says why.
    async fn tell_failure(&self, go: &Go, error: &str) {
        self.world.warn("rule switch failed", error);
        let stuck = self
            .world
            .shown()
            .await
            .is_some_and(|doc| doc["id"] == go.id.as_str() && doc["phase"] == "countdown");
        if stuck {
            let mut doc = switch_doc(&go.id, &go.from.account, &go.to, "failed");
            doc["error"] = json!(error);
            let line = format!("The switch failed: {error}.");
            self.world.publish(doc, vec![line]).await;
        }
        let text = format!(
            "The switch to {} did not happen: {error}. Open the Accounts tab to switch by hand.",
            go.label
        );
        self.world
            .toast(Level::Urgent, "The automatic switch failed", &text)
            .await;
    }

    /// Takes the running countdown out of the state and stops its task, with the state still
    /// locked and the snapshot read; `None` when no countdown runs.
    async fn stop_countdown(&self) -> Option<(MutexGuard<'_, State>, Running, Snapshot)> {
        let mut state = self.state.lock().await;
        let running = state.countdown.take()?;
        running.task.abort();
        let snapshot = self.world.snapshot().await;
        Some((state, running, snapshot))
    }

    /// Ends a running countdown at the owner's cancel and holds the rule off. Returns whether
    /// a countdown ran.
    pub(crate) async fn cancel(&self) -> bool {
        let Some((mut state, running, snapshot)) = self.stop_countdown().await else {
            return false;
        };
        let Countdown { from, to, .. } = running.countdown;
        if let Some(view) = view_of(&snapshot, &from) {
            state.hold_offs.retain(|h| h.account != from);
            state.hold_offs.push(HoldOff::after_cancel(view));
        }
        self.world.cancel_cycle().await;
        let line = "The owner cancelled the countdown.".to_owned();
        self.world
            .publish(switch_doc(&running.id, &from, &to, "cancelled"), vec![line])
            .await;
        true
    }

    /// The owner chose Switch now during a countdown: drops the countdown, without a hold-off
    /// and without a doc write (the switch reuses the countdown's id and writes the doc).
    pub(crate) async fn end(&self) -> bool {
        let Some((mut state, _running, snapshot)) = self.stop_countdown().await else {
            return false;
        };
        if Self::cycle_view(&state, &snapshot) == CycleView::Ready {
            state.spent.clone_from(&snapshot.cycle_id);
        }
        true
    }

    /// A cycle became ready: learn from it, then see whether the countdown starts.
    async fn on_event(&self, event: CycleEvent) {
        let fresh = {
            let mut state = self.state.lock().await;
            let fresh = state.learned.as_deref() != Some(event.cycle.as_str());
            if fresh {
                state.learned = Some(event.cycle.clone());
            }
            fresh
        };
        if fresh {
            self.world.learn(&event.cycle).await;
        }
        self.evaluate().await;
    }
}

fn reason_text(reason: CountdownReason) -> &'static str {
    match reason {
        CountdownReason::SwitchPoint => "the use reached the switch point",
        CountdownReason::HandoversReady => "every handover is in",
    }
}

fn countdown_text(reason: CountdownReason, from: &str) -> String {
    match reason {
        CountdownReason::SwitchPoint => {
            format!("{from} reached its switch point. Open the Accounts tab to cancel.")
        }
        CountdownReason::HandoversReady => {
            format!("Every handover from {from} is written. Open the Accounts tab to cancel.")
        }
    }
}

/// The countdown task: waits out the grace, reads the usage again, then lets the rules re-check.
async fn wait_out(desk: Weak<RulesDesk>) {
    tokio::time::sleep(Duration::from_millis(GRACE_MS.unsigned_abs())).await;
    let Some(desk) = desk.upgrade() else {
        return;
    };
    desk.world.refresh().await;
    desk.evaluate().await;
}

enum Heard {
    Ready(CycleEvent),
    Missed,
}

async fn next_event(events: &mut Option<broadcast::Receiver<CycleEvent>>) -> Heard {
    let Some(rx) = events else {
        return std::future::pending().await;
    };
    match rx.recv().await {
        Ok(event) => Heard::Ready(event),
        Err(broadcast::error::RecvError::Lagged(_)) => Heard::Missed,
        Err(broadcast::error::RecvError::Closed) => std::future::pending().await,
    }
}

/// Runs the rules after every poll and every cycle event until a channel closes.
pub(crate) async fn run(
    desk: Arc<RulesDesk>,
    mut polled: watch::Receiver<Arc<Vec<PollResult>>>,
    mut events: Option<broadcast::Receiver<CycleEvent>>,
) {
    loop {
        tokio::select! {
            changed = polled.changed() => {
                if changed.is_err() {
                    return;
                }
                drop(polled.borrow_and_update());
                desk.evaluate().await;
            }
            heard = next_event(&mut events) => match heard {
                Heard::Ready(event) => desk.on_event(event).await,
                Heard::Missed => desk.evaluate().await,
            }
        }
    }
}

/// Ends the countdown of a rule, if one runs, and holds that rule off. The cancel route of the
/// switch calls it before it cancels a switch that is already under way.
pub(crate) async fn cancel_countdown(accounts: &AccountsService) -> bool {
    match accounts.rules.get() {
        Some(desk) => desk.cancel().await,
        None => false,
    }
}

/// Ends the countdown of a rule for the owner's Switch now: no hold-off, no second countdown.
pub(crate) async fn end_countdown(accounts: &AccountsService) -> bool {
    match accounts.rules.get() {
        Some(desk) => desk.end().await,
        None => false,
    }
}

pub(crate) fn start(svc: &Arc<Services>, accounts: &Arc<AccountsService>) {
    let desk = RulesDesk::new(Arc::new(live::Live::new(svc, accounts)));
    let _ = accounts.rules.set(Arc::clone(&desk));
    // Shadow and preview: the desk exists (cancel answers) but decides nothing, so no countdown,
    // handover cycle or switch ever starts.
    if svc.shadow || !svc.tasks.runs(Timer::Accounts) {
        return;
    }
    let events = accounts.cycle_events();
    let task = tokio::spawn(run(desk, accounts.subscribe(), events));
    svc.tasks.keep(NAME, task);
}

#[cfg(test)]
mod tests;
