//! The letters ids are shown with. An item is stored as `I-12` whatever its type; the owner and the
//! threads can write it with the letters of its kind (`B-12` for a bug, `TP-3` for a test plan)
//! and every verb accepts both. [`resolve`] turns a written id into the stored one and refuses
//! the wrong letters with the right ones; [`shown`] goes the other way. The letters come from the
//! registry, never from this file.

use argus_core::ids::PORTFOLIO;
use serde_json::Value;

use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::doc;
use crate::registry::{Kind, registry};

/// The letter an item is stored with.
const STORED: &str = "I";

/// The type an item without one reads as (as the item ops read it).
const UNTYPED: &str = "task";

/// The letter of the portfolio sends (`S-3`), which sit on the portfolio board where no story
/// does: there it is never the code of a story.
const SENDS: &str = "S";

/// `LETTERS-digits`: the letters (one or two capitals) and the number of an id.
fn split(word: &str) -> Option<(&str, &str)> {
    let (letters, number) = word.split_once('-')?;
    let capitals =
        (1..=2).contains(&letters.len()) && letters.chars().all(|c| c.is_ascii_uppercase());
    let digits = !number.is_empty() && number.chars().all(|c| c.is_ascii_digit());
    (capitals && digits).then_some((letters, number))
}

/// Whether a board is the portfolio board, as the store reads it.
fn is_portfolio(board: &Board) -> bool {
    board["kind"] == "portfolio" || board["project"] == PORTFOLIO
}

/// The work item type whose ids are written with `letters` on this board.
fn item_kind(board: &Board, letters: &str) -> Option<Kind<'static>> {
    let on_sends = is_portfolio(board) && letters == SENDS;
    registry()
        .kind_by_code(letters)
        .filter(|k| k.is_work_item() && !on_sends)
}

/// The kind whose records are stored under other letters than it is shown with, when the board
/// keeps them (a dependency is shown `DP-3` and kept `D-3` on the portfolio board).
fn stored_letters(board: &Board, letters: &str) -> Option<&'static str> {
    registry()
        .kind_by_code(letters)
        .and_then(|k| k.stored)
        .filter(|_| is_portfolio(board))
}

/// The label of the type an item has, in lower case, and its letters.
fn type_of(item: &Value) -> (String, &'static str) {
    let id = item["type"].as_str().unwrap_or(UNTYPED);
    registry().kind(id).map_or_else(
        || (id.to_owned(), STORED),
        |k| (k.label.to_lowercase(), k.code),
    )
}

/// The stored id of an item written with the letters of its kind, refused when the letters are
/// another kind's.
fn item_id(board: &Board, kind: &Kind<'_>, number: &str, word: &str) -> Result<String, BoardError> {
    let stored = format!("{STORED}-{number}");
    let Some(item) = doc(board, "items", &stored).filter(|i| !i.is_null()) else {
        return Ok(stored);
    };
    let (label, letters) = type_of(item);
    if letters == kind.code {
        return Ok(stored);
    }
    Err(refuse(format!(
        "{word} is not a {}: {stored} is a {label}, {letters}-{number}.",
        kind.label.to_lowercase()
    )))
}

/// The id as the board stores it. `B-12` is `I-12` when item 12 is a bug; `I-12` and every id
/// that is not an item's (`T-4`, `Q-3`, `brock/I-4`, plain words) come back as they are. On the
/// portfolio board `DP-3` is the stored `D-3`.
///
/// # Errors
/// Refused when the letters are those of another kind than the item has, with the right ones in
/// the message (`S-12 is not a story: I-12 is a bug, B-12.`).
pub fn resolve(board: &Board, word: &str) -> Result<String, BoardError> {
    let Some((letters, number)) = split(word) else {
        return Ok(word.to_owned());
    };
    if let Some(stored) = stored_letters(board, letters) {
        return Ok(format!("{stored}-{number}"));
    }
    match item_kind(board, letters) {
        Some(kind) => item_id(board, &kind, number, word),
        None => Ok(word.to_owned()),
    }
}

