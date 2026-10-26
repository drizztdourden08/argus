//! Reading board docs the way the JavaScript did: a missing field is `undefined` (left out of the
//! JSON), `?? null` and `?? ''` have a helper each, and archived or deleted docs are skipped.

use argus_board::Board;
use argus_board::json::{display, doc_mut, section, truthy};
use argus_core::clock::parse_ms;
use serde_json::{Map, Value};

use crate::cli::exit::CoordError;

/// The docs of a collection, in order.
#[must_use]
pub fn all<'a>(board: &'a Board, coll: &str) -> Vec<&'a Value> {
    section(board, coll).values().collect()
}

/// The docs that are neither archived nor in the trash: what the summaries and reports list.
#[must_use]
pub fn visible<'a>(board: &'a Board, coll: &str) -> Vec<&'a Value> {
    all(board, coll)
        .into_iter()
        .filter(|d| !truthy(d, "archivedAt") && !truthy(d, "deletedAt"))
        .collect()
}

/// A field that exists (`undefined` is `None`).
#[must_use]
pub fn field(doc: &Value, key: &str) -> Option<Value> {
    doc.get(key).cloned()
}

/// `value ?? null`.
#[must_use]
pub fn or_null(value: Option<Value>) -> Value {
    value.filter(|v| !v.is_null()).unwrap_or(Value::Null)
}

/// A JSON object from fields that may be `undefined`: those are left out, as `JSON.stringify`
/// leaves them.
#[must_use]
pub fn object(pairs: Vec<(&str, Option<Value>)>) -> Value {
    let map: Map<String, Value> = pairs
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| (k.to_owned(), v)))
        .collect();
    Value::Object(map)
}

/// `String(value ?? '')`: a table cell.
#[must_use]
pub fn cell(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(v) => display(v),
    }
}

/// The id of a doc as text.
#[must_use]
pub fn id_of(doc: &Value) -> String {
    display(&doc["id"])
}

/// A thread of the board.
///
/// # Errors
/// Refused as `no thread "<name>".` when the board has no such thread.
pub fn named_thread<'a>(board: &'a Board, name: &str) -> Result<&'a Value, CoordError> {
    board["threads"]
        .get(name)
        .filter(|t| !t.is_null())
        .ok_or_else(|| CoordError::refuse(format!("no thread \"{name}\".")))
}

/// A thread of the board, to change.
///
/// # Errors
/// Refused as `no thread "<id>".` when the board has no such thread.
pub fn thread_mut<'a>(board: &'a mut Board, id: &str) -> Result<&'a mut Value, CoordError> {
    doc_mut(board, "threads", id).ok_or_else(|| CoordError::refuse(format!("no thread \"{id}\".")))
}

/// Whether a character is white space to JavaScript (`\s`, `trim`).
#[must_use]
pub fn is_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// The words of a text, split on runs of white space as `split(/\s+/).filter(Boolean)` does.
pub fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(is_space).filter(|w| !w.is_empty())
}

/// `iso ? Date.now() - Date.parse(iso) : Infinity`, in milliseconds. A time that does not parse
/// is `NaN`, which no comparison is true for.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn age_ms(iso: &Value, now_ms: i64) -> f64 {
    match iso.as_str().filter(|s| !s.is_empty()) {
        None if !argus_core::js::truthy(iso) => f64::INFINITY,
        None => f64::NAN,
        Some(text) => parse_ms(text).map_or(f64::NAN, |t| (now_ms - t) as f64),
    }
}

/// A JavaScript number as a template literal prints it.
#[must_use]
pub fn js_number(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_owned()
    } else if n.is_infinite() {
        if n > 0.0 { "Infinity" } else { "-Infinity" }.to_owned()
    } else {
        display(&argus_core::js::number(n))
    }
}

/// The items of a JSON array.
#[must_use]
pub fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}
