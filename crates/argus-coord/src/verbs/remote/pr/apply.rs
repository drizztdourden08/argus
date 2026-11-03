//! A pull request op applied to the board of the project, as the caller.

use argus_board::ops::{Status, apply_op};
use serde_json::Value;

use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::as_caller;
use crate::verbs::shared::caller::who_of;

/// What an applied op says.
pub(super) struct Applied {
    /// The id of the record the op made, for an open.
    pub created: Option<String>,
}

/// Applies `op` under the board lock.
///
/// # Errors
/// The board is missing or locked, or the op is refused with its sentence.
pub(super) fn apply(call: &mut Call<'_>, op: &Value) -> Result<Applied, CoordError> {
    as_caller(call, |board, who, _| {
        let res = apply_op(board, op, &who_of(who), None);
        if res.status != Status::Applied {
            return Err(CoordError::refuse(res.outcome));
        }
        let created = res.extra.get("created").and_then(|c| c["id"].as_str());
        Ok(Applied {
            created: created.map(str::to_owned),
        })
    })
}
