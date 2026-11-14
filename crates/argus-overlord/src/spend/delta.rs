//! A `result` event carries the cumulative cost of its child process, so one turn costs the
//! difference to the last value of the same process.

/// The last cumulative value the ledger holds for a session.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Last {
    pub process_seq: u64,
    pub cumulative_usd: f64,
}

/// One ledger row.
#[derive(Debug, Clone, PartialEq)]
pub struct SpendRow {
    pub at: String,
    pub session_id: String,
    pub process_seq: u64,
    pub cumulative_usd: f64,
    pub delta_usd: f64,
    pub month: String,
}

/// The cost of this turn. A new process starts from 0 (so does a resumed session in a new
/// process), and a value below the last one of the same process is a new baseline. Never negative.
#[must_use]
pub fn delta(last: Option<Last>, process_seq: u64, cumulative_usd: f64) -> f64 {
    let base = match last {
        Some(l) if l.process_seq == process_seq && cumulative_usd >= l.cumulative_usd => {
            l.cumulative_usd
        }
        _ => 0.0,
    };
    let spent = cumulative_usd - base;
    if spent.is_finite() && spent > 0.0 {
        spent
    } else {
        0.0
    }
}
