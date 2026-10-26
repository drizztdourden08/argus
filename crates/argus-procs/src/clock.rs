//! Wall clock and ISO timestamps in the shape the dashboard stores.

use chrono::{DateTime, SecondsFormat};
use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since the Unix epoch.
#[must_use]
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

/// `2026-10-25T12:00:00.000Z` for an epoch in milliseconds.
#[must_use]
pub fn iso(ms: i64) -> String {
    DateTime::from_timestamp_millis(ms)
        .unwrap_or_default()
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// The epoch in milliseconds of an ISO timestamp, or `None` when it does not parse.
#[must_use]
pub fn parse_iso(text: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(text.trim())
        .ok()
        .map(|d| d.timestamp_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_round_trips() {
        let text = iso(1_761_393_600_000);
        assert_eq!(text, "2025-10-25T12:00:00.000Z");
        assert_eq!(parse_iso(&text), Some(1_761_393_600_000));
        assert_eq!(parse_iso("nope"), None);
    }
}
