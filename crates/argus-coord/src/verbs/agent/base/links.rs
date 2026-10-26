//! Which bundles link a launcher, read from the board in its own order (the typed rules sort
//! by name, which the Node listing does not).

use argus_board::Board;
use argus_board::json::section;

/// The tags of the bundles that link the launcher (except `but`), in board order.
#[must_use]
pub fn linked(board: &Board, name: &str, but: Option<&str>) -> Vec<String> {
    section(board, "builds")
        .iter()
        .filter(|(tag, b)| b["launcher"].as_str() == Some(name) && Some(tag.as_str()) != but)
        .map(|(tag, _)| tag.clone())
        .collect()
}
