//! Turning what a caller typed for a board (an id, an old key, a legacy folder) into its id.

use std::path::Path;

use argus_core::boards::{self, display_name};
use argus_core::ids::PORTFOLIO;

use crate::cli::exit::CoordError;

fn known(home: &Path) -> String {
    let table = boards::load(home).unwrap_or_default();
    let listed: Vec<String> = table
        .boards
        .iter()
        .map(|(id, b)| format!("{} ({id})", b.name))
        .collect();
    if listed.is_empty() {
        "none".to_owned()
    } else {
        listed.join(", ")
    }
}

/// The id of the board `value` names. Without a boards table the value stands as it is (a home
/// that is not migrated keys its boards by folder).
///
/// # Errors
/// Refused when the home has a table and `value` is no id, alias or board folder of it.
pub fn board_ref(home: &Path, value: &str) -> Result<String, CoordError> {
    if value == PORTFOLIO || !home.join("boards.json").is_file() {
        return Ok(value.to_owned());
    }
    boards::resolve(home, value).ok_or_else(|| {
        CoordError::refuse(format!(
            "unknown project \"{value}\" (known: {}). A new board is added with: coord portfolio add {value} --name <Name>.",
            known(home)
        ))
    })
}

/// The name a board is shown by: its table name, else the id as given.
#[must_use]
pub fn shown(home: &Path, id: &str) -> String {
    display_name(home, id).unwrap_or_else(|| id.to_owned())
}
