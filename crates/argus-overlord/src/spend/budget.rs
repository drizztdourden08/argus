//! The budget rule: what the spent amount of the month means.

/// What to do about the spend of the month.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    /// At the warning share, and not warned yet this month.
    Warn,
    /// At the budget: the session pauses after its turn.
    Pause,
}

/// The rule. `warn_at` is a share of the budget (0.8 is 80 percent).
#[must_use]
pub fn verdict(spent_usd: f64, budget_usd: f64, warn_at: f64, already_warned: bool) -> Verdict {
    if spent_usd >= budget_usd {
        Verdict::Pause
    } else if !already_warned && spent_usd >= budget_usd * warn_at {
        Verdict::Warn
    } else {
        Verdict::Ok
    }
}

/// The money left, for `--max-budget-usd`. Never negative.
#[must_use]
pub fn left_usd(spent_usd: f64, budget_usd: f64) -> f64 {
    (budget_usd - spent_usd).max(0.0)
}
