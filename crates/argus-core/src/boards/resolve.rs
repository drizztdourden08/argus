//! Turning what a caller typed into a board id.

use std::path::Path;

use super::model::{BoardId, Boards};
use super::store::load;
use crate::ids::PORTFOLIO;

fn by_alias(boards: &Boards, s: &str) -> Option<BoardId> {
    boards
        .boards
        .iter()
        .find(|(_, b)| b.aliases.iter().any(|a| a == s))
        .map(|(id, _)| id.clone())
}

fn is_legacy_folder(home: &Path, s: &str) -> bool {
    !s.contains(['/', '\\', ':']) && home.join(s).join("board.json").is_file()
}

/// The id for `s`: an id of the table, else an alias, else a legacy folder `<home>/<s>/board.json`
/// (the folder name is the id). `_portfolio` and unknown names give `None`.
#[must_use]
pub fn resolve(home: &Path, s: &str) -> Option<BoardId> {
    if s == PORTFOLIO || s.is_empty() {
        return None;
    }
    let boards = load(home).unwrap_or_default();
    if boards.boards.contains_key(s) {
        return Some(s.to_owned());
    }
    by_alias(&boards, s).or_else(|| is_legacy_folder(home, s).then(|| s.to_owned()))
}

/// The display name of a board from the table, if it has one.
#[must_use]
pub fn display_name(home: &Path, id: &str) -> Option<String> {
    load(home).ok()?.boards.get(id).map(|b| b.name.clone())
}
