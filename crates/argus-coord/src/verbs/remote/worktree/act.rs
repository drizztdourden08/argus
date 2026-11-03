//! The frame of a worktree verb that changes the board: the board lock, taken as the calling
//! thread.

use argus_board::Board;

use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::{Scope, locked};
use crate::who::Caller;

/// Runs `act` on the project's board under its lock, as the thread that called.
///
/// # Errors
/// The board is missing or locked, the caller is unknown, or `act` refuses.
pub(super) fn as_caller<T>(
    call: &mut Call<'_>,
    act: impl FnOnce(&mut Board, &Caller, &mut Scope<'_>) -> Result<T, CoordError>,
) -> Result<T, CoordError> {
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        let who = scope.begin(board)?;
        act(board, &who, scope)
    })
}
