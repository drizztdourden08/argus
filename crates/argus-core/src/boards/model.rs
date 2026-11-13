//! The shape of `boards.json`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A board id: `b` and six Crockford base32 characters.
pub type BoardId = String;

/// A board with a job of its own in the code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The coordinator board (was `portfolio`).
    Overlord,
    /// The Argus app board.
    Argus,
}

/// One board of the table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardEntry {
    /// The display name, 1 to 30 characters.
    pub name: String,
    /// Frozen old keys; never edited by a rename.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// The role, if any.
    #[serde(default)]
    pub role: Option<Role>,
    /// When the board was added.
    #[serde(default)]
    pub created_at: String,
    /// When it was last renamed.
    #[serde(default)]
    pub renamed_at: Option<String>,
}

/// The whole table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Boards {
    /// The format version, 1.
    pub version: u32,
    /// `active` once migrated.
    pub state: String,
    /// The boards by id.
    pub boards: BTreeMap<BoardId, BoardEntry>,
}

impl Default for Boards {
    fn default() -> Self {
        Self {
            version: 1,
            state: "active".to_owned(),
            boards: BTreeMap::new(),
        }
    }
}
