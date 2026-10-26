//! Millisecond timestamps and the ISO-8601 text the boards and run rows use.

/// Milliseconds in a day.
pub const DAY_MS: i64 = 86_400_000;
const HOUR_MS: i64 = 3_600_000;
const MINUTE_MS: i64 = 60_000;

/// `2026-10-24T00:00:00.000Z` for a millisecond timestamp.
pub fn format_iso(ms: i64) -> String {
    argus_core::clock::iso_from_ms(ms)
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

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year.rem_euclid(400);
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
