//! Board and store failures as API errors.

use argus_board::BoardError;
use argus_core::{ApiError, ErrorCode};

/// The API error for a board failure: a held lock is 503 `board_locked`, a missing board 404,
/// a refusal 400, anything else 500.
#[must_use]
pub fn board_error(e: BoardError) -> ApiError {
    match e {
        BoardError::Locked(_) => ApiError::new(ErrorCode::BoardLocked, e.to_string()),
        BoardError::Missing(_) => ApiError::not_found(e.to_string()),
        BoardError::Refused(m) => ApiError::bad_request(m),
        BoardError::Io(_) | BoardError::Json(_) | BoardError::Damaged { .. } => {
            ApiError::internal()
        }
    }
}
