//! The type of an item: the one it names, or `task` when it names none.

use serde_json::Value;

use crate::error::{BoardError, refuse};
use crate::registry::{Collection, ItemType, registry};

/// The type an item without one reads as.
pub(crate) const DEFAULT_TYPE: &str = "task";

/// Whether the item names a type or a stage of its own. Only such an item follows the rules of
/// its type; one that names neither behaves as it always did.
pub(crate) fn is_typed(item: &Value) -> bool {
    !item["type"].is_null() || !item["stage"].is_null()
}

/// The type id of an item: its own, else `task`.
pub(crate) fn type_id(item: &Value) -> &str {
    item["type"].as_str().unwrap_or(DEFAULT_TYPE)
}

/// The item types, which are the ones that live in `items`.
fn item_types() -> impl Iterator<Item = &'static ItemType> {
    registry()
        .types
        .iter()
        .filter(|t| t.collection == Collection::Items)
}

/// The item type with this id, refused with the list of the types when there is none.
pub(crate) fn named(id: &str) -> Result<&'static ItemType, BoardError> {
    item_types().find(|t| t.id == id).ok_or_else(|| {
        let all: Vec<&str> = item_types().map(|t| t.id.as_str()).collect();
        refuse(format!("type must be one of {}.", all.join(", ")))
    })
}

/// The type of an item.
pub(crate) fn type_of(item: &Value) -> Result<&'static ItemType, BoardError> {
    named(type_id(item))
}

/// The type a `type` argument names.
pub(crate) fn check_type(v: &Value) -> Result<&'static ItemType, BoardError> {
    named(v.as_str().unwrap_or_default())
}
