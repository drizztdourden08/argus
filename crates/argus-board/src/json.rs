//! Helpers over `serde_json::Value` that read a board the way the JavaScript did: a missing
//! field is `null`, a missing collection is empty, and a number or text prints like a template
//! literal.

use std::sync::LazyLock;

use argus_core::js;
use serde_json::{Map, Value};

static EMPTY: LazyLock<Map<String, Value>> = LazyLock::new(Map::new);

/// The object a value holds, or an empty one.
#[must_use]
pub fn obj(v: &Value) -> &Map<String, Value> {
    v.as_object().unwrap_or(&EMPTY)
}

/// The value as an object, made one first when it is not.
pub fn obj_mut(v: &mut Value) -> &mut Map<String, Value> {
    match v {
        Value::Object(m) => m,
        other => {
            *other = Value::Object(Map::new());
            obj_mut(other)
        }
    }
}

/// A collection of a board (`threads`, `items`, ...), empty when missing.
#[must_use]
pub fn section<'a>(board: &'a Value, name: &str) -> &'a Map<String, Value> {
    obj(&board[name])
}

/// The slot of a key, created as `null` when missing.
pub fn slot<'a>(v: &'a mut Value, key: &str) -> &'a mut Value {
    obj_mut(v).entry(key.to_owned()).or_insert(Value::Null)
}

/// The object under a key, made one first when it is missing or not an object (`x[k] ??= {}`).
pub fn map_mut<'a>(v: &'a mut Value, key: &str) -> &'a mut Map<String, Value> {
    let s = slot(v, key);
    if !s.is_object() {
        *s = Value::Object(Map::new());
    }
    obj_mut(s)
}

/// The array under a key, made one first when it is missing or not an array.
pub fn array_mut<'a>(v: &'a mut Value, key: &str) -> &'a mut Vec<Value> {
    array_in(slot(v, key))
}

fn array_in(v: &mut Value) -> &mut Vec<Value> {
    match v {
        Value::Array(a) => a,
        other => {
            *other = Value::Array(Vec::new());
            array_in(other)
        }
    }
}

/// A doc of a collection.
#[must_use]
pub fn doc<'a>(board: &'a Value, coll: &str, id: &str) -> Option<&'a Value> {
    section(board, coll).get(id)
}

/// A doc of a collection, to change.
pub fn doc_mut<'a>(board: &'a mut Value, coll: &str, id: &str) -> Option<&'a mut Value> {
    board.get_mut(coll)?.get_mut(id)
}

/// The ids of a collection, in order.
#[must_use]
pub fn ids(board: &Value, coll: &str) -> Vec<String> {
    section(board, coll).keys().cloned().collect()
}

/// Set a field (insert keeps the position of an existing key).
pub fn set(v: &mut Value, key: &str, val: impl Into<Value>) {
    obj_mut(v).insert(key.to_owned(), val.into());
}

/// Delete a field, keeping the order of the others.
pub fn unset(v: &mut Value, key: &str) {
    if let Some(m) = v.as_object_mut() {
        m.shift_remove(key);
    }
}

/// `v[key] ??= val`.
pub fn default_field(v: &mut Value, key: &str, val: impl Into<Value>) {
    let s = slot(v, key);
    if s.is_null() {
        *s = val.into();
    }
}

/// A text field.
#[must_use]
pub fn text<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v[key].as_str()
}

/// A field that is neither null nor missing, as text (`String(v[key])`).
#[must_use]
pub fn nullish(v: &Value, key: &str) -> Option<String> {
    match &v[key] {
        Value::Null => None,
        other => Some(display(other)),
    }
}

/// JavaScript truthiness of a field.
#[must_use]
pub fn truthy(v: &Value, key: &str) -> bool {
    js::truthy(&v[key])
}

/// A value as a template literal prints it.
#[must_use]
pub fn display(v: &Value) -> String {
    match v {
        Value::Number(n) => n
            .as_f64()
            .map_or_else(|| n.to_string(), |f| js::number(f).to_string()),
        other => js::string(other),
    }
}

/// A possibly missing value as a template literal prints it (`undefined` when missing).
pub fn shown(v: Option<&Value>) -> String {
    v.map_or_else(|| "undefined".to_owned(), display)
}

/// `Number(v)` for the values a post carries.
#[must_use]
pub fn to_number(v: Option<&Value>) -> f64 {
    match v {
        None | Some(Value::Object(_)) => f64::NAN,
        Some(Value::Null) => 0.0,
        Some(Value::Bool(b)) => f64::from(u8::from(*b)),
        Some(Value::Number(n)) => n.as_f64().unwrap_or(f64::NAN),
        Some(Value::String(s)) => parse_number(s),
        Some(Value::Array(a)) => match a.as_slice() {
            [] => 0.0,
            [one] => to_number(Some(one)),
            _ => f64::NAN,
        },
    }
}

fn parse_number(s: &str) -> f64 {
    let t = js::trim(s);
    match t {
        "" => 0.0,
        "Infinity" | "+Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ if t.chars().all(|c| c.is_ascii_digit() || "+-.eE".contains(c)) => {
            t.parse().unwrap_or(f64::NAN)
        }
        _ => f64::NAN,
    }
}

/// `Number(v)` when it is an integer.
#[must_use]
pub fn as_integer(v: Option<&Value>) -> Option<i64> {
    let n = to_number(v);
    if n.is_finite() && n.fract() == 0.0 {
        js::number(n).as_i64()
    } else {
        None
    }
}

/// JSON equality the way `JSON.stringify(a ?? null) === JSON.stringify(b ?? null)` compares.
#[must_use]
pub fn same(a: Option<&Value>, b: Option<&Value>) -> bool {
    let text = |v: Option<&Value>| serde_json::to_string(v.unwrap_or(&Value::Null)).ok();
    text(a) == text(b)
}

/// `a === b` for JSON values: numbers compare by value.
#[must_use]
pub fn strict_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        _ => a == b,
    }
}

/// The ids of the docs of a collection that satisfy a test.
pub fn find_ids(board: &Value, coll: &str, keep: impl Fn(&Value) -> bool) -> Vec<String> {
    section(board, coll)
        .iter()
        .filter(|(_, d)| keep(d))
        .map(|(id, _)| id.clone())
        .collect()
}
