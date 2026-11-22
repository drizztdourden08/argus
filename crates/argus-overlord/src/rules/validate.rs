//! The checks of a rule's fields, and the rule a valid input or patch makes.

use argus_core::clock::iso_from_ms;

use super::schedule::next_run;
use super::{
    By, EVERY_MAX, EVERY_MIN, PROMPT_MAX, Rule, RuleInput, RulePatch, TITLE_MAX, TITLE_MIN,
};

fn title(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    let count = trimmed.chars().count();
    if (TITLE_MIN..=TITLE_MAX).contains(&count) {
        return Ok(trimmed.to_owned());
    }
    Err(format!(
        "the title must be {TITLE_MIN} to {TITLE_MAX} characters"
    ))
}

fn prompt(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    let count = trimmed.chars().count();
    if (1..=PROMPT_MAX).contains(&count) {
        return Ok(trimmed.to_owned());
    }
    Err(format!("the prompt must be 1 to {PROMPT_MAX} characters"))
}

fn every(value: i64) -> Result<u32, String> {
    u32::try_from(value)
        .ok()
        .filter(|minutes| (EVERY_MIN..=EVERY_MAX).contains(minutes))
        .ok_or_else(|| format!("everyMin must be {EVERY_MIN} to {EVERY_MAX} minutes"))
}

/// A new rule `id`, due one period after `now_ms`.
///
/// # Errors
/// A plain sentence naming the field that is out of range.
pub fn create(id: &str, input: &RuleInput, by: By, now_ms: i64) -> Result<Rule, String> {
    let every_min = every(input.every_min)?;
    let now = iso_from_ms(now_ms);
    Ok(Rule {
        id: id.to_owned(),
        title: title(&input.title)?,
        prompt: prompt(&input.prompt)?,
        every_min,
        enabled: input.enabled,
        created_by: by,
        created_at: now.clone(),
        updated_at: now,
        last_run_at: None,
        next_run_at: next_run(every_min, now_ms),
    })
}

/// The rule with the patch applied. A new period, or a rule switched back on, is due one period
/// after `now_ms`.
///
/// # Errors
/// A plain sentence naming the field that is out of range.
pub fn patch(rule: &Rule, change: &RulePatch, now_ms: i64) -> Result<Rule, String> {
    let mut next = rule.clone();
    if let Some(value) = &change.title {
        next.title = title(value)?;
    }
    if let Some(value) = &change.prompt {
        next.prompt = prompt(value)?;
    }
    if let Some(value) = change.every_min {
        next.every_min = every(value)?;
    }
    if let Some(value) = change.enabled {
        next.enabled = value;
    }
    let moved = next.every_min != rule.every_min || (next.enabled && !rule.enabled);
    if moved {
        next.next_run_at = next_run(next.every_min, now_ms);
    }
    next.updated_at = iso_from_ms(now_ms);
    Ok(next)
}
