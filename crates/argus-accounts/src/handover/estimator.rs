//! Where the handover request goes: how many points the handovers will cost, the ask point that
//! follows from it, which leads are to be asked for an estimate, and how the calibration factor
//! learns from a finished cycle. Pure: the clock, the boards and the config are inputs.
//!
//! ```text
//! need   = k x sum(estimates) + burn x T + safety
//! T      = longest minutes + 2
//! ask at = switch point - need, kept within 30 points below the switch point
//! ```

use argus_core::clock::parse_ms;

use super::records::{Cycle, CyclePhase, Estimate, Handover, HandoverState};
use crate::config::EstimateDefaults;

/// An estimate older than this is replaced by Argus's own prior.
pub const FRESH_MS: i64 = 20 * 60_000;
/// A lead is never asked for an estimate more often than this.
pub const MIN_ASK_GAP_MS: i64 = 20 * 60_000;
/// Estimates are asked for once the use is this close to the switch point.
const NEAR_POINTS: f64 = 10.0;
/// Or once the burn reaches the switch point within this many minutes.
const NEAR_MINUTES: f64 = 45.0;
/// Minutes added to the longest estimate for writing the files.
const WRITE_MINUTES: f64 = 2.0;
/// The ask point is never further below the switch point than this.
const MAX_LEAD_POINTS: f64 = 30.0;
/// Minutes a lead's handover takes before its agents are counted.
const PRIOR_BASE_MINUTES: f64 = 3.0;
/// Minutes each live agent adds to the prior.
const PRIOR_AGENT_MINUTES: f64 = 1.0;
/// A context over this many tokens costs more to hand over.
pub const LONG_CONTEXT_TOKENS: u64 = 100_000;
/// How far one cycle moves `k` toward what it measured.
const LEARNING_RATE: f64 = 0.25;
/// The range `k` stays in.
const K_RANGE: (f64, f64) = (0.5, 3.0);

/// One live agent of a lead, as far as the estimate cares.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentLoad {
    /// The model name the agent record carries.
    pub model: Option<String>,
    /// The tokens in the agent's context, when they are known.
    pub context_tokens: Option<u64>,
}

/// A working lead and what is known about its estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct LeadInput {
    /// The board key.
    pub board: String,
    /// The lead's thread.
    pub thread: String,
    /// Its live agents.
    pub agents: Vec<AgentLoad>,
    /// Its last estimate.
    pub estimate: Option<Estimate>,
    /// When it was last asked for an estimate, epoch milliseconds.
    pub asked_at_ms: Option<i64>,
}

impl LeadInput {
    /// The lead of a handover already in a cycle.
    #[must_use]
    pub fn of_handover(h: &Handover, agents: Vec<AgentLoad>) -> Self {
        LeadInput {
            board: h.board.clone(),
            thread: h.thread.clone(),
            agents,
            estimate: h.estimate.clone(),
            asked_at_ms: h.estimate_asked_at.as_deref().and_then(parse_ms),
        }
    }
}

/// What one lead is counted at.
#[derive(Debug, Clone, PartialEq)]
pub struct LeadEstimate {
    /// The board key.
    pub board: String,
    /// The lead's thread.
    pub thread: String,
    /// Points of the limit its handover costs.
    pub percent: f64,
    /// Minutes until its handover file is written.
    pub minutes: f64,
    /// Whether the lead's own fresh estimate was used, not Argus's prior.
    pub fresh: bool,
}

fn default_of(model: Option<&str>, d: &EstimateDefaults) -> f64 {
    let name = model.unwrap_or_default().to_lowercase();
    if name.contains("opus") {
        d.opus
    } else if name.contains("haiku") {
        d.haiku
    } else {
        d.sonnet
    }
}

/// Argus's own estimate of a lead: its own handover plus each live agent by model, scaled up for
/// an agent whose context is known to be long. An unknown context takes no factor.
#[must_use]
pub fn prior(agents: &[AgentLoad], d: &EstimateDefaults) -> (f64, f64) {
    let agent_points: f64 = agents
        .iter()
        .map(|a| {
            let long = a.context_tokens.is_some_and(|t| t > LONG_CONTEXT_TOKENS);
            let factor = if long { d.long_context_factor } else { 1.0 };
            default_of(a.model.as_deref(), d) * factor
        })
        .sum();
    let count = f64::from(u32::try_from(agents.len()).unwrap_or(u32::MAX));
    let minutes = PRIOR_BASE_MINUTES + PRIOR_AGENT_MINUTES * count;
    (d.lead + agent_points, minutes)
}

fn is_fresh(estimate: &Estimate, now_ms: i64) -> bool {
    parse_ms(&estimate.at).is_some_and(|at| now_ms - at <= FRESH_MS)
}

