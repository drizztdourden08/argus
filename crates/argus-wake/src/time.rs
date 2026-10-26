//! ISO timestamps the way the Node dashboard writes them (`2026-10-22T10:00:00.000Z`).

use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

const ISO: &[FormatItem<'static>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

/// Epoch milliseconds as an ISO string with millisecond precision.
#[must_use]
pub fn iso(ms: i64) -> String {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * 1_000_000)
        .ok()
        .and_then(|at| at.format(ISO).ok())
        .unwrap_or_default()
}

/// Epoch milliseconds of an ISO string, or `None` when it does not parse.
#[must_use]
pub fn parse_ms(text: &str) -> Option<i64> {
    let at = OffsetDateTime::parse(text, &Rfc3339).ok()?;
    i64::try_from(at.unix_timestamp_nanos() / 1_000_000).ok()
}

/// Epoch milliseconds of an optional ISO string; 0 when absent or unparsable.
#[must_use]
pub fn ms_or_zero(text: Option<&str>) -> i64 {
    text.and_then(parse_ms).unwrap_or(0)
}
