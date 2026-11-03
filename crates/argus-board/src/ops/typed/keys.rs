//! The fields of an item that only the typed ops write.

use serde_json::{Map, Value};

const KEYS: [&str; 7] = [
    "type",
    "parent",
    "assignee",
    "stage",
    "fields",
    "blockedBy",
    "images",
];

/// Whether `key` is a typed field: one of the names, or `fields.<id>`.
pub(crate) fn is_key(key: &str) -> bool {
    KEYS.contains(&key) || key.starts_with("fields.")
}

/// Whether an edit names any typed field.
pub(crate) fn named_in(set: &Map<String, Value>) -> bool {
    set.keys().any(|k| is_key(k))
}

/// Whether a create gives any typed field a value.
pub(crate) fn valued_in(set: &Map<String, Value>) -> bool {
    set.iter().any(|(k, v)| is_key(k) && !v.is_null())
}
