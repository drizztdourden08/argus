//! Times for the protocol files: milliseconds since the epoch, and ISO 8601 text in UTC.
// test-tag: critical

use std::time::{SystemTime, UNIX_EPOCH};

const DAY_MS: u64 = 86_400_000;

/// Milliseconds since 1970-01-01 UTC, zero for a time before it.
#[must_use]
pub fn epoch_ms(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// The time as `2026-10-22T10:00:00.000Z`.
#[must_use]
pub fn iso_utc(time: SystemTime) -> String {
    let ms = epoch_ms(time);
    let (year, month, day) = civil(i64::try_from(ms / DAY_MS).unwrap_or(0));
    let in_day = ms % DAY_MS;
    let (hour, minute) = (in_day / 3_600_000, in_day / 60_000 % 60);
    let (second, milli) = (in_day / 1000 % 60, in_day % 1000);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}Z")
}

fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (year_of_era + era * 400 + i64::from(month <= 2), month, day)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::iso_utc;

    fn at(seconds: u64, millis: u64) -> String {
        iso_utc(UNIX_EPOCH + Duration::from_millis(seconds * 1000 + millis))
    }

    #[test]
    fn known_instants_print_as_iso_text() {
        assert_eq!(at(0, 0), "1970-01-01T00:00:00.000Z");
        assert_eq!(at(1_000_000_000, 7), "2001-09-09T01:46:40.007Z");
        assert_eq!(at(1_709_164_800, 0), "2024-02-29T00:00:00.000Z");
        assert_eq!(at(1_792_670_400, 999), "2026-10-22T12:00:00.999Z");
    }
}
