//! The shape of the Kanban config.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// The rule kinds a config may name. The first four are checked when an item is written, the
/// rest by the sweep.
pub const RULE_KINDS: [&str; 8] = [
    "require-parent",
    "require-assignee",
    "limit",
    "require-field",
    "dwell",
    "assignee-gone",
    "over-limit",
    "unassigned",
];

/// A board's Kanban config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct KanbanConfig {
    /// The columns, left to right.
    pub columns: Vec<Column>,
    /// The rules the columns name.
    #[serde(default)]
    pub rules: Vec<Rule>,
}

/// One column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Column {
    /// The id, unique in the config.
    pub id: String,
    /// The title shown.
    pub name: String,
    /// A Tessera accent name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    /// Which items sit in it.
    #[serde(rename = "match")]
    pub matches: Match,
    /// How many items it may hold.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<Limit>,
    /// The ids of the rules that apply to it.
    #[serde(default)]
    pub rules: Vec<String>,
}

/// What puts an item in a column: its lane, and its stage when stages are named.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Match {
    /// The lanes (`status`) it takes.
    pub lanes: Vec<String>,
    /// The stages it narrows to; empty means every stage of the lanes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<String>,
}

/// A column limit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Limit {
    /// The most items the column holds.
    pub max: i64,
    /// What passing it does: `warn`, `refuse` or `alert`.
    pub mode: String,
}

/// One rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Rule {
    /// The id, unique in the config.
    pub id: String,
    /// One of [`RULE_KINDS`].
    pub kind: String,
    /// What the kind needs.
    #[serde(default = "no_params")]
    pub params: Value,
    /// For a write-time kind: `warn` or `refuse`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// For a sweep kind: who hears of a break, and how often.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alert: Option<Alert>,
}

/// Who is told when a sweep rule is broken.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Alert {
    /// `item-thread`, `assignee`, `lead`, `owner` or a thread id.
    pub to: String,
    /// Whether it wakes the thread, or only lands in its inbox.
    pub wake: bool,
    /// The wait before the same break is told again.
    pub repeat_ms: i64,
}

fn no_params() -> Value {
    json!({})
}
