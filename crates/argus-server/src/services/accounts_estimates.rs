//! The handover estimates of the accounts service: what the rules read after each poll
//! ([`estimate_state`]) and the calibration factor learned from a finished cycle
//! ([`learn_from_cycle`]). The arithmetic is `argus_accounts::handover::estimator`; this is the
//! reading of the boards, the usage and the config around it.

use argus_accounts::handover::estimator::{
    LeadEstimate, LeadInput, ask_at, calibrate, cost_of, count_lead, leads_to_ask, near_switch,
    need,
};
use argus_accounts::handover::{Cycle, CyclePhase, Lead};
use argus_accounts::{PollResult, binding_limit};
use argus_core::clock::now_ms;
use serde::Serialize;
use serde_json::json;

use super::AccountsService;
use super::accounts::save;
use crate::Services;

/// A lead, named by its board: every board has a `lead` thread.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LeadRef {
    /// The board key.
    pub board: String,
    /// The lead's thread.
    pub thread: String,
}

/// What one lead is counted at.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LeadState {
    /// The board key.
    pub board: String,
    /// The lead's thread.
    pub thread: String,
    /// Points of the limit its handover costs: its estimate when fresh, Argus's prior otherwise.
    pub estimate: f64,
    /// Minutes until its handover file is written.
    pub minutes: f64,
    /// Whether the lead's own fresh estimate was used.
    pub fresh: bool,
}

/// The ask point of an account and the leads to ask for an estimate.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EstimateState {
    /// The use of the binding limit at which the leads are asked for their handover.
    pub ask_at: f64,
    /// The points the handovers are expected to use, `k` and the burn included.
    pub need: f64,
    /// The calibration factor in use.
    pub k: f64,
    /// The leads counted: those of the open cycle, or the leads working now.
    pub leads: Vec<LeadState>,
    /// The leads to send a handover-estimate wake now: not before the account is near its
    /// switch point, never more often than every 20 minutes per lead.
    pub should_ask_estimates: Vec<LeadRef>,
    /// Leads working now that the open cycle does not have yet; the rules add them with
    /// `Handovers::add_lead`.
    pub new_leads: Vec<LeadRef>,
}

fn lead_ref(board: &str, thread: &str) -> LeadRef {
    LeadRef {
        board: board.to_owned(),
        thread: thread.to_owned(),
    }
}

fn state_of(l: LeadEstimate) -> LeadState {
    LeadState {
        board: l.board,
        thread: l.thread,
        estimate: l.percent,
        minutes: l.minutes,
        fresh: l.fresh,
    }
}

fn working_input(accounts: &AccountsService, lead: &Lead) -> LeadInput {
    let loads = accounts
        .handovers()
        .map(|h| h.desk().agent_loads(&lead.board, &lead.thread))
        .unwrap_or_default();
    LeadInput {
        board: lead.board.clone(),
        thread: lead.thread.clone(),
        agents: loads,
        estimate: None,
        asked_at_ms: None,
    }
}

/// The inputs of the estimator: the pending handovers of the open cycle of `account`, else the
/// leads working now. The second value lists the working leads the open cycle lacks.
fn inputs(
    accounts: &AccountsService,
    cycle: Option<&Cycle>,
    working: &[Lead],
) -> (Vec<LeadInput>, Vec<LeadRef>) {
    let Some(cycle) = cycle else {
        return (
            working.iter().map(|l| working_input(accounts, l)).collect(),
            Vec::new(),
        );
    };
    let desk = accounts.handovers();
    let listed = cycle
        .leads
        .iter()
        .filter(|h| h.state.is_pending())
        .map(|h| {
            let loads = desk
                .as_ref()
                .map(|d| d.desk().agent_loads(&h.board, &h.thread))
                .unwrap_or_default();
            LeadInput::of_handover(h, loads)
        })
        .collect();
    let new = working
        .iter()
        .filter(|l| cycle.handover_of(&l.board, &l.thread).is_none())
        .map(|l| lead_ref(&l.board, &l.thread))
        .collect();
    (listed, new)
}

fn reading_of(accounts: &AccountsService, key: &str) -> Option<(f64, Option<f64>)> {
    let latest = accounts.latest_usage();
    let polled: &PollResult = latest.get(key)?;
    let use_now = binding_limit(&polled.usage)?.percent;
    Some((use_now, polled.burn_per_min))
}

/// The ask point and the estimate plan of an account, to call after each poll. `None` when the
/// service has no handover desk yet or the account is unknown.
#[must_use]
pub fn estimate_state(accounts: &AccountsService, account_key: &str) -> Option<EstimateState> {
    let desk = accounts.handovers()?;
    let account = accounts
        .accounts()
        .into_iter()
        .find(|a| a.key == account_key)?;
    let config = accounts.config();
    let defaults = &config.estimate_defaults;
    let cycle = desk
        .desk()
        .cycle()
        .filter(|c| c.account == account_key && c.phase.is_open());
    let working = desk.desk().working_leads();
    let (inputs, new_leads) = inputs(accounts, cycle.as_ref(), &working);
    let now = now_ms();
    let counted: Vec<LeadEstimate> = inputs
        .iter()
        .map(|l| count_lead(l, defaults, now))
        .collect();
    let reading = reading_of(accounts, account_key);
    let burn = reading.and_then(|(_, burn)| burn);
    let need = need(&counted, config.k, burn, defaults.safety);
    let switch_at = account.rules.switch_at;
    let near = reading.is_some_and(|(use_now, burn)| near_switch(use_now, switch_at, burn));
    let may_ask = cycle
        .as_ref()
        .is_none_or(|c| c.phase == CyclePhase::Estimating);
    let should_ask_estimates = leads_to_ask(&inputs, near && may_ask, now)
        .into_iter()
        .map(|l| lead_ref(&l.board, &l.thread))
        .collect();
    Some(EstimateState {
        ask_at: ask_at(switch_at, need),
        need,
        k: config.k,
        leads: counted.into_iter().map(state_of).collect(),
        should_ask_estimates,
        new_leads,
    })
}

/// Learns from a finished cycle: moves `k` toward what the cycle cost, keeps it in the settings
/// and saves them. Returns the new `k`, or `None` when the cycle cannot teach (a lead was cut
/// off, a use was not read, nothing was estimated).
pub fn learn_from_cycle(svc: &Services, accounts: &AccountsService, cycle: &Cycle) -> Option<f64> {
    let cost = cost_of(cycle)?;
    let mut config = accounts.config();
    config.k = calibrate(config.k, cost);
    accounts.set_config(config.clone());
    if !svc.shadow {
        if let Err(e) = save(svc, &config) {
            svc.log.error(
                "accounts calibration not saved",
                json!({ "error": e.to_string() }),
            );
        }
    }
    Some(config.k)
}
