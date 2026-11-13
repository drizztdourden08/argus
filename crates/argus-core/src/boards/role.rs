//! Boards by role.

use std::path::Path;

use super::model::{BoardId, Role};
use super::store::load;

/// The role of a board.
#[must_use]
pub fn role(home: &Path, id: &str) -> Option<Role> {
    load(home).ok()?.boards.get(id)?.role
}

/// The board that holds a role.
#[must_use]
pub fn by_role(home: &Path, role: Role) -> Option<BoardId> {
    load(home)
        .ok()?
        .boards
        .into_iter()
        .find(|(_, b)| b.role == Some(role))
        .map(|(id, _)| id)
}
