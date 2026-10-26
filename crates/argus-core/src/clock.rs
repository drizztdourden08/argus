//! Time in the forms the Node server writes: epoch milliseconds and `Date.toISOString`.

use std::time::{SystemTime, UNIX_EPOCH};

use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime};

/// Milliseconds since the epoch, now.
#[must_use]
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// `new Date().toISOString()`: `2026-10-26T12:00:00.000Z`.
#[must_use]
pub fn now_iso() -> String {
    iso_from_ms(now_ms())
}

/// The ISO form of an epoch millisecond, always UTC with three decimals.
#[must_use]
pub fn iso_from_ms(ms: i64) -> String {
    let nanos = i128::from(ms) * 1_000_000;
    let t = OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    let fmt =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
    t.format(&fmt).unwrap_or_default()
}

/// A file-name-safe stamp: `20261017T193800Z`.
#[must_use]
pub fn file_stamp() -> String {
    now_iso()
        .replace(['-', ':'], "")
        .split('.')
        .next()
        .unwrap_or_default()
        .to_owned()
        + "Z"
}

/// `Date.parse` for the forms boards carry: RFC 3339 and plain dates. `None` when it is not a
/// date (`NaN` in JavaScript).
#[must_use]
pub fn parse_ms(text: &str) -> Option<i64> {
    let t = text.trim();
    if let Ok(dt) = OffsetDateTime::parse(t, &Rfc3339) {
        return Some(millis(dt));
    }
    let local = format_description!("[year]-[month]-[day]T[hour]:[minute]:[second]");
    if let Ok(dt) = PrimitiveDateTime::parse(t, &local) {
        return Some(millis(dt.assume_utc()));
    }
    let day = format_description!("[year]-[month]-[day]");
    Date::parse(t, &day)
        .ok()
        .map(|d| millis(d.midnight().assume_utc()))
}

fn millis(dt: OffsetDateTime) -> i64 {
    i64::try_from(dt.unix_timestamp_nanos() / 1_000_000).unwrap_or(i64::MAX)
}
