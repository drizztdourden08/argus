//! Findings that are not a difference between the two servers: a board file that cannot be read
//! at all (all NUL bytes after a crash, empty, or not JSON). Both servers would answer from a
//! hollow board or refuse it, so a run that says only "differs" would hide the cause.

use std::path::PathBuf;

use argus_board::store::damaged_boards;

/// One line per damaged board under the given coord homes, each naming the home and the cause.
#[must_use]
pub fn board_findings(coord_homes: &[PathBuf]) -> Vec<String> {
    coord_homes
        .iter()
        .flat_map(|home| {
            damaged_boards(home)
                .into_iter()
                .map(move |d| format!("{} (in {})", d, home.display()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::board_findings;

    #[test]
    fn a_nul_board_is_a_finding_with_its_cause_and_a_good_board_is_not() {
        let home = tempfile::tempdir().unwrap();
        for (key, bytes) in [("tessera", vec![0u8; 32]), ("argus", b"{}".to_vec())] {
            std::fs::create_dir_all(home.path().join(key)).unwrap();
            std::fs::write(home.path().join(key).join("board.json"), bytes).unwrap();
        }
        let found = board_findings(&[home.path().to_path_buf()]);
        assert_eq!(found.len(), 1);
        assert!(
            found[0].contains("board \"tessera\" is damaged"),
            "{found:?}"
        );
        assert!(found[0].contains("32 NUL bytes"), "{found:?}");
    }

    #[test]
    fn no_homes_and_no_boards_are_no_findings() {
        assert_eq!(board_findings(&[]), Vec::<String>::new());
        let home = tempfile::tempdir().unwrap();
        assert_eq!(
            board_findings(&[home.path().to_path_buf()]),
            Vec::<String>::new()
        );
    }
}
