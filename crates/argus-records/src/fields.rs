//! Lenient reads of a JSON record: a field of the wrong type is missing, never an error, so one
//! odd field does not hide a whole record.

use serde_json::Value;

/// A text field, or the empty string.
pub(crate) fn text(v: &Value, key: &str) -> String {
    opt_text(v, key).unwrap_or_default()
}

/// A non-empty text field.
pub(crate) fn opt_text(v: &Value, key: &str) -> Option<String> {
    v[key].as_str().filter(|s| !s.is_empty()).map(str::to_owned)
}

/// A whole number field, or 0.
pub(crate) fn number(v: &Value, key: &str) -> u64 {
    v[key].as_u64().unwrap_or(0)
}

/// The items of an array field.
pub(crate) fn list<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map_or(&[], Vec::as_slice)
}
