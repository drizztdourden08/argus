//! The approvals of the hosted Overlord: the child asks before a tool the settings do not allow,
//! the owner answers in the Overlord tab, and the answer goes back in the shape the CLI expects.
//! All pure; the rows live behind the `ApprovalStore` port and the toast behind `OwnerToast`.

mod bearer;
mod config;
mod model;
mod queue;
mod reply;
mod rpc;
mod summary;
#[cfg(test)]
mod tests;

pub use bearer::bearer_ok;
pub use config::mcp_config_json;
pub use model::{Approval, Decision, Status};
pub use queue::{DecideFault, Queue};
pub use reply::{DENIED_DEFAULT, TIMED_OUT, allow, deny, reply_for};
pub use rpc::{Parsed, Prompt, parse, tool_result};
pub use summary::summarize;

/// The most approvals the queue keeps in memory (pending ones are never dropped).
pub const KEEP: usize = 100;

/// The time a held call waits, in milliseconds.
#[must_use]
pub fn timeout_ms(approval_timeout_min: u64) -> i64 {
    i64::try_from(approval_timeout_min.saturating_mul(60_000)).unwrap_or(i64::MAX)
}

/// True when a request made at `requested_ms` has waited `wait_ms` by `now_ms`.
#[must_use]
pub fn timed_out(requested_ms: i64, now_ms: i64, wait_ms: i64) -> bool {
    now_ms.saturating_sub(requested_ms) >= wait_ms
}