/// What a lead is counted at: its own estimate when fresh, Argus's prior otherwise.
#[must_use]
pub fn count_lead(lead: &LeadInput, d: &EstimateDefaults, now_ms: i64) -> LeadEstimate {
    let own = lead.estimate.as_ref().filter(|e| is_fresh(e, now_ms));
    let (percent, minutes) = own.map_or_else(|| prior(&lead.agents, d), |e| (e.percent, e.minutes));
    LeadEstimate {
        board: lead.board.clone(),
        thread: lead.thread.clone(),
        percent,
        minutes,
        fresh: own.is_some(),
    }
}

/// The points the handovers will use: `k x sum(estimates) + burn x T + safety`, with `T` the
/// longest estimate plus the time to write the files. A burn that is unknown or negative counts
/// as none.
#[must_use]
pub fn need(leads: &[LeadEstimate], k: f64, burn: Option<f64>, safety: f64) -> f64 {
    let sum: f64 = leads.iter().map(|l| l.percent).sum();
    let longest = leads.iter().map(|l| l.minutes).fold(0.0, f64::max);
    let burn = burn.filter(|b| b.is_finite()).unwrap_or(0.0).max(0.0);
    k * sum + burn * (longest + WRITE_MINUTES) + safety
}

/// The use at which the leads are asked for their handover.
#[must_use]
pub fn ask_at(switch_at: f64, need: f64) -> f64 {
    (switch_at - need).clamp(switch_at - MAX_LEAD_POINTS, switch_at)
}

/// Whether estimates are worth asking for: the use is within 10 points of the switch point, or
/// the burn reaches it within 45 minutes.
#[must_use]
pub fn near_switch(use_now: f64, switch_at: f64, burn: Option<f64>) -> bool {
    let left = switch_at - use_now;
    if left <= NEAR_POINTS {
        return true;
    }
    burn.is_some_and(|b| b > 0.0 && left / b <= NEAR_MINUTES)
}

/// Whether this lead is to be asked for an estimate now: never inside the gap since the last ask;
/// otherwise when it has none, its agent count changed, or its estimate is no longer fresh.
#[must_use]
pub fn is_due(lead: &LeadInput, now_ms: i64) -> bool {
    if lead
        .asked_at_ms
        .is_some_and(|at| now_ms - at < MIN_ASK_GAP_MS)
    {
        return false;
    }
    let Some(estimate) = &lead.estimate else {
        return true;
    };
    let count_changed = usize::try_from(estimate.agents) != Ok(lead.agents.len());
    count_changed || !is_fresh(estimate, now_ms)
}

/// The leads to ask for an estimate: none until the account is near its switch point.
#[must_use]
pub fn leads_to_ask(leads: &[LeadInput], near: bool, now_ms: i64) -> Vec<&LeadInput> {
    if !near {
        return Vec::new();
    }
    leads.iter().filter(|l| is_due(l, now_ms)).collect()
}

/// What a finished cycle cost against what its leads estimated.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CycleCost {
    /// Points of the limit used between the request and the last lead being ready.
    pub actual: f64,
    /// The sum of the leads' own estimates, before `k`.
    pub estimated: f64,
}

/// The cost of a cycle, when it can teach anything: every lead was ready (none cut off), both
/// uses were read, and the leads estimated something.
#[must_use]
pub fn cost_of(cycle: &Cycle) -> Option<CycleCost> {
    if cycle.phase != CyclePhase::Ready && cycle.phase != CyclePhase::Done {
        return None;
    }
    let asked: Vec<&Handover> = cycle
        .leads
        .iter()
        .filter(|h| h.asked_at.is_some() || h.ready_at.is_some())
        .collect();
    let all_ready = !asked.is_empty()
        && asked.iter().all(|h| {
            matches!(
                h.state,
                HandoverState::Ready | HandoverState::Resumed | HandoverState::NotResumed
            )
        });
    if !all_ready {
        return None;
    }
    let estimated: f64 = asked
        .iter()
        .filter_map(|h| h.estimate.as_ref())
        .map(|e| e.percent)
        .sum();
    let actual = cycle.use_at_ready? - cycle.use_at_request?;
    (estimated > 0.0 && actual >= 0.0).then_some(CycleCost { actual, estimated })
}

