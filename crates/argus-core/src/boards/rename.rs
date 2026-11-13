//! Changing the display name of a board.

use std::path::Path;

use super::error::BoardsError;
use super::store::update;
use crate::clock::now_iso;

const NAME_MAX: usize = 30;

/// Sets the name (trimmed, 1 to 30 characters) and `renamedAt`. Aliases stay as they are.
///
/// # Errors
/// The name is out of bounds, the id is unknown, or the table cannot be written.
pub fn rename(home: &Path, id: &str, name: &str) -> Result<(), BoardsError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > NAME_MAX {
        return Err(BoardsError::BadName);
    }
    update(home, |boards| {
        let entry = boards
            .boards
            .get_mut(id)
            .ok_or_else(|| BoardsError::Unknown(id.to_owned()))?;
        name.clone_into(&mut entry.name);
        entry.renamed_at = Some(now_iso());
        Ok(())
    })
}
