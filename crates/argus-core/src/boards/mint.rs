//! Minting a new board id.

use std::path::Path;

use super::error::BoardsError;
use super::model::{BoardEntry, BoardId, Boards};
use super::store::update;
use crate::clock::now_iso;
use crate::ids::{is_key, is_reserved};

const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
const ATTEMPTS: usize = 1000;

fn draw() -> Result<String, BoardsError> {
    let mut bytes = [0u8; 6];
    getrandom::fill(&mut bytes).map_err(|e| std::io::Error::other(e.to_string()))?;
    let tail: String = bytes
        .iter()
        .map(|b| char::from(ALPHABET[usize::from(b & 31)]))
        .collect();
    Ok(format!("b{tail}"))
}

fn has_alias(boards: &Boards, s: &str) -> bool {
    boards
        .boards
        .values()
        .any(|b| b.aliases.iter().any(|a| a == s))
}

fn is_taken(home: &Path, boards: &Boards, s: &str) -> bool {
    is_reserved(s) || boards.boards.contains_key(s) || has_alias(boards, s) || home.join(s).exists()
}

fn fresh(home: &Path, boards: &Boards) -> Result<BoardId, BoardsError> {
    for _ in 0..ATTEMPTS {
        let id = draw()?;
        if id.bytes().any(|b| b.is_ascii_digit()) && !is_taken(home, boards, &id) {
            return Ok(id);
        }
    }
    Err(BoardsError::Exhausted)
}

/// Adds a board named `given_key`: a fresh id, `given_key` as its alias and name.
///
/// # Errors
/// `given_key` is not a project key or is already an id or alias, no free id was found, or the
/// table cannot be written.
pub fn mint(home: &Path, given_key: &str) -> Result<BoardId, BoardsError> {
    update(home, |boards| {
        if !is_key(given_key)
            || boards.boards.contains_key(given_key)
            || has_alias(boards, given_key)
        {
            return Err(BoardsError::BadKey(given_key.to_owned()));
        }
        let id = fresh(home, boards)?;
        boards.boards.insert(
            id.clone(),
            BoardEntry {
                name: given_key.chars().take(30).collect(),
                aliases: vec![given_key.to_owned()],
                role: None,
                created_at: now_iso(),
                renamed_at: None,
            },
        );
        Ok(id)
    })
}
