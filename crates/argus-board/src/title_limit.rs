//! The item title limit of a board: `titleMax` on the board, between [`MIN`] and [`MAX`]
//! characters. A board without one keeps the registry's own limit.

use serde_json::Value;

/// The least a board may set.
pub const MIN: u64 = 20;

/// The most a board may set.
pub const MAX: u64 = 200;

/// The board's title limit, `None` when it sets none.
#[must_use]
pub fn of(board: &Value) -> Option<usize> {
    board["titleMax"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
}
