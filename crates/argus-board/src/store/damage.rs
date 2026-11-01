//! A board file that cannot be a board, told as its own finding. The case that made this: an
//! unclean shutdown left `board.json` at its full size holding only NUL bytes (the data of the
//! write never reached the disk), and the read said only `expected value at line 1 column 1`.

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

use serde_json::Value;

use super::board_path;
use crate::error::BoardError;

/// Why a board file is not a board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardDamage {
    /// The file has no bytes.
    Empty,
    /// Every byte is NUL: the size survived a crash, the content did not.
    AllNul {
        /// The size of the file.
        bytes: usize,
    },
    /// The text is not JSON.
    NotJson(String),
    /// The text is JSON but not an object.
    NotObject,
}

impl fmt::Display for BoardDamage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardDamage::Empty => f.write_str("the file is empty"),
            BoardDamage::AllNul { bytes } => write!(
                f,
                "the file is {bytes} NUL bytes and nothing else (an unclean shutdown kept its size \
                 and lost its content); rebuild it with dashboard/tools/rebuild-board.mjs from the \
                 dashboard database"
            ),
            BoardDamage::NotJson(why) => write!(f, "the file is not valid JSON ({why})"),
            BoardDamage::NotObject => f.write_str("the file is JSON but not an object"),
        }
    }
}

/// What is wrong with the bytes of a board file, `None` when they are a JSON object.
#[must_use]
pub fn check_board_bytes(raw: &[u8]) -> Option<BoardDamage> {
    if raw.is_empty() {
        return Some(BoardDamage::Empty);
    }
    if raw.iter().all(|b| *b == 0) {
        return Some(BoardDamage::AllNul { bytes: raw.len() });
    }
    match serde_json::from_slice::<Value>(raw) {
        Ok(Value::Object(_)) => None,
        Ok(_) => Some(BoardDamage::NotObject),
        Err(e) => Some(BoardDamage::NotJson(e.to_string())),
    }
}

/// The text of a board file, `None` when there is no file.
///
/// # Errors
/// The file cannot be read, or is empty, all NUL bytes or not JSON ([`BoardError::Damaged`]). A
/// JSON value that is not an object is left to the caller's own refusal.
pub fn read_board_text(file: &Path) -> Result<Option<String>, BoardError> {
    let bytes = match fs::read(file) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if let Some(damage) = check_board_bytes(&bytes).filter(|d| *d != BoardDamage::NotObject) {
        return Err(BoardError::Damaged {
            file: file.display().to_string(),
            damage,
        });
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e).into())
}

/// A board of a coord home that is damaged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamagedBoard {
    /// The project key (the folder name).
    pub key: String,
    /// What is wrong.
    pub damage: BoardDamage,
}

impl fmt::Display for DamagedBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "board \"{}\" is damaged: {}", self.key, self.damage)
    }
}

/// Every project folder of `home` whose `board.json` is damaged, by key. A folder with no board
/// file, and a file that cannot be read at all, are not findings of this kind.
#[must_use]
pub fn damaged_boards(home: &Path) -> Vec<DamagedBoard> {
    let Ok(entries) = fs::read_dir(home) else {
        return Vec::new();
    };
    let mut found: Vec<DamagedBoard> = entries
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|key| {
            let raw = fs::read(board_path(home, &key)).ok()?;
            check_board_bytes(&raw).map(|damage| DamagedBoard { key, damage })
        })
        .collect();
    found.sort_by(|a, b| a.key.cmp(&b.key));
    found
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nul_bytes_are_told_apart_from_garbage_and_from_nothing() {
        assert_eq!(check_board_bytes(b""), Some(BoardDamage::Empty));
        assert_eq!(
            check_board_bytes(&[0; 4096]),
            Some(BoardDamage::AllNul { bytes: 4096 })
        );
        assert!(matches!(
            check_board_bytes(b"{\"a\":"),
            Some(BoardDamage::NotJson(_))
        ));
        assert_eq!(check_board_bytes(b"[1]"), Some(BoardDamage::NotObject));
        assert_eq!(check_board_bytes(b"{\"a\":1}\n"), None);
    }

    #[test]
    fn a_home_is_scanned_for_damaged_boards_only() {
        let home = tempfile::tempdir().unwrap();
        let put = |key: &str, bytes: &[u8]| {
            fs::create_dir_all(home.path().join(key)).unwrap();
            fs::write(home.path().join(key).join("board.json"), bytes).unwrap();
        };
        put("good", b"{\"project\":\"good\"}");
        put("hollow", &[0; 64]);
        put("junk", b"not json");
        fs::create_dir_all(home.path().join("no-board")).unwrap();
        fs::write(home.path().join("stray.txt"), "x").unwrap();
        let found = damaged_boards(home.path());
        let keys: Vec<_> = found.iter().map(|d| d.key.as_str()).collect();
        assert_eq!(keys, ["hollow", "junk"]);
        assert_eq!(found[0].damage, BoardDamage::AllNul { bytes: 64 });
    }
}
