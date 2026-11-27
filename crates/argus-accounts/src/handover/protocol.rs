//! The handover cycle as a pure state machine. Each transition changes the [`Cycle`] and returns
//! the [`Effect`]s the caller carries out (a wake to a lead, a notice to the Overlord, the event
//! that every lead is ready). Nothing here reads the clock, the boards or the store.

use std::fmt;

use super::records::{
    AgentStop, Cycle, CycleMode, CyclePhase, Estimate, Handover, HandoverState, Lead, Via,
};
use crate::types::AccountKey;

/// The most points one lead may say its handover costs.
const MAX_PERCENT: f64 = 100.0;
/// The most minutes one lead may say its handover takes.
const MAX_MINUTES: f64 = 600.0;

/// Which wake a lead gets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeKind {
    /// Answer with an estimate and keep working.
    Estimate,
    /// Bring the agents to a safe point and write the handover.
    Request,
}

impl WakeKind {
    /// The wake reason of `argus-wake`.
    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            WakeKind::Estimate => "handover-estimate",
            WakeKind::Request => "handover-request",
        }
    }
}

/// The reason of the notice the Overlord gets when a lead is ready.
pub const READY_REASON: &str = "handover-ready";

/// Every asked lead is ready, or the rest were cut off: the countdown may start.
#[derive(Debug, Clone, PartialEq)]
pub struct CycleEvent {
    /// `SW-n`.
    pub cycle: String,
    /// The account being left.
    pub account: AccountKey,
    /// The use of the limit at that moment, when known.
    pub use_at_ready: Option<f64>,
    /// The handovers that were cut off; empty when every lead was ready.
    pub cut_off: Vec<String>,
}

/// What a transition asks the caller to do.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Wake a lead.
    Wake {
        /// `H-n`, the subject of the wake.
        handover: String,
        /// The board key.
        board: String,
        /// The lead's thread.
        thread: String,
        /// Which wake.
        kind: WakeKind,
    },
    /// Tell the Overlord a lead is ready (a notice, not a wake).
    Ready {
        /// `H-n`.
        handover: String,
        /// The board key.
        board: String,
        /// The lead's thread.
        thread: String,
    },
    /// The cycle reached [`CyclePhase::Ready`].
    CycleReady(CycleEvent),
}

/// Why a lead's request was refused.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// No handover of this lead in the cycle.
    NotAsked {
        /// The board key.
        board: String,
        /// The thread.
        thread: String,
    },
    /// The cycle no longer waits on its leads.
    Closed(CyclePhase),
    /// The handover is past the step asked for.
    Finished(HandoverState),
    /// The estimate or the record is not acceptable.
    Invalid(String),
}

impl ProtocolError {
    /// The HTTP status a route answers with.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            ProtocolError::NotAsked { .. } => 404,
            ProtocolError::Closed(_) | ProtocolError::Finished(_) => 409,
            ProtocolError::Invalid(_) => 400,
        }
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::NotAsked { board, thread } => write!(
                f,
                "no handover is asked of {board}/{thread}: it was idle when the cycle started"
            ),
            ProtocolError::Closed(phase) => write!(f, "the handover cycle is {phase:?}, not open"),
            ProtocolError::Finished(state) => write!(f, "this handover is already {state:?}"),
            ProtocolError::Invalid(why) => f.write_str(why),
        }
    }
}

/// What a new cycle starts from.
#[derive(Debug, Clone)]
pub struct Start {
    /// `SW-n`.
    pub id: String,
    /// The account being left.
    pub account: AccountKey,
    /// How the request is reached.
    pub mode: CycleMode,
    /// The calibration factor in use.
    pub k: f64,
    /// The ask point, when the mode fixes it.
    pub ask_at: Option<f64>,
    /// The use of the limit now, when known.
    pub use_now: Option<f64>,
    /// Now, ISO 8601.
    pub now: String,
}

/// Checks an estimate a lead sent.
///
/// # Errors
/// A percent outside 0 to 100 or minutes outside 0 to 600.
pub fn check_estimate(estimate: &Estimate) -> Result<(), ProtocolError> {
    let fits = |v: f64, max: f64| v.is_finite() && (0.0..=max).contains(&v);
    if !fits(estimate.percent, MAX_PERCENT) {
        return Err(ProtocolError::Invalid(
            "--percent must be 0 to 100 points of the limit".into(),
        ));
    }
    if !fits(estimate.minutes, MAX_MINUTES) {
        return Err(ProtocolError::Invalid("--minutes must be 0 to 600".into()));
    }
    Ok(())
}

