//! ISO 8601 text with milliseconds, as `Date.toISOString` writes it, and back to epoch milliseconds.

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Epoch milliseconds of an ISO timestamp, or `None` when the text is not one.
#[must_use]
pub fn parse_iso(text: &str) -> Option<i64> {
    let parsed = OffsetDateTime::parse(text, &Rfc3339).ok()?;
    i64::try_from(parsed.unix_timestamp_nanos().div_euclid(1_000_000)).ok()
}

/// The ISO text of epoch milliseconds: `2026-10-23T12:00:00.000Z`.
#[must_use]
pub fn format_iso(ms: i64) -> String {
    let nanos = i128::from(ms) * 1_000_000;
    let Ok(t) = OffsetDateTime::from_unix_timestamp_nanos(nanos) else {
        return String::new();
    };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        t.year(),
        u8::from(t.month()),
        t.day(),
        t.hour(),
        t.minute(),
        t.second(),
        t.millisecond()
    )
}

/// The ISO text of a moment, or the fallback moment when the text is not a timestamp.
#[must_use]
pub fn iso_or(text: Option<&str>, fallback_ms: i64) -> String {
    format_iso(text.and_then(parse_iso).unwrap_or(fallback_ms))
}
