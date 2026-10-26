//! A process rule: which processes the monitor lists and what is acceptable for each.

use crate::num::round_to_i64;
use serde::{Serialize, Serializer};

/// The regexes a process must all match for a rule to take it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Match {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl Match {
    /// A rule with an empty match takes every process.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.cmd.is_none() && self.path.is_none()
    }
}

#[allow(clippy::ref_option)]
fn serialize_number<S: Serializer>(value: &Option<f64>, out: S) -> Result<S::Ok, S::Error> {
    match value {
        Some(n) if n.fract() == 0.0 && n.abs() < 1e15 => out.serialize_i64(round_to_i64(*n)),
        Some(n) => out.serialize_f64(*n),
        None => out.serialize_none(),
    }
}

/// One rule of the Overlord's process rules file.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Rule {
    pub name: String,
    #[serde(rename = "match")]
    pub matcher: Match,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore: Option<Vec<String>>,
    #[serde(
        rename = "minMemMB",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_number"
    )]
    pub min_mem_mb: Option<f64>,
    #[serde(
        rename = "maxMemMB",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_number"
    )]
    pub max_mem_mb: Option<f64>,
    #[serde(
        rename = "maxGroupMemMB",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_number"
    )]
    pub max_group_mem_mb: Option<f64>,
    #[serde(
        rename = "maxAgeMin",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_number"
    )]
    pub max_age_min: Option<f64>,
    #[serde(
        rename = "idleMin",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_number"
    )]
    pub idle_min: Option<f64>,
    #[serde(rename = "onlyFlagged", skip_serializing_if = "Option::is_none")]
    pub only_flagged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orphan: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hung: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub killable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Rule {
    /// Whether the rule lists only the processes that carry a flag.
    #[must_use]
    pub fn lists_only_flagged(&self) -> bool {
        self.only_flagged == Some(true)
    }

    /// Whether the kill buttons and `coord proc kill` may end the rule's processes.
    #[must_use]
    pub fn is_killable(&self) -> bool {
        self.killable == Some(true)
    }

    /// Whether the rule asks for the orphan flag.
    #[must_use]
    pub fn flags_orphans(&self) -> bool {
        self.orphan == Some(true)
    }

    /// Whether the rule asks for the hung flag.
    #[must_use]
    pub fn flags_hung(&self) -> bool {
        self.hung == Some(true)
    }
}
