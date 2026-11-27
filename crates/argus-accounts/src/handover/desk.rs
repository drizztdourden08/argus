//! The handover desk: the one open cycle, its transitions carried out over the ports. Each call
//! changes a copy of the cycle, keeps it in the store, then makes it current and sends the
//! wakes and notices the transition asked for. The rules (R9), the estimator (R11) and the
//! resumer (R12) drive it; the leads reach it through `POST /api/accounts/handover`.

use std::fmt;
use std::sync::{Arc, Mutex, PoisonError};

use argus_core::clock::iso_from_ms;

use super::estimator::AgentLoad;
use super::protocol::{CycleEvent, Effect, ProtocolError, Start, WakeKind, start};
use super::records::{AgentStop, Cycle, CycleMode, Estimate, Handover, Lead};
use super::texts::{estimate_wake, expects, ready_notice, request_wake};
use crate::config::EstimateDefaults;
use crate::ports::{Clock, HandoverStore, Threads};
use crate::types::AccountKey;

/// Why the desk refused.
#[derive(Debug, Clone, PartialEq)]
pub enum DeskError {
    /// No cycle has run yet.
    NoCycle,
    /// A cycle is still open, so another cannot start.
    Busy(String),
    /// The transition refused.
    Protocol(ProtocolError),
    /// The store refused the write; nothing changed.
    Store(String),
}

impl DeskError {
    /// The HTTP status a route answers with.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            DeskError::NoCycle => 404,
            DeskError::Busy(_) => 409,
            DeskError::Protocol(e) => e.status(),
            DeskError::Store(_) => 500,
        }
    }
}

impl fmt::Display for DeskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeskError::NoCycle => f.write_str("no handover cycle is running"),
            DeskError::Busy(id) => write!(f, "handover cycle {id} is still open"),
            DeskError::Protocol(e) => e.fmt(f),
            DeskError::Store(e) => write!(f, "the handover cycle was not saved: {e}"),
        }
    }
}

impl From<ProtocolError> for DeskError {
    fn from(e: ProtocolError) -> Self {
        DeskError::Protocol(e)
    }
}

/// What a lead asks of the desk.
#[derive(Debug, Clone, PartialEq)]
pub enum LeadAction {
    /// `coord handover estimate`.
    Estimate(Estimate),
    /// `coord handover write`: the file, and the agents when the lead listed them.
    Write {
        /// The handover file.
        file: String,
        /// The lead's agents with branch and stop point.
        agents: Vec<AgentStop>,
    },
    /// `coord handover status`.
    Status,
}

/// What the desk answers a lead.
#[derive(Debug, Clone, PartialEq)]
pub struct LeadReply {
    /// The lead's handover after the action.
    pub handover: Handover,
    /// What Argus expects next, in one word: `estimate`, `wait`, `write`, `stop` or `resume`.
    pub next: &'static str,
    /// The same, as a sentence for the lead.
    pub expects: String,
    /// Set when this action made the cycle ready.
    pub event: Option<CycleEvent>,
}

/// What a desk call that may wake leads gives back.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// The cycle after the call.
    pub cycle: Cycle,
    /// Set when the call made the cycle ready.
    pub event: Option<CycleEvent>,
    /// The wakes and notices that could not be sent, one line each.
    pub failures: Vec<String>,
}

/// A new cycle as the rules or the Switch now button ask for it.
#[derive(Debug, Clone)]
pub struct StartRequest {
    /// The account being left.
    pub account: AccountKey,
    /// How the request is reached.
    pub mode: CycleMode,
    /// The calibration factor in use.
    pub k: f64,
    /// The ask point, when the mode fixes it.
    pub ask_at: Option<f64>,
    /// The use of the account's binding limit now.
    pub use_now: Option<f64>,
}

/// The desk over its ports.
pub struct Desk {
    threads: Arc<dyn Threads>,
    store: Arc<dyn HandoverStore>,
    clock: Arc<dyn Clock>,
    cycle: Mutex<Option<Cycle>>,
}

