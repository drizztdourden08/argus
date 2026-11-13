//! The id map a rewrite applies: key to id going forward, id to key going back.

use std::collections::BTreeMap;

use crate::journal::Entry;
use crate::plan::Plan;

/// `from` name to `to` name.
pub type IdMap = BTreeMap<String, String>;

/// Old key to new id, from the plan.
pub fn forward(plan: &Plan) -> IdMap {
    plan.boards
        .iter()
        .map(|(key, p)| (key.clone(), p.id.clone()))
        .collect()
}

/// New id to old key, from the journal.
pub fn reverse(entries: &[Entry]) -> IdMap {
    entries
        .iter()
        .map(|e| (e.id.clone(), e.key.clone()))
        .collect()
}

/// The `(key, id)` pairs of the plan, for the locks.
pub fn pairs(plan: &Plan) -> Vec<(String, String)> {
    plan.boards
        .iter()
        .map(|(key, p)| (key.clone(), p.id.clone()))
        .collect()
}
