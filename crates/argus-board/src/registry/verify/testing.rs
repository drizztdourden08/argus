//! Helpers the tests of the verify modules share.

use serde_json::Value;

use crate::registry::load::{SOURCE, parse};

/// The error text of the shipped file after `edit` changed it.
pub(super) fn refused(edit: impl FnOnce(&mut Value)) -> String {
    let mut doc: Value = serde_json::from_str(SOURCE).unwrap();
    edit(&mut doc);
    parse(&doc.to_string()).unwrap_err().to_string()
}

/// The item type `id` of the document.
pub(super) fn item_type<'a>(doc: &'a mut Value, id: &str) -> &'a mut Value {
    let all = doc["types"].as_array_mut().unwrap();
    all.iter_mut().find(|t| t["id"] == id).unwrap()
}