/// The calibration factor after a cycle: a quarter of the way toward `actual / estimated`, kept
/// within 0.5 to 3.
#[must_use]
pub fn calibrate(k: f64, cost: CycleCost) -> f64 {
    let target = cost.actual / cost.estimated;
    (k + LEARNING_RATE * (target - k)).clamp(K_RANGE.0, K_RANGE.1)
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::super::records::{Cycle, CycleMode, CyclePhase, Estimate, Handover, HandoverState};
    use super::{
        AgentLoad, CycleCost, LeadEstimate, LeadInput, ask_at, calibrate, cost_of, count_lead,
        is_due, leads_to_ask, near_switch, need, prior,
    };
    use crate::config::EstimateDefaults;

    const NOW: i64 = 1_762_855_200_000;
    const MIN: i64 = 60_000;

    fn iso_ago(minutes: i64) -> String {
        argus_core::clock::iso_from_ms(NOW - minutes * MIN)
    }

    fn est(percent: f64, minutes: f64) -> LeadEstimate {
        LeadEstimate {
            board: "b".into(),
            thread: "lead".into(),
            percent,
            minutes,
            fresh: true,
        }
    }

    fn estimate(percent: f64, agents: u32, minutes_ago: i64) -> Estimate {
        Estimate {
            percent,
            minutes: 8.0,
            agents,
            note: None,
            at: iso_ago(minutes_ago),
        }
    }

    fn agent(model: &str, tokens: Option<u64>) -> AgentLoad {
        AgentLoad {
            model: Some(model.into()),
            context_tokens: tokens,
        }
    }

    fn lead(estimate: Option<Estimate>, agents: usize, asked_ago: Option<i64>) -> LeadInput {
        LeadInput {
            board: "rotp".into(),
            thread: "lead".into(),
            agents: vec![AgentLoad::default(); agents],
            estimate,
            asked_at_ms: asked_ago.map(|m| NOW - m * MIN),
        }
    }

    #[test]
    fn the_plan_example_asks_at_95_point_7() {
        let leads = [est(0.6, 3.0), est(0.4, 8.0), est(0.3, 5.0)];
        let n = need(&leads, 1.0, Some(0.05), 0.5);
        assert!((n - 2.3).abs() < 1e-9, "{n}");
        assert!((ask_at(98.0, n) - 95.7).abs() < 1e-9);
    }

    #[test]
    fn k_scales_the_estimates_only() {
        let n = need(&[est(1.0, 4.0)], 2.0, Some(0.1), 0.5);
        assert!((n - (2.0 + 0.6 + 0.5)).abs() < 1e-9);
    }

    #[test]
    fn no_leads_and_no_burn_leaves_the_safety_margin() {
        assert!((need(&[], 1.0, None, 0.5) - 0.5).abs() < 1e-9);
        assert!((need(&[est(1.0, 4.0)], 1.0, Some(-1.0), 0.5) - 1.5).abs() < 1e-9);
    }

    #[test]
    fn the_ask_point_stays_within_30_points_below_the_switch_point() {
        assert!((ask_at(98.0, 80.0) - 68.0).abs() < 1e-9);
        assert!((ask_at(98.0, -4.0) - 98.0).abs() < 1e-9);
    }

    #[test]
    fn the_prior_counts_the_lead_and_each_agent_by_model() {
        let d = EstimateDefaults::default();
        let (percent, minutes) = prior(&[agent("claude-opus-4", None), agent("haiku", None)], &d);
        assert!((percent - (0.4 + 0.3 + 0.03)).abs() < 1e-9);
        assert!((minutes - 5.0).abs() < 1e-9);
        let (alone, _) = prior(&[], &d);
        assert!((alone - 0.4).abs() < 1e-9);
    }

    #[test]
    fn a_known_long_context_scales_an_agent_and_an_unknown_one_does_not() {
        let d = EstimateDefaults::default();
        let (long, _) = prior(&[agent("sonnet", Some(150_000))], &d);
        let (short, _) = prior(&[agent("sonnet", Some(100_000))], &d);
        let (unknown, _) = prior(&[agent("sonnet", None)], &d);
        assert!((long - (0.4 + 0.1 * 1.5)).abs() < 1e-9);
        assert!((short - 0.5).abs() < 1e-9);
        assert!((unknown - 0.5).abs() < 1e-9);
    }

    #[test]
    fn a_fresh_estimate_is_used_and_a_stale_or_missing_one_gives_the_prior() {
        let d = EstimateDefaults::default();
        let fresh = count_lead(&lead(Some(estimate(0.6, 0, 5)), 0, None), &d, NOW);
        assert!(fresh.fresh);
        assert!((fresh.percent - 0.6).abs() < 1e-9);
        for stale in [
            lead(Some(estimate(0.6, 0, 25)), 0, None),
            lead(None, 0, None),
        ] {
            let counted = count_lead(&stale, &d, NOW);
            assert!(!counted.fresh);
            assert!((counted.percent - 0.4).abs() < 1e-9);
        }
    }

    #[test]
    fn estimates_are_asked_for_within_10_points_or_when_the_burn_gets_there_in_45_minutes() {
        assert!(near_switch(88.0, 98.0, None));
        assert!(!near_switch(87.9, 98.0, None));
        assert!(
            near_switch(80.0, 98.0, Some(0.4)),
            "18 points at 0.4 is 45 minutes"
        );
        assert!(!near_switch(80.0, 98.0, Some(0.3)));
        assert!(!near_switch(50.0, 98.0, Some(0.0)));
    }

    #[test]
    fn a_lead_without_an_estimate_is_asked_unless_it_was_asked_lately() {
        assert!(is_due(&lead(None, 2, None), NOW));
        assert!(is_due(&lead(None, 2, Some(21)), NOW));
        assert!(!is_due(&lead(None, 2, Some(19)), NOW));
    }

    #[test]
    fn a_lead_is_asked_again_when_its_agent_count_changed_or_its_estimate_is_old() {
        assert!(!is_due(&lead(Some(estimate(0.5, 2, 5)), 2, Some(5)), NOW));
        assert!(is_due(&lead(Some(estimate(0.5, 2, 25)), 2, Some(25)), NOW));
        assert!(
            !is_due(&lead(Some(estimate(0.5, 2, 5)), 3, Some(5)), NOW),
            "inside the gap"
        );
        assert!(is_due(&lead(Some(estimate(0.5, 2, 5)), 3, Some(30)), NOW));
        assert!(is_due(&lead(Some(estimate(0.5, 2, 30)), 2, None), NOW));
    }

    #[test]
    fn nobody_is_asked_while_the_account_is_far_from_its_switch_point() {
        let leads = [lead(None, 1, None)];
        assert!(leads_to_ask(&leads, false, NOW).is_empty());
        assert_eq!(leads_to_ask(&leads, true, NOW).len(), 1);
    }

    #[test]
    fn k_moves_a_quarter_of_the_way_and_stays_in_range() {
        let k = calibrate(
            1.0,
            CycleCost {
                actual: 1.8,
                estimated: 1.2,
            },
        );
        assert!((k - 1.125).abs() < 1e-9, "{k}");
        assert!(
            (calibrate(
                1.0,
                CycleCost {
                    actual: 0.0,
                    estimated: 1.0
                }
            ) - 0.75)
                .abs()
                < 1e-9
        );
        let high = calibrate(
            3.0,
            CycleCost {
                actual: 50.0,
                estimated: 1.0,
            },
        );
        assert!((high - 3.0).abs() < 1e-9);
        let low = calibrate(
            0.5,
            CycleCost {
                actual: 0.0,
                estimated: 1.0,
            },
        );
        assert!((low - 0.5).abs() < 1e-9);
    }

    fn handover(state: HandoverState, percent: Option<f64>) -> Handover {
        Handover {
            id: "H-1".into(),
            cycle: "SW-1".into(),
            board: "rotp".into(),
            thread: "lead".into(),
            session_id: None,
            link: None,
            state,
            estimate: percent.map(|p| estimate(p, 0, 1)),
            file: None,
            agents: Vec::new(),
            asked_at: Some(iso_ago(5)),
            ready_at: Some(iso_ago(1)),
            estimate_asked_at: None,
            via: None,
        }
    }

    fn cycle(leads: Vec<Handover>, up: Option<f64>, down: Option<f64>) -> Cycle {
        Cycle {
            id: "SW-1".into(),
            account: "second".into(),
            mode: CycleMode::Auto,
            phase: CyclePhase::Ready,
            started_at: iso_ago(30),
            ask_at: Some(95.0),
            k: 1.0,
            use_at_request: up,
            use_at_ready: down,
            leads,
        }
    }

    #[test]
    fn a_finished_cycle_gives_its_cost() {
        let c = cycle(
            vec![
                handover(HandoverState::Ready, Some(1.0)),
                handover(HandoverState::Ready, Some(0.5)),
            ],
            Some(95.0),
            Some(96.8),
        );
        let cost = cost_of(&c).unwrap();
        assert!((cost.actual - 1.8).abs() < 1e-9);
        assert!((cost.estimated - 1.5).abs() < 1e-9);
    }

    #[test]
    fn a_cycle_with_a_cut_off_lead_or_missing_uses_teaches_nothing() {
        let ready = handover(HandoverState::Ready, Some(1.0));
        let cut = handover(HandoverState::CutOff, Some(1.0));
        assert!(cost_of(&cycle(vec![ready.clone(), cut], Some(95.0), Some(97.0))).is_none());
        assert!(cost_of(&cycle(vec![ready.clone()], None, Some(97.0))).is_none());
        assert!(
            cost_of(&cycle(
                vec![handover(HandoverState::Ready, None)],
                Some(95.0),
                Some(97.0)
            ))
            .is_none()
        );
        let mut open = cycle(vec![ready], Some(95.0), Some(97.0));
        open.phase = CyclePhase::Asked;
        assert!(cost_of(&open).is_none());
    }
}