/// The id of an item as it is shown: the letters of its type and its number (`B-12`). An id that
/// is not `I-n` comes back as it is.
#[must_use]
pub fn shown(item: &Value) -> String {
    let id = item["id"].as_str().unwrap_or_default();
    match split(id) {
        Some((STORED, number)) => format!("{}-{number}", type_of(item).1),
        _ => id.to_owned(),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{resolve, shown};

    fn board() -> serde_json::Value {
        json!({
            "items": {
                "I-1": {"id": "I-1", "type": "epic"},
                "I-2": {"id": "I-2", "type": "story"},
                "I-3": {"id": "I-3", "type": "task"},
                "I-4": {"id": "I-4", "type": "bug"},
                "I-5": {"id": "I-5", "type": "debt"},
                "I-6": {"id": "I-6", "type": "test-plan"},
                "I-7": {"id": "I-7", "title": "an old item with no type"},
            }
        })
    }

    fn portfolio() -> serde_json::Value {
        let mut board = board();
        board["kind"] = json!("portfolio");
        board
    }

    fn ok(board: &serde_json::Value, word: &str) -> String {
        resolve(board, word).unwrap()
    }

    #[test]
    fn the_letters_of_a_type_name_its_item() {
        let b = board();
        let want = [
            ("E-1", "I-1"),
            ("S-2", "I-2"),
            ("K-3", "I-3"),
            ("B-4", "I-4"),
            ("X-5", "I-5"),
            ("TP-6", "I-6"),
        ];
        for (written, stored) in want {
            assert_eq!(ok(&b, written), stored, "{written}");
        }
    }

    #[test]
    fn every_stored_id_still_works() {
        let b = board();
        for n in 1..=7 {
            let id = format!("I-{n}");
            assert_eq!(ok(&b, &id), id);
        }
        assert_eq!(ok(&b, "I-99"), "I-99");
    }

    #[test]
    fn an_item_with_no_type_is_a_task() {
        let b = board();
        assert_eq!(ok(&b, "K-7"), "I-7");
        let text = resolve(&b, "B-7").unwrap_err().to_string();
        assert_eq!(text, "B-7 is not a bug: I-7 is a task, K-7.");
    }

    #[test]
    fn the_wrong_letters_are_refused_with_the_right_ones() {
        let text = resolve(&board(), "S-4").unwrap_err().to_string();
        assert_eq!(text, "S-4 is not a story: I-4 is a bug, B-4.");
        assert_eq!(ok(&board(), "T-4"), "T-4");
        let text = resolve(&board(), "X-6").unwrap_err().to_string();
        assert_eq!(text, "X-6 is not a tech debt: I-6 is a test plan, TP-6.");
    }

    #[test]
    fn the_longest_letters_are_read_whole() {
        let b = board();
        assert_eq!(ok(&b, "TP-6"), "I-6");
        assert_eq!(ok(&b, "T-6"), "T-6");
    }

    #[test]
    fn an_id_that_is_not_an_items_comes_back_as_it_was() {
        let b = board();
        for word in [
            "T-4",
            "Q-3",
            "D-1",
            "H-2",
            "N-9",
            "A-373",
            "PR-2",
            "WT-1",
            "brock/I-4",
            "hello",
            "b-4",
            "B4",
            "B-",
            "-4",
            "B-x",
            "BLX-4",
            "",
        ] {
            assert_eq!(ok(&b, word), word, "{word}");
        }
    }

    #[test]
    fn a_dependency_is_dp_on_the_portfolio_board_and_stored_d() {
        assert_eq!(ok(&portfolio(), "DP-3"), "D-3");
        assert_eq!(ok(&board(), "DP-3"), "DP-3");
        assert_eq!(ok(&portfolio(), "D-3"), "D-3");
    }

    #[test]
    fn s_is_a_send_on_the_portfolio_board_and_a_story_elsewhere() {
        assert_eq!(ok(&portfolio(), "S-4"), "S-4");
        assert_eq!(ok(&portfolio(), "E-1"), "I-1");
        assert_eq!(ok(&board(), "S-2"), "I-2");
    }

    #[test]
    fn an_item_is_shown_with_the_letters_of_its_type() {
        let b = board();
        assert_eq!(shown(&b["items"]["I-4"]), "B-4");
        assert_eq!(shown(&b["items"]["I-6"]), "TP-6");
        assert_eq!(shown(&b["items"]["I-7"]), "K-7");
        assert_eq!(shown(&json!({"id": "T-4"})), "T-4");
        assert_eq!(shown(&json!({})), "");
    }
}
