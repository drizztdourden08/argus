//! The two special boards, found by their role and never by their key: the Overlord's board
//! (once the key `portfolio`) and the Argus app's own board (once `argus`). The identity table
//! (`argus_core::boards`) is the truth when `boards.json` exists. A home without the table (test
//! homes, a home before the migration) names its boards by folder, so the old keys stand for the
//! roles there and nowhere else.

use std::path::Path;

use argus_core::boards::{self, BoardId, Role};

const TABLE: &str = "boards.json";
const LEGACY_OVERLORD: &str = "portfolio";
const LEGACY_ARGUS: &str = "argus";

fn legacy_key(role: Role) -> &'static str {
    if role == Role::Overlord {
        LEGACY_OVERLORD
    } else {
        LEGACY_ARGUS
    }
}

/// Does the home carry an identity table?
#[must_use]
pub fn table_present(home: &Path) -> bool {
    home.join(TABLE).is_file()
}

/// The id a key, alias or id names in the table; `None` without a table or when unknown.
#[must_use]
pub fn table_id(home: &Path, s: &str) -> Option<BoardId> {
    table_present(home)
        .then(|| boards::resolve(home, s))
        .flatten()
}

/// The id of the board that holds `role`: from the table, else (no table) the legacy folder.
#[must_use]
pub fn id_of(home: &Path, role: Role) -> Option<String> {
    if table_present(home) {
        return boards::by_role(home, role);
    }
    let key = legacy_key(role);
    home.join(key)
        .join("board.json")
        .is_file()
        .then(|| key.to_owned())
}

/// Does the board `id` (an id or an alias) hold `role`?
#[must_use]
pub fn is_role(home: &Path, id: &str, role: Role) -> bool {
    if table_present(home) {
        return table_id(home, id).is_some_and(|b| boards::role(home, &b) == Some(role));
    }
    id == legacy_key(role)
}

/// Is this the Overlord's board?
#[must_use]
pub fn is_overlord(home: &Path, id: &str) -> bool {
    is_role(home, id, Role::Overlord)
}

/// Is this the Argus app's own board?
#[must_use]
pub fn is_argus(home: &Path, id: &str) -> bool {
    is_role(home, id, Role::Argus)
}

/// The key of the Overlord's board: the id of the board that holds the role, else (no such board
/// yet) its old key `portfolio`. Wakes, listeners and boards are keyed by this, never by an alias.
#[must_use]
pub fn overlord_key(home: &Path) -> String {
    id_of(home, Role::Overlord).unwrap_or_else(|| LEGACY_OVERLORD.to_owned())
}

#[cfg(test)]
mod overlord_key_tests {
    use super::*;

    #[test]
    fn an_empty_home_gives_the_legacy_key() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(overlord_key(dir.path()), "portfolio");
    }

    #[test]
    fn a_table_with_an_overlord_gives_its_id() {
        let dir = tempfile::tempdir().unwrap();
        let table = r#"{"version":1,"state":"active","boards":{"bja1s18":
            {"name":"Overlord","aliases":["portfolio"],"role":"overlord"}}}"#;
        std::fs::write(dir.path().join("boards.json"), table).unwrap();
        assert_eq!(overlord_key(dir.path()), "bja1s18");
    }
}
