//! The runs of the in-memory backend: a fixed set of run rows and one log every run shares.

use serde_json::{Value, json};

use crate::backend::{LogQuery, RunFilter};
use crate::json::text;

/// Runs of `key` (or of every project) that pass the filter, in row order.
pub fn filter_runs(rows: &[Value], key: Option<&str>, filter: &RunFilter) -> Vec<Value> {
    rows.iter()
        .filter(|r| key.is_none_or(|k| text(r, "scope") == Some(k)))
        .filter(|r| {
            filter
                .status
                .as_deref()
                .is_none_or(|s| text(r, "status") == Some(s))
        })
        .filter(|r| {
            filter
                .name
                .as_deref()
                .is_none_or(|n| text(r, "name") == Some(n))
        })
        .cloned()
        .collect()
}

/// `{lines, next, dropped}` of the lines after `query.after`.
pub fn log_chunk(lines: &[Value], query: &LogQuery) -> Value {
    let after = query.after.unwrap_or(0);
    let next = lines
        .iter()
        .filter_map(|l| l["seq"].as_u64())
        .max()
        .unwrap_or(after);
    let kept: Vec<&Value> = lines
        .iter()
        .filter(|l| l["seq"].as_u64().unwrap_or(0) > after)
        .collect();
    json!({ "lines": kept, "next": next, "dropped": 0 })
}
