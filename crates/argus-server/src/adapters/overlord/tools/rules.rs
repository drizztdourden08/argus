//! The rule tools: the same service the owner's page calls, with its refusals as sentences.

use argus_overlord::rules::{By, RuleInput, RulePatch};

use crate::Services;
use crate::services::overlord::rules::{self, RuleFault};

fn sentence(fault: RuleFault) -> String {
    match fault {
        RuleFault::Invalid(text) => text,
        RuleFault::Unknown => "no such rule".to_owned(),
        RuleFault::NotRunning => "the hosted Overlord is not running".to_owned(),
        RuleFault::Store => "the rules could not be read".to_owned(),
    }
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| e.to_string())
}

/// `rules_list`: every rule as JSON.
pub(super) fn list(svc: &Services) -> Result<String, String> {
    json(&rules::list(svc).map_err(sentence)?)
}

/// `rule_add`: a rule made by the Overlord, answered as JSON.
pub(super) fn add(svc: &Services, input: &RuleInput) -> Result<String, String> {
    json(&rules::add(svc, input, By::Overlord).map_err(sentence)?)
}

/// `rule_update`: the changed rule as JSON.
pub(super) fn update(svc: &Services, id: &str, change: &RulePatch) -> Result<String, String> {
    json(&rules::update(svc, id, change).map_err(sentence)?)
}

/// `rule_remove`: a short confirmation.
pub(super) fn remove(svc: &Services, id: &str) -> Result<String, String> {
    rules::remove(svc, id).map_err(sentence)?;
    Ok(format!("removed rule {id}"))
}
