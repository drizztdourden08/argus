//! Limit confirmed: an account is at its limit only after a real refusal (a 429 or a
//! `rate_limit_error` the wake relay saw for it), never because its usage bar reads 100. The
//! refusal is kept in memory and ends with the account's limit window.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use argus_accounts::binding_limit;
use argus_core::clock::{now_ms, parse_ms};
use serde_json::{Value, json};

use super::AccountsService;

/// How long a refusal counts when the usage read names no reset: the five-hour window.
const WINDOW_MS: i64 = 5 * 3_600_000;

/// One refusal: when it was seen and when its window ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Refusal {
    at_ms: i64,
    until_ms: i64,
}

/// The refusals seen, by account key.
#[derive(Default)]
pub(crate) struct Refusals(Mutex<HashMap<String, Refusal>>);

impl Refusals {
    fn record(&self, key: &str, at_ms: i64, until_ms: i64) {
        let mut all = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        all.insert(key.to_owned(), Refusal { at_ms, until_ms });
    }

    /// The `limit` of an account at `now_ms`: confirmed while the window of its last refusal
    /// lasts.
    fn shown(&self, key: &str, now_ms: i64) -> Value {
        let all = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        match all.get(key).filter(|r| now_ms < r.until_ms) {
            Some(r) => json!({ "confirmed": true, "atMs": r.at_ms, "untilMs": r.until_ms }),
            None => json!({ "confirmed": false, "atMs": null, "untilMs": null }),
        }
    }
}

impl AccountsService {
    /// Notes that the account `key` was refused with a rate limit just now. The refusal lasts
    /// until its binding limit resets. Only the key is kept, never a login.
    pub(crate) fn record_refusal(&self, key: &str) {
        let now = now_ms();
        let reset = self
            .latest_usage()
            .get(key)
            .and_then(|p| {
                binding_limit(&p.usage).and_then(|l| l.resets_at.as_deref().and_then(parse_ms))
            })
            .filter(|at| *at > now);
        self.refusals
            .record(key, now, reset.unwrap_or(now + WINDOW_MS));
    }

    /// The `limit` object of the account `key`.
    pub(crate) fn limit_shown(&self, key: &str) -> Value {
        self.refusals.shown(key, now_ms())
    }

    /// Whether the account `key` has a confirmed refusal now.
    pub(crate) fn limit_confirmed(&self, key: &str) -> bool {
        self.limit_shown(key)["confirmed"] == json!(true)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusal_is_confirmed_until_its_window_ends() {
        let refusals = Refusals::default();
        assert_eq!(refusals.shown("a", 5)["confirmed"], json!(false));
        refusals.record("a", 10, 100);
        let shown = refusals.shown("a", 99);
        assert_eq!(
            shown,
            json!({ "confirmed": true, "atMs": 10, "untilMs": 100 })
        );
        assert_eq!(refusals.shown("a", 100)["confirmed"], json!(false));
        assert_eq!(refusals.shown("b", 99)["confirmed"], json!(false));
    }
}
