//! The parent a type requires: a story sits under an epic and a task under an epic or a story, so
//! neither is created without one and neither loses it by an edit. Items that already have none
//! stay readable and editable in their other fields.

use serde_json::Value;

use super::kind::DEFAULT_TYPE;
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{doc, find_ids, truthy};
use crate::registry::{ItemType, registry};

/// The most open epics a refusal names.
const NAMED: usize = 5;

/// The open epics as `I-n "title"`, at most [`NAMED`], for a refusal to point at.
fn open_epics(board: &Board) -> String {
    let ids = find_ids(board, "items", |i| {
        !truthy(i, "deletedAt")
            && i["type"].as_str() == Some("epic")
            && !matches!(i["status"].as_str(), Some("done" | "dropped"))
    });
    let named: Vec<String> = ids
        .iter()
        .take(NAMED)
        .map(|id| {
            let title = doc(board, "items", id).map_or("", |i| i["title"].as_str().unwrap_or(""));
            format!("{id} \"{title}\"")
        })
        .collect();
    if named.is_empty() {
        "There is no open epic: create one first.".to_owned()
    } else {
        format!("Open epics: {}.", named.join(", "))
    }
}

fn refusal(board: &Board, ty: &ItemType, what: &str) -> BoardError {
    refuse(format!(
        "A {} needs a parent ({}): {what}. Pass one with --parent I-n. {}",
        ty.id,
        ty.parents.join(" or "),
        open_epics(board)
    ))
}

/// A new item of this type names a parent.
pub(crate) fn on_create(board: &Board, ty: &ItemType, parent: &Value) -> Result<(), BoardError> {
    if ty.parent_required && parent.is_null() {
        return Err(refusal(board, ty, "none was given"));
    }
    Ok(())
}

/// A plain item reads as the default type, so it needs the parent that type needs.
///
/// # Errors
/// Refused when the default type requires a parent, since a plain item names none.
pub fn on_plain(board: &Board) -> Result<(), BoardError> {
    match registry().item_type(DEFAULT_TYPE) {
        Some(ty) => on_create(board, ty, &Value::Null),
        None => Ok(()),
    }
}

/// An edit that sets the parent to none leaves a type that requires one without it.
pub(crate) fn on_clear(board: &Board, ty: &ItemType, parent: &Value) -> Result<(), BoardError> {
    if ty.parent_required && parent.is_null() {
        return Err(refusal(board, ty, "the edit would clear it"));
    }
    Ok(())
}

/// An item changing to this type keeps a parent, or is refused when the type requires one.
pub(crate) fn on_retype(board: &Board, ty: &ItemType, item: &Value) -> Result<(), BoardError> {
    if ty.parent_required && item["parent"].is_null() {
        return Err(refusal(board, ty, "the item has none"));
    }
    Ok(())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn board() -> Board {
        json!({"items": {
            "I-1": {"type": "epic", "title": "Boards", "status": "todo"},
            "I-2": {"type": "epic", "title": "Gone", "status": "done"}
        }})
    }

    #[test]
    fn a_task_without_a_parent_is_refused_naming_the_open_epics() {
        let ty = registry().item_type("task").unwrap();
        let text = on_create(&board(), ty, &Value::Null)
            .unwrap_err()
            .to_string();
        assert!(text.contains("--parent I-n"), "{text}");
        assert!(text.contains("I-1 \"Boards\""), "{text}");
        assert!(!text.contains("I-2"), "{text}");
        assert!(on_create(&board(), ty, &json!("I-1")).is_ok());
    }

    #[test]
    fn clearing_the_parent_of_a_story_or_a_bug_is_refused_but_an_epic_is_a_root() {
        let reg = registry();
        for ty in ["story", "bug"] {
            let t = reg.item_type(ty).unwrap();
            assert!(on_clear(&board(), t, &Value::Null).is_err(), "{ty}");
        }
        let epic = reg.item_type("epic").unwrap();
        assert!(on_clear(&board(), epic, &Value::Null).is_ok());
    }
}
