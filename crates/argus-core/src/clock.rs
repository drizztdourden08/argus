//! Time in the forms the Node server writes (epoch milliseconds and `Date.toISOString`) and the
//! injected clock every service reads.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, PrimitiveDateTime, UtcOffset};

/// A source of epoch milliseconds, injected so tests can drive time.
pub trait Clock: Send + Sync {
    /// The current time in epoch milliseconds.
    fn now_ms(&self) -> i64;
}

/// The wall clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> i64 {
        now_ms()
    }
}

/// A clock that only moves when told to; clones share one time.
#[derive(Debug, Clone, Default)]
pub struct ManualClock(Arc<AtomicI64>);

impl ManualClock {
    /// A manual clock showing `ms`.
    #[must_use]
    pub fn at(ms: i64) -> Self {
        Self(Arc::new(AtomicI64::new(ms)))
    }

    /// Jump to `ms`.
    pub fn set(&self, ms: i64) {
        self.0.store(ms, Ordering::SeqCst);
    }

    /// Move forward by `ms`.
    pub fn advance(&self, ms: i64) {
        self.0.fetch_add(ms, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

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

/// An age as `40m` under an hour, `5h` under two days, else `2d`.
#[must_use]
pub fn age_text(ms: i64) -> String {
    let minutes = ((ms + 30_000).div_euclid(60_000)).max(1);
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = (minutes + 30).div_euclid(60);
    if hours < 48 {
        format!("{hours}h")
    } else {
        format!("{}d", (hours + 12).div_euclid(24))
    }
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

/// The end of a day written as a bare `YYYY-MM-DD`: the start of the day after, in the zone `offset_seconds` east of
/// UTC. `None` when the text is anything else (a full timestamp keeps its meaning through [`parse_ms`]).
#[must_use]
pub fn end_of_day_ms(text: &str, offset_seconds: i32) -> Option<i64> {
    let day = format_description!("[year]-[month]-[day]");
    let date = Date::parse(text.trim(), &day).ok()?;
    let offset = UtcOffset::from_whole_seconds(offset_seconds).ok()?;
    Some(millis(date.next_day()?.midnight().assume_offset(offset)))
}

/// This PC's offset from UTC in seconds east, UTC when the system will not say.
#[must_use]
pub fn local_offset_seconds() -> i32 {
    UtcOffset::current_local_offset().map_or(0, UtcOffset::whole_seconds)
}

fn millis(dt: OffsetDateTime) -> i64 {
    i64::try_from(dt.unix_timestamp_nanos() / 1_000_000).unwrap_or(i64::MAX)
}
