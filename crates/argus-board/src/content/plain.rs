//! The small kinds: a slug, one of a list of values, a date, a flag.

use argus_core::js;
use serde_json::Value;

use super::{Rejection, reject, within};
use crate::registry::{Field, registry};

/// A slug: lower case words joined by hyphens, within the field's max.
pub(super) fn slug(field: &str, spec: &Field, text: &str) -> Result<Value, Rejection> {
    let text = js::trim(text);
    within(field, text, spec.max)?;
    if !registry().content.slug.pattern.0.is_match(text) {
        return Err(reject(
            "bad-slug",
            format!("{field} is lower case words joined by hyphens, like release-2."),
        ));
    }
    Ok(Value::String(text.to_owned()))
}

/// One of the field's values.
pub(super) fn choice(field: &str, spec: &Field, text: &str) -> Result<Value, Rejection> {
    let text = js::trim(text);
    if !spec.values.iter().any(|v| v == text) {
        return Err(reject(
            "bad-enum",
            format!("{field} is one of {}.", spec.values.join(", ")),
        ));
    }
    Ok(Value::String(text.to_owned()))
}

/// A `YYYY-MM-DD` date that exists on the calendar.
pub(super) fn date(field: &str, text: &str) -> Result<Value, Rejection> {
    let text = js::trim(text);
    if !registry().content.date.pattern.0.is_match(text) || !on_calendar(text) {
        return Err(reject(
            "bad-date",
            format!("{field} is a date like 2026-11-02."),
        ));
    }
    Ok(Value::String(text.to_owned()))
}

/// Whether the year, month and day of an `YYYY-MM-DD` text name a real day.
fn on_calendar(text: &str) -> bool {
    let mut parts = text.split('-').filter_map(|p| p.parse::<u32>().ok());
    let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=days).contains(&day)
}

/// A flag: `true` or `false`, or the words "true" and "false" a command line gives.
pub(super) fn flag(field: &str, value: &Value) -> Result<Value, Rejection> {
    let known = match value {
        Value::Bool(on) => Some(*on),
        Value::String(text) => match js::trim(text) {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    };
    known
        .map(Value::Bool)
        .ok_or_else(|| reject("bad-flag", format!("{field} is true or false.")))
}
