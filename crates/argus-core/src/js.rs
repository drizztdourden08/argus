//! JavaScript semantics the Node server relies on, so Argus answers the same: string length in
//! UTF-16 units, `String.prototype.trim`, numbers that print like `JSON.stringify`, and
//! truthiness of JSON values.

use serde_json::{Number, Value};

/// `s.length` in JavaScript: UTF-16 code units.
#[must_use]
pub fn len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// `s.trim()` in JavaScript: white space and line terminators, the BOM included.
#[must_use]
pub fn trim(s: &str) -> &str {
    s.trim_matches(is_js_space)
}

fn is_js_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// The first `n` UTF-16 units of `s`, never splitting a character.
#[must_use]
pub fn slice(s: &str, n: usize) -> &str {
    let mut units = 0;
    for (i, c) in s.char_indices() {
        units += c.len_utf16();
        if units > n {
            return &s[..i];
        }
    }
    s
}

/// A number as `JSON.stringify` prints it: integral values without a fraction.
#[must_use]
pub fn number(n: f64) -> Value {
    if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
        #[allow(clippy::cast_possible_truncation)]
        return Value::Number(Number::from(n as i64));
    }
    Number::from_f64(n).map_or(Value::Null, Value::Number)
}

/// JavaScript truthiness of a JSON value.
#[must_use]
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0 && !f.is_nan()),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// `String(v)` for the JSON values a board holds.
#[must_use]
pub fn string(v: &Value) -> String {
    match v {
        Value::Null => "null".to_owned(),
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(string).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".to_owned(),
        other => other.to_string(),
    }
}
