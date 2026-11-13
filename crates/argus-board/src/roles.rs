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
