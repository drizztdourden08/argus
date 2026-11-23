//! Checking the rules of a config: ids, kinds, params, modes and alerts.

use std::collections::HashSet;

use serde_json::Value;

use super::types::{Alert, KanbanConfig, RULE_KINDS, Rule};
use super::validation::ConfigError;

const WRITE_MODES: [&str; 2] = ["warn", "refuse"];

pub(super) fn check_rules(config: &KanbanConfig) -> Result<(), ConfigError> {
    let mut seen = HashSet::new();
    for rule in &config.rules {
        if !seen.insert(rule.id.as_str()) {
            return Err(ConfigError::new(
                "duplicate-rule",
                format!("rule id \"{}\" is used twice.", rule.id),
            ));
        }
        check_rule(rule)?;
    }
    Ok(())
}

fn check_rule(rule: &Rule) -> Result<(), ConfigError> {
    if !RULE_KINDS.contains(&rule.kind.as_str()) {
        return Err(ConfigError::new(
            "unknown-rule-kind",
            format!(
                "rule \"{}\" has kind \"{}\"; the kinds are {}.",
                rule.id,
                rule.kind,
                RULE_KINDS.join(", ")
            ),
        ));
    }
    let bad_mode = rule
        .mode
        .as_ref()
        .is_some_and(|m| !WRITE_MODES.contains(&m.as_str()));
    if bad_mode {
        return Err(ConfigError::new(
            "bad-rule-mode",
            format!("rule \"{}\" mode must be warn or refuse.", rule.id),
        ));
    }
    check_params(rule)?;
    rule.alert
        .as_ref()
        .map_or(Ok(()), |a| check_alert(&rule.id, a))
}

fn bad_params(rule: &Rule, what: &str) -> ConfigError {
    ConfigError::new(
        "bad-params",
        format!("rule \"{}\" ({}) {what}.", rule.id, rule.kind),
    )
}

fn positive(rule: &Rule, field: &str) -> Result<(), ConfigError> {
    match rule.params[field].as_i64() {
        Some(n) if n > 0 => Ok(()),
        _ => Err(bad_params(
            rule,
            &format!("needs params.{field} as a whole number above 0"),
        )),
    }
}

fn names(rule: &Rule, field: &str, needed: bool) -> Result<(), ConfigError> {
    let fits = match &rule.params[field] {
        Value::Null => !needed,
        Value::Array(list) => {
            (!needed || !list.is_empty())
                && list
                    .iter()
                    .all(|v| v.as_str().is_some_and(|s| !s.is_empty()))
        }
        _ => false,
    };
    if fits {
        Ok(())
    } else {
        Err(bad_params(
            rule,
            &format!("needs params.{field} as a list of names"),
        ))
    }
}

fn check_params(rule: &Rule) -> Result<(), ConfigError> {
    if !rule.params.is_object() {
        return Err(bad_params(rule, "needs params as an object"));
    }
    match rule.kind.as_str() {
        "dwell" | "unassigned" => positive(rule, "ms"),
        "require-parent" => names(rule, "types", true),
        "require-assignee" => names(rule, "kinds", false),
        "require-field" => match rule.params["field"].as_str() {
            Some(f) if !f.is_empty() => Ok(()),
            _ => Err(bad_params(rule, "needs params.field as a field name")),
        },
        _ => Ok(()),
    }
}

fn check_alert(rule: &str, alert: &Alert) -> Result<(), ConfigError> {
    if alert.to.trim().is_empty() {
        return Err(ConfigError::new(
            "bad-alert",
            format!("rule \"{rule}\" alert needs a recipient in to."),
        ));
    }
    if alert.repeat_ms < 1 {
        return Err(ConfigError::new(
            "bad-alert",
            format!("rule \"{rule}\" alert repeatMs must be above 0."),
        ));
    }
    Ok(())
}
