//! The key order of a JavaScript object: keys that are array indexes (`"0"`, `"7"`, never `"07"`)
//! come first in ascending order, every other key after them in the order it was added. Node
//! writes a map of locks or log subjects that way, so a new key is put where Node would keep it.

use serde_json::{Map, Value};

fn index(key: &str) -> Option<u32> {
    let canonical = key == "0" || (!key.starts_with('0') && !key.is_empty());
    if !canonical || !key.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    key.parse().ok().filter(|n| *n != u32::MAX)
}

/// The keys in JavaScript order.
#[must_use]
pub fn js_keys(keys: Vec<String>) -> Vec<String> {
    let (mut numbered, others): (Vec<String>, Vec<String>) =
        keys.into_iter().partition(|k| index(k).is_some());
    numbered.sort_by_key(|k| index(k));
    numbered.extend(others);
    numbered
}

/// Puts the keys of a map in JavaScript order.
pub fn js_order(map: &mut Map<String, Value>) {
    let order = js_keys(map.keys().cloned().collect());
    let mut old = std::mem::take(map);
    for key in order {
        if let Some(value) = old.shift_remove(&key) {
            map.insert(key, value);
        }
    }
}
