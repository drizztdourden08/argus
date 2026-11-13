//! The names the owner sees for the main coordinator and its project. Only what is shown
//! differs from the ids: the thread id stays `coordinator`. The Overlord's board is found by its
//! role (`crate::roles`), never by a key, and the identity table holds the display name.

use std::path::Path;

use argus_core::boards;
use argus_core::js;

use crate::roles::{is_overlord, table_id};

/// The dashboard app and its overview page.
pub const ARGUS: &str = "Argus";
/// The main coordinator and its project, as shown.
pub const OVERLORD: &str = "Overlord";

fn is_legacy(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.strip_prefix("portfolio").is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'))
    })
}

fn usable<'a>(key: &str, name: Option<&'a str>) -> Option<&'a str> {
    let name = js::trim(name?);
    (!name.is_empty() && name != key && !is_legacy(name)).then_some(name)
}

fn table_name(home: &Path, id: &str) -> Option<String> {
    let board = table_id(home, id)?;
    boards::display_name(home, &board)
}

fn overlord_name(key: &str, board_name: Option<&str>, registry_name: Option<&str>) -> String {
    usable(key, registry_name)
        .or_else(|| usable(key, board_name))
        .unwrap_or(OVERLORD)
        .to_owned()
}

/// The display name of a board: the identity table's name, else (no table) its board name, else
/// its registry name, else the id. The Overlord's board shows the registry or board name when it
/// is a real choice, else `Overlord`.
#[must_use]
pub fn display_name(
    home: &Path,
    id: &str,
    board_name: Option<&str>,
    registry_name: Option<&str>,
) -> String {
    if let Some(name) = table_name(home, id) {
        return name;
    }
    if is_overlord(home, id) {
        return overlord_name(id, board_name, registry_name);
    }
    board_name.or(registry_name).unwrap_or(id).to_owned()
}

/// What a session title starts with: `Overlord` for the Overlord's board, else the table name,
/// else the board name.
#[must_use]
pub fn session_name(home: &Path, id: &str, board_name: Option<&str>) -> String {
    if is_overlord(home, id) {
        return OVERLORD.to_owned();
    }
    table_name(home, id).unwrap_or_else(|| board_name.unwrap_or(id).to_owned())
}
