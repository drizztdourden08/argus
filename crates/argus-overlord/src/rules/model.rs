//! The shape of a rule and of the input that makes or changes one.

use serde::{Deserialize, Serialize};

/// Who made a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum By {
    /// The owner, from the Rules tab or the seed.
    Owner,
    /// The hosted Overlord, through its tools.
    Overlord,
}

/// A timed prompt. Times are ISO strings, as everywhere in this crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    /// `OR-n`.
    pub id: String,
    pub title: String,
    pub prompt: String,
    /// Minutes between two runs.
    pub every_min: u32,
    pub enabled: bool,
    pub created_by: By,
    pub created_at: String,
    pub updated_at: String,
    pub last_run_at: Option<String>,
    pub next_run_at: String,
}

/// What a new rule needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleInput {
    pub title: String,
    pub prompt: String,
    pub every_min: i64,
    pub enabled: bool,
}

/// The fields a change may carry; a missing one stays as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RulePatch {
    pub title: Option<String>,
    pub prompt: Option<String>,
    pub every_min: Option<i64>,
    pub enabled: Option<bool>,
}
