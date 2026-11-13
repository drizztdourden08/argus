//! Small helpers to walk a JSON value.

use serde_json::Value;

use crate::map::IdMap;

/// The values of an object or the elements of an array.
pub fn children_mut(v: &mut Value) -> Vec<&mut Value> {
    match v {
        Value::Object(m) => m.values_mut().collect(),
        Value::Array(a) => a.iter_mut().collect(),
        _ => Vec::new(),
    }
}

/// Replaces a string that is a key of `map` by its value.
pub fn swap_str(v: &mut Value, map: &IdMap) {
    if let Some(to) = v.as_str().and_then(|s| map.get(s)) {
        *v = Value::String(to.clone());
    }
}

/// `swap_str` on one field of an object.
pub fn swap_field(v: &mut Value, field: &str, map: &IdMap) {
    if let Some(inner) = v.get_mut(field) {
        swap_str(inner, map);
    }
}

/// Renames the keys of an object that are keys of `map`, keeping the order.
pub fn rekey(v: &mut Value, map: &IdMap) {
    if let Value::Object(m) = v {
        let old = std::mem::take(m);
        for (k, val) in old {
            let name = map.get(&k).cloned().unwrap_or(k);
            m.insert(name, val);
        }
    }
}

/// `rekey` on one field of an object, then the inner `key` field of every entry.
pub fn rekey_field(v: &mut Value, field: &str, map: &IdMap, inner_key: bool) {
    let Some(table) = v.get_mut(field) else {
        return;
    };
    rekey(table, map);
    if inner_key {
        for entry in children_mut(table) {
            swap_field(entry, "key", map);
        }
    }
}
