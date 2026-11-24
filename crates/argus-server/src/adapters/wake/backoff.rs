//! How long a lead whose headless resume failed is left alone: 2 minutes after the first failure, doubling
//! to 30 minutes, and back to nothing after a success.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::clock::iso_from_ms;
use argus_wake::relay::ThreadKey;

/// The wait after the first failure.
pub const BASE_MS: i64 = 120_000;
/// The longest wait.
pub const CAP_MS: i64 = 1_800_000;

/// The wait after `fails` failures in a row (at least one).
#[must_use]
pub fn wait_ms(fails: u32) -> i64 {
    let doublings = fails.saturating_sub(1).min(16);
    BASE_MS.saturating_mul(1 << doublings).min(CAP_MS)
}

/// The time of day of `ms`, as `HH:MM UTC`.
#[must_use]
pub fn clock_text(ms: i64) -> String {
    let iso = iso_from_ms(ms);
    format!("{} UTC", iso.get(11..16).unwrap_or("?"))
}

#[derive(Debug, Clone, Copy)]
struct State {
    fails: u32,
    retry_at: i64,
}

/// The failures of each lead and when it may be tried again.
#[derive(Debug, Default)]
pub struct Backoff(Mutex<HashMap<ThreadKey, State>>);

impl Backoff {
    fn map(&self) -> MutexGuard<'_, HashMap<ThreadKey, State>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Records a failure at `now` and returns when the lead may be tried again (epoch ms).
    pub fn fail(&self, key: &ThreadKey, now: i64) -> i64 {
        let mut map = self.map();
        let fails = map.get(key).map_or(1, |s| s.fails.saturating_add(1));
        let retry_at = now.saturating_add(wait_ms(fails));
        map.insert(key.clone(), State { fails, retry_at });
        retry_at
    }

    /// Forgets the failures of a lead.
    pub fn reset(&self, key: &ThreadKey) {
        self.map().remove(key);
    }

    /// When the lead may be tried again, while that is still ahead of `now`.
    pub fn waiting(&self, key: &ThreadKey, now: i64) -> Option<i64> {
        self.map()
            .get(key)
            .map(|s| s.retry_at)
            .filter(|at| *at > now)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_starts_at_two_minutes_doubles_and_stops_at_thirty() {
        let minutes: Vec<i64> = (1..=7).map(|n| wait_ms(n) / 60_000).collect();
        assert_eq!(minutes, vec![2, 4, 8, 16, 30, 30, 30]);
        assert_eq!(wait_ms(u32::MAX), CAP_MS);
    }

    #[test]
    fn a_lead_waits_until_its_retry_time_and_a_success_clears_it() {
        let backoff = Backoff::default();
        let key = ThreadKey::new("p", "t");
        assert_eq!(backoff.waiting(&key, 0), None);
        assert_eq!(backoff.fail(&key, 1000), 1000 + BASE_MS);
        let before = 1000 + BASE_MS - 1;
        assert_eq!(backoff.waiting(&key, before), Some(1000 + BASE_MS));
        assert_eq!(backoff.waiting(&key, 1000 + BASE_MS), None);
        assert_eq!(backoff.fail(&key, 5000), 5000 + 2 * BASE_MS);
        backoff.reset(&key);
        assert_eq!(backoff.fail(&key, 9000), 9000 + BASE_MS);
    }

    #[test]
    fn the_clock_text_is_hours_and_minutes() {
        assert_eq!(clock_text(0), "00:00 UTC");
        assert_eq!(clock_text(13 * 3_600_000 + 5 * 60_000), "13:05 UTC");
    }
}
