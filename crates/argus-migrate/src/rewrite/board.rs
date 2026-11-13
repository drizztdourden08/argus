//! Step 4: what changes inside a board's `board.json`: `project`, the project of the cross-board
//! references on items, and the path prefixes. Actor names, log channels and thread ids that
//! happen to equal a key stay as they are.

use std::path::Path;

use serde_json::Value;

use super::strings;
use super::walk::{children_mut, swap_field};
use crate::error::Result;
use crate::jsonio;
use crate::map::IdMap;

fn swap_refs(refs: &mut Value, map: &IdMap) {
    match refs {
        Value::Array(a) => a.iter_mut().for_each(|r| swap_field(r, "project", map)),
        other => swap_field(other, "project", map),
    }
}

/// Rewrites a board in memory; whether anything changed.
pub fn rewrite(board: &mut Value, map: &IdMap) -> bool {
    let before = board.clone();
    swap_field(board, "project", map);
    if let Some(items) = board.get_mut("items") {
        for item in children_mut(items) {
            for field in ["waitsOn", "requestedBy"] {
                if let Some(refs) = item.get_mut(field) {
                    swap_refs(refs, map);
                }
            }
        }
    }
    strings::rewrite(board, map);
    *board != before
}

/// Rewrites a `board.json` file, temp file then rename, only when it changes.
pub fn rewrite_file(file: &Path, map: &IdMap) -> Result<()> {
    let mut board = jsonio::read(file)?;
    if rewrite(&mut board, map) {
        jsonio::write(file, &board)?;
    }
    Ok(())
}
