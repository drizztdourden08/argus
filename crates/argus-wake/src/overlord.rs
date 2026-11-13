//! The one check of who may act as the Overlord: the live coordinator thread of the portfolio board.

use crate::board::BoardView;
use crate::external::RequestError;

/// Is `by` a coordinator thread of the portfolio board (`overlord_board` is its id) that has not ended?
///
/// # Errors
/// 403, with a message that says what only the Overlord may do (`what`), when `by` is anybody else.
pub fn overlord_only(
    by: &str,
    overlord_board: &str,
    portfolio: Option<&BoardView>,
    what: &str,
) -> Result<(), RequestError> {
    let overlord = portfolio
        .and_then(|b| b.threads.get(by))
        .is_some_and(|t| t.role.is("coordinator") && !t.is_ended());
    if overlord {
        Ok(())
    } else {
        Err(RequestError::Forbidden(format!(
            "only the Overlord (the coordinator thread of project {overlord_board}) may {what}"
        )))
    }
}
