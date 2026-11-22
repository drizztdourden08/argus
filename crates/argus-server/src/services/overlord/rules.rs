//! The timed rules of the hosted Overlord (I-190), as one service: the routes of the owner's page
//! and the tools of the Overlord both call it, so a rule is checked, stored and delivered one way.
//! The pure model is `argus_overlord::rules`, the rows are `adapters::overlord::rules`, the timer
//! is `rules_tick.rs`. Every function blocks on the database; call it from a blocking thread.

use argus_core::clock::now_ms;
use argus_overlord::rules::{By, Rule, RuleInput, RulePatch, after_run, create, patch, prompt_text};
use argus_overlord::session::owner::deliver_plain;
use argus_store::{ActivityLevel, NewActivity};

use super::config;
use crate::Services;
use crate::adapters::overlord::rules::SqlRules;

const KIND: &str = "overlord";

/// Why a rule call did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuleFault {
    /// The input is out of range; the text says which field.
    Invalid(String),
    /// No rule has the id.
    Unknown,
    /// A run was asked for and no hosted child is up.
    NotRunning,
    /// The database failed.
    Store,
}

fn store(svc: &Services) -> Result<SqlRules, RuleFault> {
    SqlRules::open(svc.db.clone(), now_ms()).map_err(|_| RuleFault::Store)
}

fn found(rules: &SqlRules, id: &str) -> Result<Rule, RuleFault> {
    rules
        .get(id)
        .map_err(|_| RuleFault::Store)?
        .ok_or(RuleFault::Unknown)
}

/// Every rule, oldest first.
pub(crate) fn list(svc: &Services) -> Result<Vec<Rule>, RuleFault> {
    store(svc)?.list().map_err(|_| RuleFault::Store)
}

/// Adds a rule made by `by`, due one period from now.
pub(crate) fn add(svc: &Services, input: &RuleInput, by: By) -> Result<Rule, RuleFault> {
    let rules = store(svc)?;
    let id = rules.next_id().map_err(|_| RuleFault::Store)?;
    let rule = create(&id, input, by, now_ms()).map_err(RuleFault::Invalid)?;
    rules.save(&rule).map_err(|_| RuleFault::Store)?;
    Ok(rule)
}

/// Changes a rule.
pub(crate) fn update(svc: &Services, id: &str, change: &RulePatch) -> Result<Rule, RuleFault> {
    let rules = store(svc)?;
    let rule = patch(&found(&rules, id)?, change, now_ms()).map_err(RuleFault::Invalid)?;
    rules.save(&rule).map_err(|_| RuleFault::Store)?;
    Ok(rule)
}

/// Removes a rule.
pub(crate) fn remove(svc: &Services, id: &str) -> Result<(), RuleFault> {
    match store(svc)?.delete(id) {
        Ok(true) => Ok(()),
        Ok(false) => Err(RuleFault::Unknown),
        Err(_) => Err(RuleFault::Store),
    }
}

/// Gives the rule's text to the child, when the mode is live and a child is up.
pub(super) fn deliver(svc: &Services, rule: &Rule) -> bool {
    config::is_live(svc)
        && deliver_plain(svc.overlord.link().as_ref(), &prompt_text(rule)).is_ok()
}

/// Saves the rule after a run (or a skipped one) and writes its activity line.
pub(super) fn record(svc: &Services, rules: &SqlRules, rule: &Rule, ran: bool) {
    let saved = rules.save(&after_run(rule, now_ms()));
    let text = if ran {
        format!("overlord rule {} ran", rule.id)
    } else {
        format!("overlord rule {} skipped: not running", rule.id)
    };
    svc.activity
        .add(NewActivity::new(KIND, ActivityLevel::Info, text));
    if saved.is_err() {
        let id = serde_json::json!({ "id": rule.id });
        svc.log.warn("overlord rule not saved", id);
    }
}

/// Runs a rule now. Nothing is saved when no child is up.
pub(crate) fn run_now(svc: &Services, id: &str) -> Result<Rule, RuleFault> {
    let rules = store(svc)?;
    let rule = found(&rules, id)?;
    if !deliver(svc, &rule) {
        return Err(RuleFault::NotRunning);
    }
    record(svc, &rules, &rule, true);
    found(&rules, id)
}
