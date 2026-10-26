//! The clock the crate reads, and the time texts it writes.

use std::time::{SystemTime, UNIX_EPOCH};

pub use argus_core::clock::{Clock, ManualClock, SystemClock, iso_from_ms};

pub const DAY_MS: i64 = 86_400_000;
pub const HOUR_MS: i64 = 3_600_000;

/// A file time as milliseconds since the Unix epoch (whole milliseconds).
pub fn system_ms(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()).unwrap_or(i64::MAX),
        Err(before) => -i64::try_from(before.duration().as_millis()).unwrap_or(i64::MAX),
    }
}

/// `yyyymmddhhmm` in UTC, the time stamp of a backup's file name.
pub fn stamp_from_ms(ms: i64) -> String {
    iso_from_ms(ms)
        .chars()
        .filter(char::is_ascii_digit)
        .take(12)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_and_stamp_match_known_instants() {
        assert_eq!(iso_from_ms(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso_from_ms(1_761_393_600_000), "2025-10-25T12:00:00.000Z");
        assert_eq!(iso_from_ms(1_730_000_000_123), "2024-10-27T03:33:20.123Z");
        assert_eq!(stamp_from_ms(1_761_393_600_000), "202510251200");
        assert_eq!(iso_from_ms(951_782_400_000), "2000-02-29T00:00:00.000Z");
    }
}
