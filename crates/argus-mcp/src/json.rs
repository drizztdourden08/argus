//! Small helpers over `serde_json::Value` that follow how JavaScript reads the same documents:
//! a missing key is `undefined` and left out when written, `??` skips only null and undefined.

use argus_core::js;
use serde_json::{Map, Value};

/// The value under `key` unless it is missing or null (`doc.key ?? fallback`).
#[must_use]
pub fn nn<'a>(doc: &'a Value, key: &str) -> Option<&'a Value> {
    doc.get(key).filter(|v| !v.is_null())
}

/// The string under `key`.
#[must_use]
pub fn text<'a>(doc: &'a Value, key: &str) -> Option<&'a str> {
    doc.get(key).and_then(Value::as_str)
}

/// JavaScript truthiness of `doc.key`.
#[must_use]
pub fn truthy(doc: &Value, key: &str) -> bool {
    doc.get(key).is_some_and(js::truthy)
}

/// The array under `key`, empty when missing or not an array.
#[must_use]
pub fn items<'a>(doc: &'a Value, key: &str) -> &'a [Value] {
    doc.get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

/// Copy `keys` from `src` into `out`, skipping those `src` does not have (`undefined`).
pub fn pick(out: &mut Map<String, Value>, src: &Value, keys: &[&str]) {
    for key in keys {
        if let Some(v) = src.get(*key) {
            out.insert((*key).to_owned(), v.clone());
        }
    }
}

/// `a[key] === b[key2]` where both exist.
#[must_use]
pub fn same(a: &Value, key: &str, b: &Value, key2: &str) -> bool {
    matches!((a.get(key), b.get(key2)), (Some(x), Some(y)) if x == y)
}

/// `String(doc.key)` as JavaScript prints it, `undefined` for a missing key.
#[must_use]
pub fn display(doc: &Value, key: &str) -> String {
    doc.get(key)
        .map_or_else(|| "undefined".to_owned(), js::string)
}
