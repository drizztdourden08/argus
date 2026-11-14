//! `config.overlord`: the parts the spend meter reads, with the plan's defaults.

use serde_json::Value;

use crate::Services;

const DEFAULT_BUDGET_USD: f64 = 100.0;
const DEFAULT_WARN_AT: f64 = 0.8;

/// The monthly budget and the share of it that warns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Budget {
    pub budget_usd: f64,
    pub warn_at: f64,
}

fn share(setting: &Value, name: &str, default: f64, ok: impl Fn(f64) -> bool) -> f64 {
    setting[name]
        .as_f64()
        .filter(|v| v.is_finite() && ok(*v))
        .unwrap_or(default)
}

/// `budgetUsd` (default 100, not below 0) and `warnAt` (default 0.8, between 0 and 1).
pub(crate) fn budget(svc: &Services) -> Budget {
    let config = svc.config();
    let setting = config.extra.get("overlord").unwrap_or(&Value::Null);
    Budget {
        budget_usd: share(setting, "budgetUsd", DEFAULT_BUDGET_USD, |v| v >= 0.0),
        warn_at: share(setting, "warnAt", DEFAULT_WARN_AT, |v| (0.0..=1.0).contains(&v)),
    }
}
