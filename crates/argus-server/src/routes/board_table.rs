//! What `/api/boards` shows and changes: the identity table's boards, plus the legacy folders a
//! home has before its migration, and the display name of either kind.

use std::path::Path;

use argus_board::BoardError;
use argus_board::json::set;
use argus_board::names::display_name;
use argus_board::roles::{is_argus, is_overlord};
use argus_board::store::{read_board, with_board};
use argus_core::ApiError;
use argus_core::boards::{self, Boards, BoardsError};
use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use super::params::scan_boards;

const NAME_MAX: usize = 30;
const BAD_NAME: &str = "a board name is 1 to 30 characters";

fn legacy_role(home: &Path, id: &str) -> Value {
    if is_overlord(home, id) {
        json!("overlord")
    } else if is_argus(home, id) {
        json!("argus")
    } else {
        Value::Null
    }
}

fn legacy_row(home: &Path, id: &str, registry: &Value) -> Value {
    let board = read_board(home, id).ok().flatten();
    let named = board.as_ref().and_then(|b| b["name"].as_str());
    let registered = registry["projects"][id]["name"].as_str();
    json!({
        "id": id,
        "name": display_name(home, id, named, registered),
        "aliases": [],
        "role": legacy_role(home, id),
    })
}

fn has_entry(table: &Boards, key: &str) -> bool {
    table.boards.contains_key(key)
        || table
            .boards
            .values()
            .any(|b| b.aliases.iter().any(|a| a == key))
}

/// Every board: the table's entries, then the folders that have no entry.
#[must_use]
pub fn list(home: &Path) -> Vec<Value> {
    let table = boards::load(home).unwrap_or_default();
    let mut rows: Vec<Value> = table
        .boards
        .iter()
        .map(|(id, b)| json!({ "id": id, "name": b.name, "aliases": b.aliases, "role": b.role }))
        .collect();
    let registry = read_board(home, PORTFOLIO)
        .ok()
        .flatten()
        .unwrap_or(Value::Null);
    rows.extend(
        scan_boards(home)
            .iter()
            .filter(|key| !has_entry(&table, key))
            .map(|key| legacy_row(home, key, &registry)),
    );
    rows
}

fn checked(name: &str) -> Result<&str, ApiError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > NAME_MAX {
        return Err(ApiError::bad_request(BAD_NAME));
    }
    Ok(name)
}

fn table_error(e: &BoardsError, id: &str) -> ApiError {
    match e {
        BoardsError::BadName => ApiError::bad_request(BAD_NAME),
        BoardsError::Unknown(_) => ApiError::not_found(format!("no board \"{id}\"")),
        BoardsError::Locked => ApiError::unavailable("the boards table is locked, retry"),
        _ => ApiError::internal(),
    }
}

fn legacy_error(e: &BoardError, id: &str) -> ApiError {
    match e {
        BoardError::Missing(_) => ApiError::not_found(format!("no board \"{id}\"")),
        BoardError::Locked(_) => ApiError::unavailable("the board is locked, retry"),
        _ => ApiError::internal(),
    }
}

/// Renames the board `named` (an id, an old key or a legacy folder) and gives `{id, name}`. A
/// board with a table entry changes the table; a legacy folder changes its own `name` field.
///
/// # Errors
/// 400 for a bad name, 404 for an unknown board, 503 when a lock stays held.
pub fn rename(home: &Path, named: &str, name: &str) -> Result<Value, ApiError> {
    let name = checked(name)?;
    let id = boards::resolve(home, named)
        .ok_or_else(|| ApiError::not_found(format!("no board \"{named}\"")))?;
    let in_table = boards::load(home).is_ok_and(|t| t.boards.contains_key(&id));
    if in_table {
        boards::rename(home, &id, name).map_err(|e| table_error(&e, &id))?;
    } else {
        with_board(home, &id, false, |board| {
            set(board, "name", name);
            Ok(())
        })
        .map_err(|e| legacy_error(&e, &id))?;
    }
    Ok(json!({ "id": id, "name": name }))
}
