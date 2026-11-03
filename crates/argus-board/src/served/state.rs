//! What a launcher or an item record says about serving and closing.

use serde_json::Value;

use crate::json::{text, truthy};

/// Why an item is closed, in the words a refusal uses: `None` while it is open. The done and
/// dropped lanes and the trash close an item; an archive does not.
#[must_use]
pub fn closure(item: &Value) -> Option<&'static str> {
    if truthy(item, "deletedAt") {
        return Some("in the trash");
    }
    match item["status"].as_str() {
        Some("done") => Some("done"),
        Some("dropped") => Some("dropped"),
        _ => None,
    }
}

/// The item a launcher record serves (its `for`), `None` for a build that has no link.
#[must_use]
pub fn for_of(rec: &Value) -> Option<&str> {
    text(rec, "for").filter(|id| !id.is_empty())
}

/// Whether a launcher record is retired.
pub(super) fn is_retired(rec: &Value) -> bool {
    truthy(rec, "retiredAt")
}
