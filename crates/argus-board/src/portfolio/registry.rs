//! The portfolio board's shape and the list of projects it covers.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::Board;
use crate::json::{default_field, display, obj, set, truthy};

/// Folders under the coord home that are never project boards.
const NOT_PROJECTS: [&str; 2] = [argus_core::ids::PORTFOLIO, "dashboard"];

/// Give a portfolio board its shape: `kind`, the project registry and the card hashes.
pub fn shape(p: &mut Board) {
    set(p, "kind", "portfolio");
    default_field(p, "projects", serde_json::json!({}));
    default_field(p, "pushedHash", serde_json::json!({}));
}

fn boards_on_disk(home: &Path) -> Vec<String> {
    let Ok(dir) = fs::read_dir(home) else {
        return Vec::new();
    };
    dir.filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|k| !NOT_PROJECTS.contains(&k.as_str()) && home.join(k).join("board.json").exists())
        .collect()
}

/// The projects of the portfolio: the registered ones and every folder with a board, except the
/// retired ones, sorted.
#[must_use]
pub fn project_keys(home: &Path, p: &Board) -> Vec<String> {
    let projects = obj(&p["projects"]);
    let retired: BTreeSet<String> = projects
        .values()
        .filter(|x| truthy(x, "retired"))
        .map(|x| display(&x["key"]))
        .collect();
    let all: BTreeSet<String> = projects
        .keys()
        .cloned()
        .chain(boards_on_disk(home))
        .collect();
    all.into_iter().filter(|k| !retired.contains(k)).collect()
}
