//! K1: the short age text.

// test-tag: critical

use argus_core::clock::{age_text, end_of_day_ms, parse_ms};

#[test]
fn a_bare_date_ends_at_the_next_local_midnight() {
    let utc_next_midnight = parse_ms("2026-11-21").unwrap();
    assert_eq!(end_of_day_ms("2026-11-20", 0), Some(utc_next_midnight));
    let five_hours = 5 * 3_600_000;
    assert_eq!(
        end_of_day_ms("2026-11-20", -5 * 3600),
        Some(utc_next_midnight + five_hours)
    );
    assert_eq!(
        parse_ms("2026-11-21T05:00:00Z"),
        end_of_day_ms("2026-11-20", -5 * 3600)
    );
}

#[test]
fn a_full_timestamp_is_not_a_bare_date() {
    assert_eq!(end_of_day_ms("2026-11-20T10:00:00Z", -5 * 3600), None);
    assert_eq!(end_of_day_ms("1795000000000", 0), None);
    assert!(parse_ms("2026-11-20T10:00:00Z").is_some());
}

#[test]
fn ages_read_short() {
    assert_eq!(age_text(10 * 60_000), "10m");
    assert_eq!(age_text(0), "1m");
    assert_eq!(age_text(60 * 60_000), "1h");
    assert_eq!(age_text(47 * 3_600_000), "47h");
    assert_eq!(age_text(3 * 86_400_000), "3d");
}
