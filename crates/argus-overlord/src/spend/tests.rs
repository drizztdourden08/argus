use super::*;

fn at(seq: u64, usd: f64) -> Option<Last> {
    Some(Last {
        process_seq: seq,
        cumulative_usd: usd,
    })
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

// test-tag: critical
#[test]
fn a_cumulative_value_costs_the_step_from_the_last_one() {
    assert!(near(delta(None, 1, 0.064), 0.064));
    assert!(near(delta(at(1, 0.064), 1, 0.0668), 0.0028));
}

#[test]
fn a_new_process_starts_from_zero() {
    assert!(near(delta(at(1, 0.5), 2, 0.07), 0.07));
}

#[test]
fn a_lower_value_in_the_same_process_is_a_new_baseline() {
    assert!(near(delta(at(1, 0.5), 1, 0.2), 0.2));
}

#[test]
fn a_resumed_session_in_a_new_process_counts_its_whole_value() {
    assert!(near(delta(at(3, 1.25), 4, 0.05), 0.05));
}

#[test]
fn a_bad_number_costs_nothing() {
    assert!(near(delta(None, 1, f64::NAN), 0.0));
    assert!(near(delta(None, 1, -1.0), 0.0));
}

#[test]
fn the_month_follows_the_local_calendar() {
    let utc_new_year = 1_767_225_600_000;
    assert_eq!(month_key(utc_new_year, 0), "2026-01");
    assert_eq!(month_key(utc_new_year - 1, 0), "2025-12");
    assert_eq!(month_key(utc_new_year, -5 * 3600), "2025-12");
    assert_eq!(month_key(utc_new_year - 1, 3600), "2026-01");
    assert_eq!(month_key(0, 0), "1970-01");
    assert_eq!(month_key(-1, 0), "1969-12");
}

#[test]
fn the_warning_comes_once_a_month() {
    assert_eq!(verdict(79.0, 100.0, 0.8, false), Verdict::Ok);
    assert_eq!(verdict(80.0, 100.0, 0.8, false), Verdict::Warn);
    assert_eq!(verdict(90.0, 100.0, 0.8, true), Verdict::Ok);
}

#[test]
fn the_budget_pauses_even_after_a_warning() {
    assert_eq!(verdict(100.0, 100.0, 0.8, true), Verdict::Pause);
    assert_eq!(verdict(130.0, 100.0, 0.8, false), Verdict::Pause);
}

#[test]
fn the_money_left_is_never_negative() {
    assert!(near(left_usd(40.0, 100.0), 60.0));
    assert!(near(left_usd(130.0, 100.0), 0.0));
}
