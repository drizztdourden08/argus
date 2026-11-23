//! Checking a config before it is stored. Every refusal carries a stable key, the first word of
//! its message.

use std::collections::HashSet;
use std::fmt;

use serde_json::Value;

use super::rule_check::check_rules;
use super::types::{Column, KanbanConfig, Limit};
use crate::registry::registry;

const LIMIT_MODES: [&str; 3] = ["warn", "refuse", "alert"];

/// A config that cannot be stored: a stable key and what is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// `bad-shape`, `no-columns`, `bad-column`, `duplicate-column`, `unknown-lane`,
    /// `unknown-stage`, `bad-limit`, `unknown-rule`, `duplicate-rule`, `unknown-rule-kind`,
    /// `bad-rule-mode`, `bad-params` or `bad-alert`.
    pub key: &'static str,
    /// What is wrong, in a sentence.
    pub detail: String,
}

impl ConfigError {
    pub(super) fn new(key: &'static str, detail: impl Into<String>) -> Self {
        ConfigError {
            key,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.key, self.detail)
    }
}

/// Read a config from JSON and check it.
///
/// # Errors
/// The first thing wrong, with its key.
pub fn parse(value: &Value) -> Result<KanbanConfig, ConfigError> {
    let config: KanbanConfig = serde_json::from_value(value.clone())
        .map_err(|e| ConfigError::new("bad-shape", e.to_string()))?;
    check(&config)?;
    Ok(config)
}

fn check(config: &KanbanConfig) -> Result<(), ConfigError> {
    if config.columns.is_empty() {
        return Err(ConfigError::new("no-columns", "a config needs a column."));
    }
    check_rules(config)?;
    let rule_ids: HashSet<&str> = config.rules.iter().map(|r| r.id.as_str()).collect();
    let mut seen = HashSet::new();
    for column in &config.columns {
        if !seen.insert(column.id.as_str()) {
            return Err(ConfigError::new(
                "duplicate-column",
                format!("column id \"{}\" is used twice.", column.id),
            ));
        }
        check_column(column, &rule_ids)?;
    }
    Ok(())
}

fn check_column(column: &Column, rule_ids: &HashSet<&str>) -> Result<(), ConfigError> {
    if column.id.trim().is_empty() || column.name.trim().is_empty() {
        return Err(ConfigError::new(
            "bad-column",
            "a column needs an id and a name.",
        ));
    }
    check_match(column)?;
    if let Some(limit) = &column.limit {
        check_limit(&column.id, limit)?;
    }
    match column
        .rules
        .iter()
        .find(|id| !rule_ids.contains(id.as_str()))
    {
        Some(id) => Err(ConfigError::new(
            "unknown-rule",
            format!(
                "column \"{}\" names rule \"{id}\", which the config does not define.",
                column.id
            ),
        )),
        None => Ok(()),
    }
}

fn check_match(column: &Column) -> Result<(), ConfigError> {
    let reg = registry();
    let lanes = &column.matches.lanes;
    if lanes.is_empty() {
        return Err(ConfigError::new(
            "unknown-lane",
            format!("column \"{}\" takes no lane.", column.id),
        ));
    }
    if let Some(lane) = lanes.iter().find(|l| !reg.lanes.contains(l)) {
        return Err(ConfigError::new(
            "unknown-lane",
            format!(
                "column \"{}\" names lane \"{lane}\"; the lanes are {}.",
                column.id,
                reg.lanes.join(", ")
            ),
        ));
    }
    let unknown = |s: &&String| !reg.types.iter().any(|t| t.stage(s).is_some());
    match column.matches.stages.iter().find(unknown) {
        Some(stage) => Err(ConfigError::new(
            "unknown-stage",
            format!(
                "column \"{}\" names stage \"{stage}\", which no item type has.",
                column.id
            ),
        )),
        None => Ok(()),
    }
}

fn check_limit(column: &str, limit: &Limit) -> Result<(), ConfigError> {
    if limit.max < 1 {
        return Err(ConfigError::new(
            "bad-limit",
            format!("column \"{column}\" limit max must be 1 or more."),
        ));
    }
    if !LIMIT_MODES.contains(&limit.mode.as_str()) {
        return Err(ConfigError::new(
            "bad-limit",
            format!(
                "column \"{column}\" limit mode must be one of {}.",
                LIMIT_MODES.join(", ")
            ),
        ));
    }
    Ok(())
}
