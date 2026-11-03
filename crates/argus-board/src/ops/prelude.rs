//! What every op handler of the owner's hand-over ops (`build`, the worktree ops and the build
//! ops) uses: the audit line, the result, the caller, the board and its errors.

pub(super) use super::announce::audit;
pub(super) use super::result::OpResult;
pub(super) use super::who::Who;
pub(super) use crate::Board;
pub(super) use crate::error::{BoardError, refuse};
