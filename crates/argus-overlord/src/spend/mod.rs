//! The spend meter: cumulative cost to deltas, the month key, and the budget rule. All pure; the
//! rows live behind the `SpendLedger` port.

mod budget;
mod delta;
mod month;
#[cfg(test)]
mod tests;

pub use budget::{Verdict, left_usd, verdict};
pub use delta::{Last, SpendRow, delta};
pub use month::month_key;