/// The handover of a lead. A lead that is idle with a current handover file starts ready, via
/// idle, whatever step was asked for.
fn handover_of(id: String, cycle: &str, lead: &Lead, state: HandoverState, now: &str) -> Handover {
    let idle = lead.handed_over.is_some();
    let state = if idle { HandoverState::Ready } else { state };
    Handover {
        id,
        cycle: cycle.to_owned(),
        board: lead.board.clone(),
        thread: lead.thread.clone(),
        session_id: lead.session_id.clone(),
        link: lead.link.clone(),
        state,
        estimate: None,
        file: lead.handed_over.clone(),
        agents: lead.agents.clone(),
        estimate_asked_at: (state == HandoverState::EstimateAsked).then(|| now.to_owned()),
        asked_at: (state == HandoverState::Asked).then(|| now.to_owned()),
        ready_at: idle.then(|| now.to_owned()),
        via: idle.then_some(Via::Idle),
    }
}

fn wake(h: &Handover, kind: WakeKind) -> Effect {
    Effect::Wake {
        handover: h.id.clone(),
        board: h.board.clone(),
        thread: h.thread.clone(),
        kind,
    }
}

fn ready_effect(h: &Handover) -> Effect {
    Effect::Ready {
        handover: h.id.clone(),
        board: h.board.clone(),
        thread: h.thread.clone(),
    }
}

/// The effect a handover starts with: a wake for the step asked, or the notice that it is
/// ready already.
fn opening_effect(h: &Handover, kind: WakeKind) -> Effect {
    if h.state == HandoverState::Ready {
        ready_effect(h)
    } else {
        wake(h, kind)
    }
}

/// A new cycle over the leads working now, with an `H-n` from `next_id` for each. In Auto mode
/// each lead is asked for an estimate; in Fixed and Now mode each is asked for its handover.
#[must_use]
pub fn start(
    spec: Start,
    leads: &[Lead],
    mut next_id: impl FnMut() -> String,
) -> (Cycle, Vec<Effect>) {
    let leads = leads
        .iter()
        .map(|lead| {
            handover_of(
                next_id(),
                &spec.id,
                lead,
                HandoverState::EstimateAsked,
                &spec.now,
            )
        })
        .collect();
    let mut cycle = Cycle {
        id: spec.id,
        account: spec.account,
        mode: spec.mode,
        phase: CyclePhase::Estimating,
        started_at: spec.now.clone(),
        ask_at: spec.ask_at,
        k: spec.k,
        use_at_request: None,
        use_at_ready: None,
        leads,
    };
    let effects = if spec.mode == CycleMode::Auto {
        cycle
            .leads
            .iter()
            .map(|h| opening_effect(h, WakeKind::Estimate))
            .collect()
    } else {
        let notices = cycle
            .leads
            .iter()
            .filter(|h| h.state == HandoverState::Ready);
        let mut effects: Vec<Effect> = notices.map(ready_effect).collect();
        effects.extend(cycle.ask_all(spec.use_now, &spec.now));
        effects
    };
    (cycle, effects)
}

impl Cycle {
    fn open_handover(&mut self, board: &str, thread: &str) -> Result<&mut Handover, ProtocolError> {
        if !self.phase.is_open() {
            return Err(ProtocolError::Closed(self.phase));
        }
        self.leads
            .iter_mut()
            .find(|h| h.is_of(board, thread))
            .ok_or_else(|| ProtocolError::NotAsked {
                board: board.to_owned(),
                thread: thread.to_owned(),
            })
    }

    /// A lead that started working after the cycle did. In an estimating cycle it is asked for an
    /// estimate; in an asked cycle it is asked for its handover at once. A lead already in the
    /// cycle is left as it is.
    ///
    /// # Errors
    /// The cycle no longer waits on its leads.
    pub fn add_lead(
        &mut self,
        lead: &Lead,
        id: String,
        now: &str,
    ) -> Result<Vec<Effect>, ProtocolError> {
        if !self.phase.is_open() {
            return Err(ProtocolError::Closed(self.phase));
        }
        if self.handover_of(&lead.board, &lead.thread).is_some() {
            return Ok(Vec::new());
        }
        let (state, kind) = if self.phase == CyclePhase::Estimating {
            (HandoverState::EstimateAsked, WakeKind::Estimate)
        } else {
            (HandoverState::Asked, WakeKind::Request)
        };
        let h = handover_of(id, &self.id, lead, state, now);
        let mut effects = vec![opening_effect(&h, kind)];
        self.leads.push(h);
        effects.extend(self.check_ready(None));
        Ok(effects)
    }

