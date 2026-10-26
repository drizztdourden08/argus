//! The launcher name argument of every launcher verb.

use argus_launchers::launcher::require_launcher_name;

use crate::cli::exit::CoordError;
use crate::verbs::agent::rules::refused;

/// `nameArg`: the launcher the first word names.
///
/// # Errors
/// No word, or a word that is not a launcher name.
pub fn name_arg(positional: &[String], verb: &str) -> Result<String, CoordError> {
    match positional.first().filter(|n| !n.is_empty()) {
        Some(name) => require_launcher_name(name).map_err(refused),
        None => Err(CoordError::refuse(format!(
            "name the launcher: coord launcher {verb} <name>."
        ))),
    }
}
