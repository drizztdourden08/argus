//! What a prune would delete: the rule of the contract.

use crate::entry::Entry;
use crate::places::Usage;
use crate::tmp::Tmp;

/// An entry may go when no place uses its name at its exact version and the grace period, counted from its last use,
/// is over.
pub fn is_prunable(entry: &Entry, usage: &Usage, now_ms: i64, grace_ms: i64) -> bool {
    usage.users_of(&entry.name, &entry.version).is_empty()
        && now_ms - entry.last_used_ms >= grace_ms
}

/// The entries and the left-over temp folders a prune deletes.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Plan {
    pub entries: Vec<Entry>,
    pub tmp: Vec<Tmp>,
}

pub fn plan(entries: &[Entry], usage: &Usage, tmp: Vec<Tmp>, now_ms: i64, grace_ms: i64) -> Plan {
    Plan {
        entries: entries
            .iter()
            .filter(|e| is_prunable(e, usage, now_ms, grace_ms))
            .cloned()
            .collect(),
        tmp,
    }
}
