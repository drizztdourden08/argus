//! Millisecond timestamps and the ISO-8601 text the boards and run rows use.

use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds in a day.
pub const DAY_MS: i64 = 86_400_000;
const HOUR_MS: i64 = 3_600_000;
const MINUTE_MS: i64 = 60_000;

/// The wall clock in milliseconds since the Unix epoch.
pub fn system_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

/// `2026-10-24T00:00:00.000Z` for a millisecond timestamp.
pub fn format_iso(ms: i64) -> String {
    let (year, month, day) = civil_from_days(ms.div_euclid(DAY_MS));
    let rest = ms.rem_euclid(DAY_MS);
    let hour = rest / HOUR_MS;
    let minute = rest / MINUTE_MS % 60;
    let second = rest / 1000 % 60;
    let milli = rest % 1000;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milli:03}Z")
}

/// Milliseconds for `YYYY-MM-DDTHH:MM:SS[.fff]Z`, or `None` for anything else.
pub fn parse_iso(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let separators_ok = bytes.get(4) == Some(&b'-')
        && bytes.get(7) == Some(&b'-')
        && matches!(bytes.get(10), Some(b'T' | b' '))
        && bytes.get(13) == Some(&b':')
        && bytes.get(16) == Some(&b':');
    if !separators_ok {
        return None;
    }
    let days = days_from_civil(
        number(text, 0..4)?,
        number(text, 5..7)?,
        number(text, 8..10)?,
    );
    let clock = number(text, 11..13)? * HOUR_MS
        + number(text, 14..16)? * MINUTE_MS
        + number(text, 17..19)? * 1000;
    Some(days * DAY_MS + clock + fraction(text.get(19..)?))
}

fn number(text: &str, range: std::ops::Range<usize>) -> Option<i64> {
    let part = text.get(range)?;
    if part.bytes().all(|b| b.is_ascii_digit()) {
        part.parse().ok()
    } else {
        None
    }
}

fn fraction(rest: &str) -> i64 {
    let digits: String = rest
        .strip_prefix('.')
        .map(|f| f.chars().take_while(char::is_ascii_digit).take(3).collect())
        .unwrap_or_default();
    let scale = 10_i64.pow(3 - u32::try_from(digits.len()).unwrap_or(3));
    digits.parse::<i64>().unwrap_or(0) * scale
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
