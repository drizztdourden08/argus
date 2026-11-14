//! The month of a moment by the local calendar, from the local offset so the rule stays pure.

const SECS_PER_DAY: i64 = 86_400;

/// `YYYY-MM` of `unix_ms` at `offset_secs` east of UTC.
#[must_use]
pub fn month_key(unix_ms: i64, offset_secs: i32) -> String {
    let secs = unix_ms.div_euclid(1000) + i64::from(offset_secs);
    let (year, month) = civil_month(secs.div_euclid(SECS_PER_DAY));
    format!("{year:04}-{month:02}")
}

/// Year and month of a day count since 1970-01-01 (the civil-from-days algorithm).
fn civil_month(days: i64) -> (i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month)
}
