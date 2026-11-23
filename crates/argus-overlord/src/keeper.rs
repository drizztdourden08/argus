//! The keeper's backoff (I-196): how long it waits before it starts the hosted child again after a
//! failed start or a child that ended. The wait is 30 s, then 60 s, then 2 min, then 5 min at most.
//! A child that ran 10 minutes or more clears the count. All pure: the caller passes a monotonic
//! clock reading as a `Duration` since any fixed point.

use std::time::Duration;

/// How often the keeper looks.
pub const TICK: Duration = Duration::from_secs(30);

/// A child that ran this long has earned a fresh start of the schedule.
pub const STABLE: Duration = Duration::from_secs(600);

const STEPS_SECS: [u64; 4] = [30, 60, 120, 300];

/// The wait after the `failures`-th failure in a row (the first is 1).
#[must_use]
pub fn delay(failures: u32) -> Duration {
    let at = usize::try_from(failures.saturating_sub(1)).unwrap_or(usize::MAX);
    let last = STEPS_SECS.len() - 1;
    Duration::from_secs(STEPS_SECS[at.min(last)])
}

/// The keeper's memory of the child it started.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Backoff {
    failures: u32,
    wait_until: Option<Duration>,
    started_at: Option<Duration>,
}

impl Backoff {
    /// True when no wait is pending at `now`.
    #[must_use]
    pub fn may_try(&self, now: Duration) -> bool {
        self.wait_until.is_none_or(|until| now >= until)
    }

    /// Whole seconds left of the wait at `now`, rounded up; `None` when there is no wait.
    #[must_use]
    pub fn retry_in_secs(&self, now: Duration) -> Option<u64> {
        let left = self.wait_until?.saturating_sub(now);
        (!left.is_zero()).then(|| left.as_secs() + u64::from(left.subsec_nanos() > 0))
    }

    /// True while a child the keeper saw start has not been seen to end.
    #[must_use]
    pub fn tracking(&self) -> bool {
        self.started_at.is_some()
    }

    /// A child is up at `now`; the first sight of it fixes its start.
    pub fn started(&mut self, now: Duration) {
        self.started_at.get_or_insert(now);
        self.wait_until = None;
    }

    /// A start failed or the child ended at `now`: counts a failure and sets the wait. A child that
    /// ran [`STABLE`] first clears the count.
    pub fn ended(&mut self, now: Duration) {
        if let Some(from) = self.started_at.take() {
            if now.saturating_sub(from) >= STABLE {
                self.failures = 0;
            }
        }
        self.failures = self.failures.saturating_add(1);
        self.wait_until = Some(now + delay(self.failures));
    }

    /// Forgets every failure and wait, as a saved key does.
    pub fn reset(&mut self) {
        self.failures = 0;
        self.wait_until = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    // test-tag: critical
    #[test]
    fn the_delay_climbs_and_stops_at_five_minutes() {
        let waits: Vec<u64> = (1..=6).map(|n| delay(n).as_secs()).collect();
        assert_eq!(waits, vec![30, 60, 120, 300, 300, 300]);
        assert_eq!(delay(0).as_secs(), 30);
    }

    #[test]
    fn a_failure_holds_the_next_try_until_the_delay_has_passed() {
        let mut backoff = Backoff::default();
        assert!(backoff.may_try(secs(0)));
        backoff.ended(secs(100));
        assert!(!backoff.may_try(secs(129)));
        assert!(backoff.may_try(secs(130)));
        assert_eq!(backoff.retry_in_secs(secs(110)), Some(20));
        assert_eq!(backoff.retry_in_secs(secs(130)), None);
    }

    #[test]
    fn failures_in_a_row_double_the_wait() {
        let mut backoff = Backoff::default();
        backoff.ended(secs(0));
        backoff.ended(secs(30));
        assert_eq!(backoff.retry_in_secs(secs(30)), Some(60));
        backoff.ended(secs(90));
        assert_eq!(backoff.retry_in_secs(secs(90)), Some(120));
    }

    #[test]
    fn a_child_that_ran_ten_minutes_starts_the_schedule_over() {
        let mut backoff = Backoff::default();
        backoff.ended(secs(0));
        backoff.ended(secs(30));
        backoff.started(secs(100));
        backoff.ended(secs(700));
        assert_eq!(backoff.retry_in_secs(secs(700)), Some(30));
    }

    #[test]
    fn a_child_that_ran_less_keeps_the_count() {
        let mut backoff = Backoff::default();
        backoff.ended(secs(0));
        backoff.started(secs(30));
        backoff.ended(secs(200));
        assert_eq!(backoff.retry_in_secs(secs(200)), Some(60));
    }

    #[test]
    fn the_first_sight_of_a_child_fixes_its_start() {
        let mut backoff = Backoff::default();
        assert!(!backoff.tracking());
        backoff.started(secs(10));
        backoff.started(secs(500));
        assert!(backoff.tracking());
        backoff.ended(secs(600));
        assert!(!backoff.tracking());
        assert_eq!(backoff.retry_in_secs(secs(600)), Some(30));
    }

    #[test]
    fn a_saved_key_clears_the_wait() {
        let mut backoff = Backoff::default();
        backoff.ended(secs(0));
        backoff.reset();
        assert!(backoff.may_try(secs(1)));
        assert_eq!(backoff, Backoff::default());
    }
}
