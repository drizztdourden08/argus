//! One raw event line made safe to keep.

use argus_core::redact::Redactor;
use serde_json::{Value, json};

fn scrub(redactor: &Redactor, value: &mut Value) {
    match value {
        Value::String(s) => *s = redactor.redact(s),
        Value::Array(items) => items.iter_mut().for_each(|v| scrub(redactor, v)),
        Value::Object(map) => {
            for (k, mut v) in std::mem::take(map) {
                scrub(redactor, &mut v);
                map.insert(redactor.redact(&k), v);
            }
        }
        _ => {}
    }
}

/// The event as one compact JSON line with every string scrubbed, so the result is still JSON.
/// A line that is not JSON is kept as `{"type":"raw","text":...}`.
#[must_use]
pub fn redacted_line(redactor: &Redactor, raw: &str) -> String {
    match serde_json::from_str::<Value>(raw) {
        Ok(mut value) => {
            scrub(redactor, &mut value);
            value.to_string()
        }
        Err(_) => json!({ "type": "raw", "text": redactor.redact(raw) }).to_string(),
    }
}
