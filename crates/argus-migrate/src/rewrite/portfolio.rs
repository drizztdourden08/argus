//! Step 5: what changes inside `_portfolio/board.json`: the registry, the cards and the pushed
//! hashes are re-keyed, and every field that names a board follows.

use std::path::Path;

use serde_json::Value;

use super::strings;
use super::walk::{children_mut, rekey_field, swap_field, swap_str};
use crate::error::Result;
use crate::jsonio;
use crate::map::IdMap;

fn swap_dep_ends(deps: &mut Value, map: &IdMap) {
    for dep in children_mut(deps) {
        for end in ["from", "to"] {
            if let Some(e) = dep.get_mut(end) {
                swap_field(e, "project", map);
            }
        }
    }
}

fn swap_entries(entries: &mut Value, map: &IdMap) {
    for entry in children_mut(entries) {
        swap_field(entry, "project", map);
        if let Some(list) = entry.get_mut("deliveredTo") {
            children_mut(list).into_iter().for_each(|s| swap_str(s, map));
        }
    }
}

/// Rewrites the portfolio in memory; whether anything changed.
pub fn rewrite(portfolio: &mut Value, map: &IdMap) -> bool {
    let before = portfolio.clone();
    rekey_field(portfolio, "projects", map, true);
    rekey_field(portfolio, "cards", map, true);
    rekey_field(portfolio, "pushedHash", map, false);
    if let Some(boards) = portfolio.get_mut("boards") {
        children_mut(boards)
            .into_iter()
            .for_each(|b| swap_field(b, "key", map));
    }
    if let Some(deps) = portfolio.get_mut("deps") {
        swap_dep_ends(deps, map);
    }
    if let Some(entries) = portfolio.get_mut("entries") {
        swap_entries(entries, map);
    }
    strings::rewrite(portfolio, map);
    *portfolio != before
}

/// Rewrites `<home>/_portfolio/board.json` when it exists and changes.
pub fn rewrite_file(file: &Path, map: &IdMap) -> Result<()> {
    if !file.is_file() {
        return Ok(());
    }
    let mut portfolio = jsonio::read(file)?;
    if rewrite(&mut portfolio, map) {
        jsonio::write(file, &portfolio)?;
    }
    Ok(())
}
