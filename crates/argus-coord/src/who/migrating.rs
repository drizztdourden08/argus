//! The board migration marker: while `<coord home>/MIGRATING` exists, no board is written and
//! no dashboard is started.

use std::path::Path;

use crate::cli::exit::CoordError;

/// What a refused write says.
pub const MESSAGE: &str = "board migration in progress, try again in a minute";

/// Whether a board migration is running in this home.
#[must_use]
pub fn migrating(home: &Path) -> bool {
    home.join("MIGRATING").exists()
}

/// Refuses while a board migration is running.
///
/// # Errors
/// Refused with [`MESSAGE`] when the marker exists.
pub fn refuse_while_migrating(home: &Path) -> Result<(), CoordError> {
    if migrating(home) {
        return Err(CoordError::refuse(MESSAGE));
    }
    Ok(())
}
