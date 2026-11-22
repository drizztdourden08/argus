//! When a rule is due, what a run changes, and the text the child gets.

use argus_core::clock::{iso_from_ms, parse_ms};

use super::Rule;

const MINUTE_MS: i64 = 60_000;

/// The ISO time one period after `now_ms`.
pub(super) fn next_run(every_min: u32, now_ms: i64) -> String {
    iso_from_ms(now_ms.saturating_add(i64::from(every_min) * MINUTE_MS))
}

/// The ids of the enabled rules whose time has come, in the order given.
#[must_use]
pub fn due(rules: &[Rule], now_ms: i64) -> Vec<String> {
    rules
        .iter()
        .filter(|rule| rule.enabled)
        .filter(|rule| parse_ms(&rule.next_run_at).is_some_and(|at| at <= now_ms))
        .map(|rule| rule.id.clone())
        .collect()
}

/// The rule after a run or a skipped run: its last run is now and its next one a period later.
#[must_use]
pub fn after_run(rule: &Rule, now_ms: i64) -> Rule {
    Rule {
        last_run_at: Some(iso_from_ms(now_ms)),
        next_run_at: next_run(rule.every_min, now_ms),
        ..rule.clone()
    }
}

/// The soonest next run among the enabled rules.
#[must_use]
pub fn next_due_at(rules: &[Rule]) -> Option<String> {
    rules
        .iter()
        .filter(|rule| rule.enabled)
        .filter_map(|rule| parse_ms(&rule.next_run_at).map(|at| (at, &rule.next_run_at)))
        .min_by_key(|(at, _)| *at)
        .map(|(_, text)| text.clone())
}

/// The message for the child: `Rule OR-n "<title>" (every N min): <prompt>`.
#[must_use]
pub fn prompt_text(rule: &Rule) -> String {
    format!(
        "Rule {} \"{}\" (every {} min): {}",
        rule.id, rule.title, rule.every_min, rule.prompt
    )
}