    /// Asks a lead for a fresh estimate: it keeps its last one until it answers. Only while the
    /// cycle is estimating and the lead has not been asked for its handover.
    ///
    /// # Errors
    /// The lead is not in the cycle, the cycle is not estimating, or the handover is past the
    /// estimate.
    pub fn re_ask_estimate(
        &mut self,
        board: &str,
        thread: &str,
        now: &str,
    ) -> Result<Vec<Effect>, ProtocolError> {
        if self.phase != CyclePhase::Estimating {
            return Err(ProtocolError::Closed(self.phase));
        }
        let h = self.open_handover(board, thread)?;
        if !matches!(
            h.state,
            HandoverState::EstimateAsked | HandoverState::Estimated
        ) {
            return Err(ProtocolError::Finished(h.state));
        }
        h.state = HandoverState::EstimateAsked;
        h.estimate_asked_at = Some(now.to_owned());
        Ok(vec![wake(h, WakeKind::Estimate)])
    }

    /// A lead's estimate: kept on its handover, which moves from estimate asked to estimated.
    /// An estimate sent after the request is kept too, the state stays.
    ///
    /// # Errors
    /// The lead is not in the cycle, the cycle is closed, the handover is past its request, or
    /// the estimate is out of bounds.
    pub fn on_estimate(
        &mut self,
        board: &str,
        thread: &str,
        estimate: Estimate,
    ) -> Result<&Handover, ProtocolError> {
        check_estimate(&estimate)?;
        let h = self.open_handover(board, thread)?;
        if !h.state.is_pending() {
            return Err(ProtocolError::Finished(h.state));
        }
        if h.state == HandoverState::EstimateAsked {
            h.state = HandoverState::Estimated;
        }
        h.estimate = Some(estimate);
        Ok(h)
    }

    /// The ask point: every lead that still owes its handover is asked for it. Only an
    /// estimating cycle moves; a second call does nothing.
    pub fn ask_all(&mut self, use_now: Option<f64>, now: &str) -> Vec<Effect> {
        if self.phase != CyclePhase::Estimating {
            return Vec::new();
        }
        self.phase = CyclePhase::Asked;
        self.use_at_request = use_now;
        let mut effects = Vec::new();
        for h in &mut self.leads {
            if matches!(
                h.state,
                HandoverState::EstimateAsked | HandoverState::Estimated
            ) {
                h.state = HandoverState::Asked;
                h.asked_at = Some(now.to_owned());
                effects.push(wake(h, WakeKind::Request));
            }
        }
        effects.extend(self.check_ready(use_now));
        effects
    }

    /// A lead asked for its status: an asked handover moves to writing, since the lead has read
    /// the request. Any other state is answered as it is.
    ///
    /// # Errors
    /// The lead is not in the cycle.
    pub fn on_status(&mut self, board: &str, thread: &str) -> Result<&Handover, ProtocolError> {
        let open = self.phase.is_open();
        let h = self
            .leads
            .iter_mut()
            .find(|h| h.is_of(board, thread))
            .ok_or_else(|| ProtocolError::NotAsked {
                board: board.to_owned(),
                thread: thread.to_owned(),
            })?;
        if open && h.state == HandoverState::Asked {
            h.state = HandoverState::Writing;
        }
        Ok(h)
    }

    /// A lead wrote its handover file: its handover is ready, the Overlord gets a notice, and
    /// when it was the last one the cycle is ready. A lead that is ready already may write again
    /// to correct its file; that sends nothing.
    ///
    /// # Errors
    /// The lead is not in the cycle, the cycle is closed, or the file is empty.
    pub fn on_ready(
        &mut self,
        board: &str,
        thread: &str,
        file: &str,
        agents: Vec<AgentStop>,
        (use_now, now): (Option<f64>, &str),
    ) -> Result<(Handover, Vec<Effect>), ProtocolError> {
        if file.trim().is_empty() {
            return Err(ProtocolError::Invalid(
                "the handover file is required".into(),
            ));
        }
        let h = self.open_handover(board, thread)?;
        let first = h.state.is_pending();
        if !first && h.state != HandoverState::Ready {
            return Err(ProtocolError::Finished(h.state));
        }
        h.state = HandoverState::Ready;
        h.file = Some(file.to_owned());
        if !agents.is_empty() {
            h.agents = agents;
        }
        if first {
            h.ready_at = Some(now.to_owned());
        }
        let done = h.clone();
        let mut effects = Vec::new();
        if first {
            effects.push(ready_effect(&done));
            effects.extend(self.check_ready(use_now));
        }
        Ok((done, effects))
    }

