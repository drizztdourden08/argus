//! Changing the type of an item: its parent and its children must stay valid for the new type.

use serde_json::Value;

use super::tree::{allowed, children_of, type_in};
use crate::Board;
use crate::error::{BoardError, refuse};
use crate::registry::{ItemType, registry};

/// The parent of the item still holds the new type.
pub(crate) fn parent_fits(board: &Board, item: &Value, to: &ItemType) -> Result<(), BoardError> {
    match item["parent"].as_str() {
        Some(parent) => allowed(&to.id, &type_in(board, parent), parent),
        None => Ok(()),
    }
}

/// Every child of the item may still sit under the new type.
pub(crate) fn children_fit(board: &Board, id: &str, to: &ItemType) -> Result<(), BoardError> {
    for child in children_of(board, id) {
        let kind = type_in(board, &child);
        if !registry().allows_parent(&kind, &to.id) {
            return Err(refuse(format!(
                "{child} is a {kind} and cannot sit under a {}; move it first.",
                to.id
            )));
        }
    }
    Ok(())
}
