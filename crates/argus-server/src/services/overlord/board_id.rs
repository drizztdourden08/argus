//! The key of the Overlord's board. Wakes, listeners and boards are keyed by the board id, never by
//! an alias, so every place that names the Overlord's board asks here.
// test-tag: critical

use std::path::Path;

use argus_board::roles;
use argus_core::boards::Role;

/// The Overlord's board key when no board holds the role yet: its old key.
const LEGACY: &str = "portfolio";

/// The id of the board that holds the Overlord role; `portfolio` in a home without one.
#[must_use]
pub(crate) fn overlord_board_id(home: &Path) -> String {
    roles::id_of(home, Role::Overlord).unwrap_or_else(|| LEGACY.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_home_falls_back_to_the_legacy_key() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(overlord_board_id(dir.path()), "portfolio");
    }

    #[test]
    fn a_table_with_an_overlord_gives_its_id() {
        let dir = tempfile::tempdir().unwrap();
        let table = r#"{"version":1,"state":"active","boards":{"bja1s18":
            {"name":"Overlord","aliases":["portfolio"],"role":"overlord"}}}"#;
        std::fs::write(dir.path().join("boards.json"), table).unwrap();
        assert_eq!(overlord_board_id(dir.path()), "bja1s18");
    }
}
