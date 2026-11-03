//! What an op did to the builds of the items it changed: `closedWith` lists the builds that
//! closed with their item, `reopenedWith` the ones that came back, so the CLI can say so.

use std::collections::BTreeSet;

use serde_json::json;

use super::result::{OpResult, Status};
use crate::Board;
use crate::served::closed_by_items;

/// The builds closed with an item before an op runs.
pub(super) fn before(board: &Board) -> BTreeSet<String> {
    closed_by_items(board)
}

/// `result` with the builds the op closed (`closedWith`) and brought back (`reopenedWith`),
/// compared with the set taken before it ran. A result that changed nothing is left alone.
pub(super) fn tell(board: &Board, was: &BTreeSet<String>, mut result: OpResult) -> OpResult {
    if result.status != Status::Applied {
        return result;
    }
    let now = closed_by_items(board);
    let closed: Vec<&String> = now.difference(was).collect();
    let back: Vec<&String> = was.difference(&now).collect();
    if !closed.is_empty() {
        result.extra.insert("closedWith".to_owned(), json!(closed));
    }
    if !back.is_empty() {
        result.extra.insert("reopenedWith".to_owned(), json!(back));
    }
    result
}
