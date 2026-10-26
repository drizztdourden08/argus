//! Rate limits over the toasts let through. A ping over a limit is never refused: it is recorded
//! without a toast, and at most one summary toast a minute per thread says how many.

use std::collections::{HashMap, VecDeque};

/// Toasts one thread gets in a minute.
pub const THREAD_PER_MINUTE: usize = 3;
/// Toasts one thread gets in an hour.
pub const THREAD_PER_HOUR: usize = 20;
/// Toasts everyone together get in a minute.
pub const GLOBAL_PER_MINUTE: usize = 8;
/// The least time between two summary toasts of one thread, in milliseconds.
pub const SUMMARY_EVERY_MS: i64 = 60_000;

const MINUTE: i64 = 60_000;
const HOUR: i64 = 3_600_000;
const FORGET_ABOVE: usize = 500;

#[derive(Debug, Default)]
struct Held {
    count: usize,
    summary_at: Option<i64>,
}

/// The counters, in memory: a restart forgets them.
#[derive(Debug, Default)]
pub struct Limiter {
    threads: HashMap<String, Vec<i64>>,
    everyone: VecDeque<i64>,
    held: HashMap<String, Held>,
}

impl Limiter {
    /// None when a toast may be shown now (and the attempt is counted), else the sentence naming
    /// the limit that was hit.
    pub fn admit(&mut self, key: &str, now: i64) -> Option<String> {
        while self.everyone.front().is_some_and(|at| now - at >= MINUTE) {
            self.everyone.pop_front();
        }
        let mine = self.threads.entry(key.to_owned()).or_default();
        mine.retain(|at| now - at < HOUR);
        let last_minute = mine.iter().filter(|at| now - **at < MINUTE).count();
        let why = if last_minute >= THREAD_PER_MINUTE {
            Some(format!("you pinged {} times in a minute", last_minute + 1))
        } else if mine.len() >= THREAD_PER_HOUR {
            Some(format!("you pinged {} times in an hour", mine.len() + 1))
        } else if self.everyone.len() >= GLOBAL_PER_MINUTE {
            let all = self.everyone.len() + 1;
            Some(format!(
                "{all} notifications in a minute across all threads"
            ))
        } else {
            None
        };
        if why.is_none() {
            mine.push(now);
            self.everyone.push_back(now);
            self.forget_idle(now);
        }
        why
    }

    /// Counts one limited notification of the thread; the number of limited ones since the
    /// previous summary when a summary toast is due now, else none.
    pub fn hold(&mut self, key: &str, now: i64) -> Option<usize> {
        let held = self.held.entry(key.to_owned()).or_default();
        held.count += 1;
        if held
            .summary_at
            .is_some_and(|at| now - at < SUMMARY_EVERY_MS)
        {
            return None;
        }
        held.summary_at = Some(now);
        Some(std::mem::take(&mut held.count))
    }

    fn forget_idle(&mut self, now: i64) {
        if self.threads.len() > FORGET_ABOVE {
            self.threads
                .retain(|_, list| list.iter().any(|at| now - at < HOUR));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Limiter, MINUTE};

    #[test]
    fn the_fourth_ping_of_a_minute_is_limited_and_a_minute_later_it_is_free() {
        let mut limiter = Limiter::default();
        let results: Vec<_> = (0..4).map(|_| limiter.admit("a/b", 0)).collect();
        assert_eq!(
            results,
            [
                None,
                None,
                None,
                Some("you pinged 4 times in a minute".into())
            ]
        );
        assert_eq!(limiter.admit("a/b", MINUTE + 1), None);
    }

    #[test]
    fn a_summary_is_due_once_a_minute_and_counts_what_it_covers() {
        let mut limiter = Limiter::default();
        assert_eq!(limiter.hold("a/b", 0), Some(1));
        assert_eq!(limiter.hold("a/b", 1_000), None);
        assert_eq!(limiter.hold("a/b", 2_000), None);
        assert_eq!(limiter.hold("a/b", MINUTE), Some(3));
        assert_eq!(limiter.hold("c/d", 1_000), Some(1));
    }
}