impl Desk {
    /// A desk with the last cycle of the store.
    ///
    /// # Errors
    /// The store cannot be read.
    pub fn new(
        threads: Arc<dyn Threads>,
        store: Arc<dyn HandoverStore>,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, String> {
        let cycle = store.load()?;
        Ok(Desk {
            threads,
            store,
            clock,
            cycle: Mutex::new(cycle),
        })
    }

    /// The current or last cycle.
    #[must_use]
    pub fn cycle(&self) -> Option<Cycle> {
        self.cycle
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn now(&self) -> String {
        iso_from_ms(self.clock.now_ms())
    }

    fn next_id(&self, counter: &str) -> Result<String, DeskError> {
        let n = self.store.next_number(counter).map_err(DeskError::Store)?;
        Ok(format!("{counter}-{n}"))
    }

    /// Starts a cycle over the leads working now. Refused while another cycle is open.
    ///
    /// # Errors
    /// A cycle is open, or the store refused.
    pub fn start(
        &self,
        request: StartRequest,
        defaults: &EstimateDefaults,
    ) -> Result<Outcome, DeskError> {
        let mut slot = self.cycle.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(open) = slot.as_ref().filter(|c| c.phase.is_open()) {
            return Err(DeskError::Busy(open.id.clone()));
        }
        let leads = self.threads.working_leads();
        let id = self.next_id("SW")?;
        let ids = leads
            .iter()
            .map(|_| self.next_id("H"))
            .collect::<Result<Vec<_>, _>>()?;
        let spec = Start {
            id,
            account: request.account,
            mode: request.mode,
            k: request.k,
            ask_at: request.ask_at,
            use_now: request.use_now,
            now: self.now(),
        };
        let mut ids = ids.into_iter();
        let (cycle, effects) = start(spec, &leads, || ids.next().unwrap_or_default());
        self.store.save(&cycle).map_err(DeskError::Store)?;
        *slot = Some(cycle.clone());
        Ok(self.carry_out(cycle, effects, defaults))
    }

    /// Runs `change` on a copy of the cycle, keeps the copy, then makes it current.
    fn change<T>(
        &self,
        change: impl FnOnce(&mut Cycle) -> Result<T, DeskError>,
    ) -> Result<(Cycle, T), DeskError> {
        let mut slot = self.cycle.lock().unwrap_or_else(PoisonError::into_inner);
        let mut next = slot.clone().ok_or(DeskError::NoCycle)?;
        let value = change(&mut next)?;
        if slot.as_ref() != Some(&next) {
            self.store.save(&next).map_err(DeskError::Store)?;
            *slot = Some(next.clone());
        }
        Ok((next, value))
    }

    /// The ask point: every lead that owes its handover gets the handover-request wake.
    ///
    /// # Errors
    /// No cycle, or the store refused.
    pub fn ask_all(&self, use_now: Option<f64>) -> Result<Outcome, DeskError> {
        let now = self.now();
        let (cycle, effects) = self.change(|c| Ok(c.ask_all(use_now, &now)))?;
        Ok(self.carry_out(cycle, effects, &EstimateDefaults::default()))
    }

    /// The switch point came first: the leads that are not ready are cut off.
    ///
    /// # Errors
    /// No cycle, or the store refused.
    pub fn cut_off_rest(&self, use_now: Option<f64>) -> Result<Outcome, DeskError> {
        let (cycle, effects) = self.change(|c| Ok(c.cut_off_rest(use_now)))?;
        Ok(self.carry_out(cycle, effects, &EstimateDefaults::default()))
    }

    /// The leads working now, as the boards say.
    #[must_use]
    pub fn working_leads(&self) -> Vec<Lead> {
        self.threads.working_leads()
    }

    /// The live agents of a lead with their model and context size.
    #[must_use]
    pub fn agent_loads(&self, board: &str, thread: &str) -> Vec<AgentLoad> {
        self.threads.agent_loads(board, thread)
    }

    /// The display name of the project a board belongs to, when the boards know it.
    #[must_use]
    pub fn project_name(&self, board: &str) -> Option<String> {
        self.threads.project_name(board)
    }

    /// Asks a lead of the estimating cycle for a fresh estimate.
    ///
    /// # Errors
    /// No cycle, the lead is not in it, the cycle is not estimating, or the store refused.
    pub fn re_ask_estimate(
        &self,
        (board, thread): (&str, &str),
        defaults: &EstimateDefaults,
    ) -> Result<Outcome, DeskError> {
        let now = self.now();
        let (cycle, effects) = self.change(|c| Ok(c.re_ask_estimate(board, thread, &now)?))?;
        Ok(self.carry_out(cycle, effects, defaults))
    }

    /// Adds a lead that started working after the cycle did: it gets an estimate wake in an
    /// estimating cycle and the handover request at once in an asked one.
    ///
    /// # Errors
    /// No cycle, the cycle is closed, or the store refused.
    pub fn add_lead(&self, lead: &Lead, defaults: &EstimateDefaults) -> Result<Outcome, DeskError> {
        let now = self.now();
        let id = self.next_id("H")?;
        let (cycle, effects) = self.change(|c| Ok(c.add_lead(lead, id, &now)?))?;
        Ok(self.carry_out(cycle, effects, defaults))
    }

    /// The owner skips one lead's wait: ready with `file`, or cut off without one. The Overlord
    /// gets the ready notice once, and the last skip makes the cycle ready.
    ///
    /// # Errors
    /// No cycle, the lead is not in it, it owes nothing any more, or the store refused.
    pub fn skip_lead(
        &self,
        lead: (&str, &str),
        file: Option<&str>,
        use_now: Option<f64>,
    ) -> Result<Outcome, DeskError> {
        let now = self.now();
        let (cycle, effects) = self.change(|c| Ok(c.skip_lead(lead, file, (use_now, &now))?))?;
        Ok(self.carry_out(cycle, effects, &EstimateDefaults::default()))
    }

    /// A pending lead that is idle with a current handover `file` counts as ready.
    ///
    /// # Errors
    /// No cycle, the lead is not in it, it owes nothing any more, or the store refused.
    pub fn confirm_idle(
        &self,
        lead: (&str, &str),
        file: &str,
        use_now: Option<f64>,
    ) -> Result<Outcome, DeskError> {
        let now = self.now();
        let (cycle, effects) = self.change(|c| Ok(c.confirm_idle(lead, file, (use_now, &now))?))?;
        Ok(self.carry_out(cycle, effects, &EstimateDefaults::default()))
    }

    /// The resumer's verdict on one handover.
    ///
    /// # Errors
    /// No cycle, no such handover, it is not waiting for the resume, or the store refused.
    pub fn mark_resumed(&self, id: &str, resumed: bool) -> Result<Handover, DeskError> {
        let (_, h) = self.change(|c| Ok(c.mark_resumed(id, resumed)?.clone()))?;
        Ok(h)
    }

    /// Calls the cycle off. Returns whether it was still running.
    ///
    /// # Errors
    /// No cycle, or the store refused.
    pub fn cancel(&self) -> Result<bool, DeskError> {
        let (_, running) = self.change(|c| Ok(c.cancel()))?;
        Ok(running)
    }

    /// A lead's estimate, its handover written, or its question what comes next.
    ///
    /// # Errors
    /// No cycle, the lead is not in it, the step is refused, or the store refused.
    pub fn lead(
        &self,
        (board, thread): (&str, &str),
        action: LeadAction,
        use_now: Option<f64>,
        defaults: &EstimateDefaults,
    ) -> Result<LeadReply, DeskError> {
        let now = self.now();
        let (cycle, (handover, effects)) = self.change(|c| {
            Ok(match action {
                LeadAction::Estimate(estimate) => {
                    (c.on_estimate(board, thread, estimate)?.clone(), Vec::new())
                }
                LeadAction::Status => (c.on_status(board, thread)?.clone(), Vec::new()),
                LeadAction::Write { file, agents } => {
                    c.on_ready(board, thread, &file, agents, (use_now, &now))?
                }
            })
        })?;
        let outcome = self.carry_out(cycle, effects, defaults);
        let (next, expects) = expects(&handover, defaults);
        Ok(LeadReply {
            handover,
            next,
            expects,
            event: outcome.event,
        })
    }

    fn carry_out(
        &self,
        cycle: Cycle,
        effects: Vec<Effect>,
        defaults: &EstimateDefaults,
    ) -> Outcome {
        let mut event = None;
        let mut failures = Vec::new();
        for effect in effects {
            let sent = match effect {
                Effect::Wake {
                    handover,
                    board,
                    thread,
                    kind,
                } => {
                    let text = match kind {
                        WakeKind::Estimate => estimate_wake(defaults),
                        WakeKind::Request => request_wake(&board),
                    };
                    self.threads
                        .wake((&board, &thread), kind, &handover, &text)
                        .map_err(|e| format!("{board}/{thread}: {e}"))
                }
                Effect::Ready { handover, .. } => cycle
                    .leads
                    .iter()
                    .find(|h| h.id == handover)
                    .map_or(Ok(()), |h| self.threads.notice(&h.id, &ready_notice(h)))
                    .map_err(|e| format!("Overlord notice for {handover}: {e}")),
                Effect::CycleReady(ready) => {
                    event = Some(ready);
                    Ok(())
                }
            };
            if let Err(line) = sent {
                failures.push(line);
            }
        }
        Outcome {
            cycle,
            event,
            failures,
        }
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use super::{Desk, DeskError, LeadAction, StartRequest};
    use crate::config::EstimateDefaults;
    use crate::handover::{
        AgentStop, Cycle, CycleMode, CyclePhase, HandoverState, Lead, Via, WakeKind,
    };
    use crate::ports::{Clock, HandoverStore, Threads};

    #[derive(Default)]
    struct World {
        leads: Vec<Lead>,
        wakes: Mutex<Vec<(String, WakeKind, String)>>,
        notices: Mutex<Vec<String>>,
        counters: Mutex<HashMap<String, u64>>,
        saved: Mutex<Option<Cycle>>,
    }

    impl Threads for World {
        fn working_leads(&self) -> Vec<Lead> {
            self.leads.clone()
        }
        fn wake(
            &self,
            (board, _): (&str, &str),
            kind: WakeKind,
            subject: &str,
            _: &str,
        ) -> Result<(), String> {
            self.wakes
                .lock()
                .unwrap()
                .push((board.to_owned(), kind, subject.to_owned()));
            Ok(())
        }
        fn notice(&self, subject: &str, _: &str) -> Result<(), String> {
            self.notices.lock().unwrap().push(subject.to_owned());
            Ok(())
        }
    }

    impl HandoverStore for World {
        fn next_number(&self, counter: &str) -> Result<u64, String> {
            let mut all = self.counters.lock().unwrap();
            let n = all.entry(counter.to_owned()).or_default();
            *n += 1;
            Ok(*n)
        }
        fn load(&self) -> Result<Option<Cycle>, String> {
            Ok(self.saved.lock().unwrap().clone())
        }
        fn save(&self, cycle: &Cycle) -> Result<(), String> {
            *self.saved.lock().unwrap() = Some(cycle.clone());
            Ok(())
        }
    }

    impl Clock for World {
        fn now_ms(&self) -> i64 {
            1_762_855_200_000
        }
    }

    fn lead(board: &str) -> Lead {
        Lead {
            board: board.into(),
            thread: "lead".into(),
            session_id: None,
            link: None,
            agents: Vec::new(),
            handed_over: None,
        }
    }

    fn desk(boards: &[&str]) -> (Arc<World>, Desk) {
        let world = Arc::new(World {
            leads: boards.iter().map(|b| lead(b)).collect(),
            ..World::default()
        });
        let desk = Desk::new(world.clone(), world.clone(), world.clone()).unwrap();
        (world, desk)
    }

    fn request(mode: CycleMode) -> StartRequest {
        StartRequest {
            account: "second".into(),
            mode,
            k: 1.0,
            ask_at: None,
            use_now: Some(95.0),
        }
    }

    #[test]
    fn a_cycle_numbers_itself_and_its_handovers_and_wakes_each_lead() {
        let (world, desk) = desk(&["rotp", "portfolio"]);
        let out = desk
            .start(request(CycleMode::Auto), &EstimateDefaults::default())
            .unwrap();
        assert_eq!(out.cycle.id, "SW-1");
        assert_eq!(out.cycle.leads[1].id, "H-2");
        let wakes = world.wakes.lock().unwrap().clone();
        assert_eq!(wakes[0], ("rotp".into(), WakeKind::Estimate, "H-1".into()));
        assert_eq!(world.saved.lock().unwrap().as_ref(), Some(&out.cycle));
    }

    #[test]
    fn a_second_cycle_waits_for_the_first() {
        let (_, desk) = desk(&["rotp"]);
        desk.start(request(CycleMode::Auto), &EstimateDefaults::default())
            .unwrap();
        let err = desk
            .start(request(CycleMode::Now), &EstimateDefaults::default())
            .unwrap_err();
        assert_eq!(err, DeskError::Busy("SW-1".into()));
        desk.cancel().unwrap();
        let next = desk
            .start(request(CycleMode::Now), &EstimateDefaults::default())
            .unwrap();
        assert_eq!(next.cycle.id, "SW-2");
    }

    #[test]
    fn the_last_write_notices_the_overlord_and_reports_the_event() {
        let (world, desk) = desk(&["rotp"]);
        desk.start(request(CycleMode::Now), &EstimateDefaults::default())
            .unwrap();
        let status = desk
            .lead(
                ("rotp", "lead"),
                LeadAction::Status,
                None,
                &EstimateDefaults::default(),
            )
            .unwrap();
        assert_eq!(status.next, "write");
        assert_eq!(status.handover.state, HandoverState::Writing);
        let write = LeadAction::Write {
            file: "C:/Users/x/.coord/rotp/handover.md".into(),
            agents: vec![AgentStop {
                id: "A-3".into(),
                branch: None,
                stop: None,
            }],
        };
        let reply = desk
            .lead(
                ("rotp", "lead"),
                write,
                Some(96.0),
                &EstimateDefaults::default(),
            )
            .unwrap();
        assert_eq!(reply.next, "stop");
        assert_eq!(reply.event.map(|e| e.cycle), Some("SW-1".into()));
        assert_eq!(world.notices.lock().unwrap().clone(), ["H-1"]);
        assert_eq!(desk.cycle().map(|c| c.phase), Some(CyclePhase::Ready));
    }

    #[test]
    fn no_cycle_answers_not_found() {
        let (_, desk) = desk(&[]);
        let err = desk
            .lead(
                ("rotp", "lead"),
                LeadAction::Status,
                None,
                &EstimateDefaults::default(),
            )
            .unwrap_err();
        assert_eq!(err.status(), 404);
    }

    #[test]
    fn a_restarted_desk_keeps_the_cycle() {
        let (world, desk) = desk(&["rotp"]);
        desk.start(request(CycleMode::Auto), &EstimateDefaults::default())
            .unwrap();
        let again = Desk::new(world.clone(), world.clone(), world).unwrap();
        assert_eq!(again.cycle().map(|c| c.id), Some("SW-1".into()));
    }

    #[test]
    fn a_skipped_lead_notices_the_overlord_once_and_the_cycle_is_saved() {
        let (world, desk) = desk(&["rotp", "brock"]);
        let defaults = EstimateDefaults::default();
        desk.start(request(CycleMode::Now), &defaults).unwrap();
        let first = desk
            .skip_lead(("rotp", "lead"), Some("rotp.md"), Some(96.0))
            .unwrap();
        assert_eq!(first.event, None);
        assert_eq!(world.notices.lock().unwrap().clone(), ["H-1"]);
        let saved = world.saved.lock().unwrap().clone().unwrap();
        assert_eq!(saved.leads[0].via, Some(Via::Owner));
        let last = desk.skip_lead(("brock", "lead"), None, Some(97.0)).unwrap();
        assert_eq!(last.event.map(|e| e.cycle), Some("SW-1".into()));
        assert_eq!(world.notices.lock().unwrap().len(), 1);
        assert_eq!(desk.cycle().map(|c| c.phase), Some(CyclePhase::Ready));
        let again = desk.skip_lead(("brock", "lead"), None, None).unwrap_err();
        assert_eq!(again.status(), 409);
    }

    #[test]
    fn a_confirmed_idle_lead_is_saved_ready_via_idle() {
        let (world, desk) = desk(&["rotp"]);
        desk.start(request(CycleMode::Now), &EstimateDefaults::default())
            .unwrap();
        let out = desk.confirm_idle(("rotp", "lead"), "idle.md", None).unwrap();
        assert_eq!(out.cycle.leads[0].via, Some(Via::Idle));
        assert_eq!(world.notices.lock().unwrap().clone(), ["H-1"]);
        assert_eq!(world.saved.lock().unwrap().as_ref(), Some(&out.cycle));
    }

    #[test]
    fn a_re_asked_lead_and_a_lead_added_mid_cycle_are_woken() {
        let (world, desk) = desk(&["rotp"]);
        let defaults = EstimateDefaults::default();
        desk.start(request(CycleMode::Auto), &defaults).unwrap();
        let added = desk.add_lead(&lead("brock"), &defaults).unwrap();
        assert_eq!(added.cycle.leads.len(), 2);
        desk.re_ask_estimate(("rotp", "lead"), &defaults).unwrap();
        let wakes = world.wakes.lock().unwrap();
        assert_eq!(wakes.len(), 3);
        assert!(wakes.iter().all(|(_, kind, _)| *kind == WakeKind::Estimate));
    }
}
