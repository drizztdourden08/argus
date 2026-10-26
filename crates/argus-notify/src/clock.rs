//! The time formats of the records.

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

const NANOS_PER_MS: i128 = 1_000_000;

/// `2026-10-26T12:00:00.000Z`, the form the records carry (the Node `toISOString`).
#[must_use]
pub fn iso(ms: i64) -> String {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(ms) * NANOS_PER_MS)
        .ok()
        .and_then(|moment| {
            moment
                .format(format_description!(
                    "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
                ))
                .ok()
        })
        .unwrap_or_default()
}

/// The milliseconds of an ISO 8601 time, if it is one.
#[must_use]
pub fn parse_ms(value: &str) -> Option<i64> {
    let moment = OffsetDateTime::parse(value, &Rfc3339).ok()?;
    i64::try_from(moment.unix_timestamp_nanos() / NANOS_PER_MS).ok()
}

#[cfg(test)]
mod tests {
    use super::{iso, parse_ms};

    #[test]
    fn a_time_round_trips_through_the_record_format() {
        let ms = 1_793_016_000_123;
        assert_eq!(iso(ms), "2026-10-26T12:00:00.123Z");
        assert_eq!(parse_ms("2026-10-26T12:00:00.123Z"), Some(ms));
    }

    #[test]
    fn a_value_that_is_not_a_time_is_none() {
        assert_eq!(parse_ms("x"), None);
        assert_eq!(parse_ms(""), None);
    }
}
