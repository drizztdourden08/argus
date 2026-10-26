//! The start of a shadow home: what the Node dashboard's home gives it when it has none of its own.
//! The Node home is only read.

use std::fs;
use std::path::Path;

use argus_procs::RULES_FILE;
use argus_store::snapshot;

use crate::ServeError;

const CONFIG_FILE: &str = "config.json";
const DB_FILE: &str = "dashboard.db";

fn copy_if_absent(node_home: &Path, shadow_home: &Path, name: &str) -> Result<(), ServeError> {
    let (from, to) = (node_home.join(name), shadow_home.join(name));
    if from.is_file() && !to.exists() {
        fs::create_dir_all(shadow_home)?;
        fs::copy(from, to)?;
    }
    Ok(())
}

fn same_folder(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// Gives a shadow home a copy of the Node database (`VACUUM INTO`, read-only on the Node side),
/// and the config and the process rules, each only when the shadow home has none. Does nothing
/// when both homes are one folder.
///
/// # Errors
/// A copy that fails; a Node home with no database is not an error (the shadow starts empty).
pub fn seed(node_home: &Path, shadow_home: &Path) -> Result<(), ServeError> {
    if same_folder(node_home, shadow_home) {
        return Ok(());
    }
    let (from, to) = (node_home.join(DB_FILE), shadow_home.join(DB_FILE));
    if from.is_file() && !to.exists() {
        snapshot(&from, &to)?;
    }
    copy_if_absent(node_home, shadow_home, CONFIG_FILE)?;
    copy_if_absent(node_home, shadow_home, RULES_FILE)
}