    /// Settles a lead that still owes its handover: ready with `file`, or cut off without one.
    fn settle(
        &mut self,
        (board, thread): (&str, &str),
        (state, file, via): (HandoverState, Option<&str>, Via),
        (use_now, now): (Option<f64>, &str),
    ) -> Result<Vec<Effect>, ProtocolError> {
        let h = self.open_handover(board, thread)?;
        if !h.state.is_pending() {
            return Err(ProtocolError::Finished(h.state));
        }
        h.state = state;
        h.file = file.map(str::to_owned);
        h.via = Some(via);
        let mut effects = Vec::new();
        if state == HandoverState::Ready {
            h.ready_at = Some(now.to_owned());
            effects.push(ready_effect(h));
        }
        effects.extend(self.check_ready(use_now));
        Ok(effects)
    }

    /// The owner skips one lead's wait: with the handover `file` the lead is ready, without one
    /// it is cut off. The Overlord gets the ready notice, and the last skip readies the cycle.
    ///
    /// # Errors
    /// The lead is not in the cycle, the cycle is closed, or the lead owes nothing any more.
    pub fn skip_lead(
        &mut self,
        lead: (&str, &str),
        file: Option<&str>,
        at: (Option<f64>, &str),
    ) -> Result<Vec<Effect>, ProtocolError> {
        let state = if file.is_some() {
            HandoverState::Ready
        } else {
            HandoverState::CutOff
        };
        self.settle(lead, (state, file, Via::Owner), at)
    }

    /// A lead that went idle with a current handover file counts as ready.
    ///
    /// # Errors
    /// The lead is not in the cycle, the cycle is closed, or the lead owes nothing any more.
    pub fn confirm_idle(
        &mut self,
        lead: (&str, &str),
        file: &str,
        at: (Option<f64>, &str),
    ) -> Result<Vec<Effect>, ProtocolError> {
        self.settle(lead, (HandoverState::Ready, Some(file), Via::Idle), at)
    }

    /// Whether no lead owes its handover any more.
    #[must_use]
    pub fn all_ready(&self) -> bool {
        self.leads.iter().all(|h| !h.state.is_pending())
    }

    fn check_ready(&mut self, use_now: Option<f64>) -> Option<Effect> {
        (self.phase == CyclePhase::Asked && self.all_ready())
            .then(|| self.close(use_now, Vec::new()))
    }

    fn close(&mut self, use_now: Option<f64>, cut_off: Vec<String>) -> Effect {
        self.phase = CyclePhase::Ready;
        self.use_at_ready = use_now;
        Effect::CycleReady(CycleEvent {
            cycle: self.id.clone(),
            account: self.account.clone(),
            use_at_ready: use_now,
            cut_off,
        })
    }

    /// The switch point came first: every lead that still owes its handover is cut off and the
    /// cycle is ready. A cycle that is not open does nothing.
    pub fn cut_off_rest(&mut self, use_now: Option<f64>) -> Vec<Effect> {
        if !self.phase.is_open() {
            return Vec::new();
        }
        let mut cut = Vec::new();
        for h in self.leads.iter_mut().filter(|h| h.state.is_pending()) {
            h.state = HandoverState::CutOff;
            cut.push(h.id.clone());
        }
        vec![self.close(use_now, cut)]
    }

    /// The resumer's verdict on one handover after the switch. Once every handover has one, the
    /// cycle is done.
    ///
    /// # Errors
    /// No handover has this id, or it is not waiting for the resume.
    pub fn mark_resumed(&mut self, id: &str, resumed: bool) -> Result<&Handover, ProtocolError> {
        let Some(at) = self.leads.iter().position(|h| h.id == id) else {
            return Err(ProtocolError::Invalid(format!(
                "no handover {id} in {}",
                self.id
            )));
        };
        let state = self.leads[at].state;
        if !state.awaits_resume() {
            return Err(ProtocolError::Finished(state));
        }
        self.leads[at].state = if resumed {
            HandoverState::Resumed
        } else {
            HandoverState::NotResumed
        };
        if self.phase == CyclePhase::Ready && !self.leads.iter().any(|h| h.state.awaits_resume()) {
            self.phase = CyclePhase::Done;
        }
        Ok(&self.leads[at])
    }

    /// A switch is made from a ready cycle: it closes, so no restart can use it a second time.
    /// Returns whether it was ready.
    pub fn spend(&mut self) -> bool {
        let ready = self.phase == CyclePhase::Ready;
        if ready {
            self.phase = CyclePhase::Done;
        }
        ready
    }

