//! The start of a preview home: a copied `COORD_HOME` with board writes and no outward
//! effect. A preview refuses the live boards, then marks its home so every later start of a
//! server on it (any verb, the tray, the bridge) is a preview too.

use std::path::Path;

use argus_core::Paths;

use crate::ServeError;

pub use argus_core::preview_mark::{PREVIEW_PORT, marked};

/// Refuses the live boards, then writes the preview marker into the home's `config.json`.
/// `live` is the live boards folder; `None` is the default `~/.coord`.
///
/// # Errors
/// [`ServeError::PreviewRefused`] for the live boards or a home inside them; a config write
/// that fails.
pub fn prepare(paths: &Paths, live: Option<&Path>) -> Result<(), ServeError> {
    let default = Paths::live_coord_home();
    let live = live.unwrap_or(&default);
    if let Some(why) = argus_core::preview_mark::refusal(paths, live) {
        return Err(ServeError::PreviewRefused(why));
    }
    argus_core::preview_mark::mark(paths)?;
    Ok(())
}
