//! `config.buildCache`: the grace period of an unused entry, and when the first prune runs.

use serde_json::Value;

use crate::Services;

const GRACE_DAYS: u32 = 14;
const START_DELAY_MS: u64 = 60_000;

fn setting(svc: &Services) -> Value {
    svc.config()
        .extra
        .get("buildCache")
        .cloned()
        .unwrap_or(Value::Null)
}

/// `graceDays`: a whole number of at least one, else 14.
pub(super) fn grace_days(svc: &Services) -> u32 {
    setting(svc)["graceDays"]
        .as_u64()
        .filter(|days| *days >= 1)
        .and_then(|days| u32::try_from(days).ok())
        .unwrap_or(GRACE_DAYS)
}

/// `startDelayMs`: how long after the start the first prune waits, so the worktree scan has run.
pub(super) fn start_delay_ms(svc: &Services) -> u64 {
    setting(svc)["startDelayMs"]
        .as_u64()
        .unwrap_or(START_DELAY_MS)
}