    /// Calls the cycle off before the switch. Returns whether it was still running. Every lead
    /// that still owed a handover or waited to stop is closed as resumed, so
    /// `coord handover status` tells it to carry on.
    pub fn cancel(&mut self) -> bool {
        let running = self.phase.is_open() || self.phase == CyclePhase::Ready;
        if running {
            self.phase = CyclePhase::Cancelled;
            for lead in &mut self.leads {
                if lead.state.is_pending() || lead.state.awaits_resume() {
                    lead.state = HandoverState::Resumed;
                }
            }
        }
        running
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::super::records::{
        AgentStop, CycleMode, CyclePhase, Estimate, HandoverState, Lead, Via,
    };
    use super::{Cycle, Effect, ProtocolError, Start, WakeKind, start};

    const NOW: &str = "2026-11-11T10:00:00.000Z";

    fn lead(board: &str) -> Lead {
        Lead {
            board: board.into(),
            thread: "lead".into(),
            session_id: Some(format!("{board}-session")),
            link: None,
            agents: vec![AgentStop {
                id: "A-1".into(),
                branch: Some("wip".into()),
                stop: None,
            }],
            handed_over: None,
        }
    }

    fn idle_lead(board: &str) -> Lead {
        Lead {
            handed_over: Some(format!("{board}/handover.md")),
            ..lead(board)
        }
    }

    fn spec(mode: CycleMode) -> Start {
        Start {
            id: "SW-1".into(),
            account: "second".into(),
            mode,
            k: 1.0,
            ask_at: None,
            use_now: Some(96.0),
            now: NOW.into(),
        }
    }

    fn cycle(mode: CycleMode, boards: &[&str]) -> (Cycle, Vec<Effect>) {
        let leads: Vec<Lead> = boards.iter().map(|b| lead(b)).collect();
        let mut n = 0;
        start(spec(mode), &leads, || {
            n += 1;
            format!("H-{n}")
        })
    }

    fn estimate(percent: f64) -> Estimate {
        Estimate {
            percent,
            minutes: 8.0,
            agents: 1,
            note: None,
            at: NOW.into(),
        }
    }

    fn kinds(effects: &[Effect]) -> Vec<WakeKind> {
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::Wake { kind, .. } => Some(*kind),
                _ => None,
            })
            .collect()
    }

    fn ready_events(effects: &[Effect]) -> usize {
        effects
            .iter()
            .filter(|e| matches!(e, Effect::CycleReady(_)))
            .count()
    }

    #[test]
    fn auto_asks_each_working_lead_for_an_estimate() {
        let (c, effects) = cycle(CycleMode::Auto, &["rotp", "brock"]);
        assert_eq!(c.phase, CyclePhase::Estimating);
        assert_eq!(kinds(&effects), [WakeKind::Estimate, WakeKind::Estimate]);
        assert!(
            c.leads
                .iter()
                .all(|h| h.state == HandoverState::EstimateAsked)
        );
        assert_eq!(c.leads[1].id, "H-2");
        assert_eq!(c.leads[0].agents[0].branch.as_deref(), Some("wip"));
    }

    #[test]
    fn fixed_and_now_skip_the_estimate() {
        for mode in [CycleMode::Fixed, CycleMode::Now] {
            let (c, effects) = cycle(mode, &["rotp"]);
            assert_eq!(c.phase, CyclePhase::Asked);
            assert_eq!(kinds(&effects), [WakeKind::Request]);
            assert_eq!(c.use_at_request, Some(96.0));
            assert_eq!(c.leads[0].asked_at.as_deref(), Some(NOW));
        }
    }

    #[test]
    fn now_with_no_working_lead_is_ready_at_once() {
        let (c, effects) = cycle(CycleMode::Now, &[]);
        assert_eq!(c.phase, CyclePhase::Ready);
        assert_eq!(ready_events(&effects), 1);
    }

    #[test]
    fn an_estimate_moves_the_lead_and_is_kept() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp"]);
        let h = c.on_estimate("rotp", "lead", estimate(0.6)).unwrap();
        assert_eq!(h.state, HandoverState::Estimated);
        assert_eq!(h.estimate.as_ref().map(|e| e.percent), Some(0.6));
        let again = c.on_estimate("rotp", "lead", estimate(0.8)).unwrap();
        assert_eq!(again.estimate.as_ref().map(|e| e.percent), Some(0.8));
    }

    #[test]
    fn an_estimate_from_an_idle_lead_or_out_of_bounds_is_refused() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp"]);
        let err = c.on_estimate("tessera", "lead", estimate(0.3)).unwrap_err();
        assert_eq!(err.status(), 404);
        let err = c.on_estimate("rotp", "lead", estimate(140.0)).unwrap_err();
        assert!(matches!(err, ProtocolError::Invalid(_)));
        let err = c
            .on_estimate("rotp", "lead", estimate(f64::NAN))
            .unwrap_err();
        assert_eq!(err.status(), 400);
    }

    #[test]
    fn a_new_lead_in_an_estimating_cycle_gets_an_estimate_wake() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp"]);
        let effects = c.add_lead(&lead("brock"), "H-2".into(), NOW).unwrap();
        assert_eq!(kinds(&effects), [WakeKind::Estimate]);
        assert_eq!(c.leads[1].state, HandoverState::EstimateAsked);
        assert_eq!(c.leads[1].estimate_asked_at.as_deref(), Some(NOW));
        assert!(
            c.add_lead(&lead("brock"), "H-3".into(), NOW)
                .unwrap()
                .is_empty()
        );
        assert_eq!(c.leads.len(), 2, "a lead is added once");
    }

    #[test]
    fn a_new_lead_in_an_asked_cycle_gets_the_request_at_once() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp"]);
        let effects = c.add_lead(&lead("brock"), "H-2".into(), NOW).unwrap();
        assert_eq!(kinds(&effects), [WakeKind::Request]);
        assert_eq!(c.leads[1].state, HandoverState::Asked);
        assert_eq!(c.leads[1].asked_at.as_deref(), Some(NOW));
        assert!(!c.all_ready());
    }

    #[test]
    fn a_new_lead_after_the_cycle_closed_is_refused() {
        let (mut c, _) = cycle(CycleMode::Now, &[]);
        let err = c.add_lead(&lead("brock"), "H-1".into(), NOW).unwrap_err();
        assert_eq!(err.status(), 409);
    }

    #[test]
    fn re_asking_keeps_the_last_estimate_and_wakes_the_lead_again() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp"]);
        c.on_estimate("rotp", "lead", estimate(0.6)).unwrap();
        let effects = c.re_ask_estimate("rotp", "lead", NOW).unwrap();
        assert_eq!(kinds(&effects), [WakeKind::Estimate]);
        assert_eq!(c.leads[0].state, HandoverState::EstimateAsked);
        assert_eq!(c.leads[0].estimate.as_ref().map(|e| e.percent), Some(0.6));
        assert_eq!(c.leads[0].estimate_asked_at.as_deref(), Some(NOW));
    }

    #[test]
    fn re_asking_is_refused_once_the_leads_were_asked_for_the_handover() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp"]);
        assert_eq!(
            c.re_ask_estimate("tessera", "lead", NOW)
                .unwrap_err()
                .status(),
            404
        );
        c.ask_all(None, NOW);
        assert_eq!(
            c.re_ask_estimate("rotp", "lead", NOW).unwrap_err().status(),
            409
        );
    }

    #[test]
    fn ask_all_requests_every_pending_lead_once() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp", "brock"]);
        c.on_estimate("rotp", "lead", estimate(0.6)).unwrap();
        let effects = c.ask_all(Some(95.7), NOW);
        assert_eq!(kinds(&effects), [WakeKind::Request, WakeKind::Request]);
        assert_eq!(c.phase, CyclePhase::Asked);
        assert_eq!(c.use_at_request, Some(95.7));
        assert!(c.ask_all(Some(96.0), NOW).is_empty());
    }

    #[test]
    fn status_after_the_request_marks_the_lead_writing() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp"]);
        let h = c.on_status("rotp", "lead").unwrap();
        assert_eq!(h.state, HandoverState::Writing);
        let (mut early, _) = cycle(CycleMode::Auto, &["rotp"]);
        let h = early.on_status("rotp", "lead").unwrap();
        assert_eq!(h.state, HandoverState::EstimateAsked);
    }

    #[test]
    fn the_last_ready_lead_makes_the_cycle_ready() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        let (h, effects) = c
            .on_ready("rotp", "lead", "rotp.md", Vec::new(), (Some(96.0), NOW))
            .unwrap();
        assert_eq!(h.state, HandoverState::Ready);
        assert_eq!(h.agents.len(), 1, "the agents seen at the start are kept");
        assert!(matches!(effects[0], Effect::Ready { .. }));
        assert_eq!(ready_events(&effects), 0);
        assert!(!c.all_ready());
        let (_, effects) = c
            .on_ready("brock", "lead", "brock.md", Vec::new(), (Some(96.4), NOW))
            .unwrap();
        assert_eq!(ready_events(&effects), 1);
        assert_eq!(c.phase, CyclePhase::Ready);
        assert_eq!(c.use_at_ready, Some(96.4));
    }

    #[test]
    fn a_second_write_corrects_the_file_and_sends_nothing() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        c.on_ready("rotp", "lead", "a.md", Vec::new(), (None, NOW))
            .unwrap();
        let stop = AgentStop {
            id: "A-1".into(),
            branch: Some("wip".into()),
            stop: Some("tests pass".into()),
        };
        let (h, effects) = c
            .on_ready("rotp", "lead", "b.md", vec![stop], (None, NOW))
            .unwrap();
        assert!(effects.is_empty());
        assert_eq!(h.file.as_deref(), Some("b.md"));
        assert_eq!(h.agents[0].stop.as_deref(), Some("tests pass"));
    }

    #[test]
    fn a_lead_ready_before_the_ask_point_is_not_asked_again() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp", "brock"]);
        c.on_ready("rotp", "lead", "a.md", Vec::new(), (None, NOW))
            .unwrap();
        assert_eq!(
            c.phase,
            CyclePhase::Estimating,
            "ready only counts once asked"
        );
        let effects = c.ask_all(Some(95.0), NOW);
        assert_eq!(kinds(&effects), [WakeKind::Request]);
    }

    #[test]
    fn the_switch_point_cuts_off_the_rest() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        c.on_ready("rotp", "lead", "a.md", Vec::new(), (None, NOW))
            .unwrap();
        let effects = c.cut_off_rest(Some(98.0));
        let Some(Effect::CycleReady(event)) = effects.first() else {
            panic!("no event: {effects:?}");
        };
        assert_eq!(event.cut_off, ["H-2"]);
        assert_eq!(c.leads[1].state, HandoverState::CutOff);
        assert!(c.cut_off_rest(Some(99.0)).is_empty());
        let err = c
            .on_ready("brock", "lead", "b.md", Vec::new(), (None, NOW))
            .unwrap_err();
        assert_eq!(err.status(), 409);
    }

    #[test]
    fn resume_verdicts_finish_the_cycle() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        c.on_ready("rotp", "lead", "a.md", Vec::new(), (None, NOW))
            .unwrap();
        c.cut_off_rest(None);
        assert_eq!(
            c.mark_resumed("H-1", true).unwrap().state,
            HandoverState::Resumed
        );
        assert_eq!(c.phase, CyclePhase::Ready);
        c.mark_resumed("H-2", false).unwrap();
        assert_eq!(c.phase, CyclePhase::Done);
        assert!(c.mark_resumed("H-2", true).is_err());
        assert!(c.mark_resumed("H-9", true).is_err());
    }

    fn started(mode: CycleMode, leads: &[Lead]) -> (Cycle, Vec<Effect>) {
        let mut n = 0;
        start(spec(mode), leads, || {
            n += 1;
            format!("H-{n}")
        })
    }

    fn ready_notices(effects: &[Effect]) -> usize {
        effects
            .iter()
            .filter(|e| matches!(e, Effect::Ready { .. }))
            .count()
    }

    #[test]
    fn an_idle_lead_with_a_current_handover_starts_ready_via_idle_with_no_wake() {
        let (c, effects) = started(CycleMode::Fixed, &[idle_lead("rotp"), lead("brock")]);
        let idle = &c.leads[0];
        assert_eq!(idle.state, HandoverState::Ready);
        assert_eq!(idle.via, Some(Via::Idle));
        assert_eq!(idle.file.as_deref(), Some("rotp/handover.md"));
        assert_eq!(idle.ready_at.as_deref(), Some(NOW));
        assert_eq!(idle.asked_at, None);
        assert_eq!(kinds(&effects), [WakeKind::Request]);
        assert_eq!(ready_notices(&effects), 1);
        assert_eq!(c.phase, CyclePhase::Asked);
        let (auto, effects) = started(CycleMode::Auto, &[idle_lead("rotp")]);
        assert_eq!(kinds(&effects).len(), 0);
        assert_eq!(auto.leads[0].estimate_asked_at, None);
    }

    #[test]
    fn a_now_cycle_of_only_idle_leads_is_ready_at_once() {
        let (c, effects) = started(CycleMode::Now, &[idle_lead("rotp"), idle_lead("brock")]);
        assert_eq!(c.phase, CyclePhase::Ready);
        assert_eq!(ready_events(&effects), 1);
        assert_eq!(kinds(&effects).len(), 0);
    }

    #[test]
    fn an_idle_lead_added_to_an_asked_cycle_is_ready_without_a_wake() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp"]);
        let effects = c.add_lead(&idle_lead("brock"), "H-2".into(), NOW).unwrap();
        assert_eq!(kinds(&effects).len(), 0);
        assert_eq!(ready_notices(&effects), 1);
        assert_eq!(c.leads[1].via, Some(Via::Idle));
        assert_eq!(c.phase, CyclePhase::Asked);
    }

    #[test]
    fn skipping_a_lead_with_a_file_makes_it_ready_via_the_owner() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        let at = (Some(96.0), NOW);
        let effects = c.skip_lead(("rotp", "lead"), Some("rotp.md"), at).unwrap();
        let h = &c.leads[0];
        assert_eq!(h.state, HandoverState::Ready);
        assert_eq!(h.via, Some(Via::Owner));
        assert_eq!(h.file.as_deref(), Some("rotp.md"));
        assert_eq!(h.ready_at.as_deref(), Some(NOW));
        assert_eq!(ready_notices(&effects), 1);
        assert_eq!(ready_events(&effects), 0);
    }

    #[test]
    fn skipping_a_lead_without_a_file_cuts_it_off_and_the_last_skip_readies_the_cycle() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        c.skip_lead(("rotp", "lead"), Some("rotp.md"), (None, NOW))
            .unwrap();
        let effects = c
            .skip_lead(("brock", "lead"), None, (Some(97.0), NOW))
            .unwrap();
        let h = &c.leads[1];
        assert_eq!((h.state, h.via), (HandoverState::CutOff, Some(Via::Owner)));
        assert_eq!(h.file, None);
        assert_eq!(ready_notices(&effects), 0);
        assert_eq!(ready_events(&effects), 1);
        assert_eq!(c.phase, CyclePhase::Ready);
        assert_eq!(c.use_at_ready, Some(97.0));
    }

    #[test]
    fn a_skip_of_a_ready_lead_or_a_closed_cycle_is_refused_and_an_unknown_lead_is_not_found() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        let at = (None, NOW);
        c.on_ready("rotp", "lead", "a.md", Vec::new(), at).unwrap();
        let ready = c.skip_lead(("rotp", "lead"), None, at).unwrap_err();
        assert_eq!(ready, ProtocolError::Finished(HandoverState::Ready));
        assert_eq!(ready.status(), 409);
        let unknown = c.skip_lead(("tessera", "lead"), None, at).unwrap_err();
        assert_eq!(unknown.status(), 404);
        c.cut_off_rest(None);
        let closed = c.skip_lead(("brock", "lead"), None, at).unwrap_err();
        assert_eq!(closed.status(), 409);
    }

    #[test]
    fn confirming_an_idle_lead_makes_it_ready_via_idle_once() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp"]);
        let at = (Some(96.0), NOW);
        let effects = c.confirm_idle(("rotp", "lead"), "idle.md", at).unwrap();
        assert_eq!(c.leads[0].via, Some(Via::Idle));
        assert_eq!(c.leads[0].file.as_deref(), Some("idle.md"));
        assert_eq!(ready_events(&effects), 1);
        let again = c.confirm_idle(("rotp", "lead"), "idle.md", at).unwrap_err();
        assert_eq!(again.status(), 409);
    }

    #[test]
    fn spend_closes_a_ready_cycle_once() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp"]);
        assert!(!c.spend());
        c.cut_off_rest(None);
        assert!(c.spend());
        assert_eq!(c.phase, CyclePhase::Done);
        assert!(!c.spend());
    }

    #[test]
    fn cancel_stops_a_running_cycle_only() {
        let (mut c, _) = cycle(CycleMode::Auto, &["rotp"]);
        assert!(c.cancel());
        assert_eq!(c.phase, CyclePhase::Cancelled);
        assert!(!c.cancel());
        assert!(c.ask_all(None, NOW).is_empty());
    }

    #[test]
    fn cancel_closes_every_lead_of_the_cycle() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp", "brock"]);
        c.on_ready("rotp", "lead", "~/h.md", Vec::new(), (None, NOW))
            .unwrap();
        assert_eq!(c.leads[0].state, HandoverState::Ready);
        assert!(c.cancel());
        let states: Vec<_> = c.leads.iter().map(|h| h.state).collect();
        assert_eq!(states, [HandoverState::Resumed, HandoverState::Resumed]);
        let answer = c.on_status("rotp", "lead").unwrap();
        assert_eq!(answer.state, HandoverState::Resumed);
    }

    #[test]
    fn cancel_leaves_a_spent_cycle_alone() {
        let (mut c, _) = cycle(CycleMode::Now, &["rotp"]);
        c.cut_off_rest(None);
        assert!(c.spend());
        assert!(!c.cancel());
        assert_eq!(c.leads[0].state, HandoverState::CutOff);
    }
}
