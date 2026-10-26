//! JavaScript text forms the server client verbs print: cells of a table, numbers, collapsed
//! white space and the `Number(...)` of an option.

use argus_board::json::{display, to_number};
use argus_core::js;
use serde_json::Value;

use crate::cli::Invocation;

/// `String(c ?? '')`: a table cell, empty for a missing or null value.
#[must_use]
pub fn cell(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(other) => display(other),
    }
}

/// A number as JavaScript prints it.
#[must_use]
pub fn num(n: f64) -> String {
    js::string(&js::number(n))
}

/// `Number(text)`.
#[must_use]
pub fn number_of(text: &str) -> f64 {
    to_number(Some(&Value::String(text.to_owned())))
}

/// `Number.isInteger(n)`.
#[must_use]
pub fn is_integer(n: f64) -> bool {
    n.is_finite() && n.fract() == 0.0
}

/// An integer within `min..=max`, as the verbs bound their numbers.
#[must_use]
pub fn int_within(n: f64, min: f64, max: f64) -> Option<i64> {
    let whole = is_integer(n) && (min..=max).contains(&n);
    #[allow(clippy::cast_possible_truncation)]
    whole.then_some(n as i64)
}

/// Whether `c` is white space to a JavaScript `\s`.
#[must_use]
pub fn is_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// `text.replace(/\s+/g, ' ').trim()`.
#[must_use]
pub fn collapse(text: &str) -> String {
    text.split(is_space)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// `Math.round` of a non-negative number.
#[must_use]
pub fn round(n: f64) -> f64 {
    (n + 0.5).floor()
}

/// `value[field]` is JavaScript truthy.
#[must_use]
pub fn truthy(v: &Value, field: &str) -> bool {
    js::truthy(&v[field])
}

/// `console.log(lines.join('\n'))` for lines built as owned texts.
pub fn say_all(inv: &mut Invocation, lines: &[String]) {
    let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
    inv.say(&refs);
}

/// An amount and its unit as the duration options are typed: digits, an optional fraction, white
/// space, then a unit (`6h`, `30 m`, `1.5d`). Returns the number and the unit in lower case,
/// empty when there is none.
#[must_use]
pub fn amount_and_unit(text: &str) -> Option<(f64, String)> {
    let t = js::trim(text);
    let whole = t.find(|c: char| !c.is_ascii_digit()).unwrap_or(t.len());
    if whole == 0 {
        return None;
    }
    let mut end = whole;
    if let Some(fraction) = t[whole..].strip_prefix('.') {
        let digits = fraction
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(fraction.len());
        if digits == 0 {
            return None;
        }
        end = whole + 1 + digits;
    }
    let unit = t[end..].trim_start_matches(is_space).to_lowercase();
    Some((t[..end].parse().ok()?, unit))
}

/// A project key: lowercase letters, digits and dashes, not starting with a dash.
#[must_use]
pub fn is_key(text: &str) -> bool {
    let plain = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    let mut chars = text.chars();
    chars.next().is_some_and(plain) && chars.all(|c| plain(c) || c == '-')
}
