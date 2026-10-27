//! The start of a shadow home: what the Node dashboard's home gives it when it has none of its own.
//! The Node home is only read.

use std::fs;
use std::path::Path;

use argus_board::store::damaged_boards;
use argus_core::Paths;
use argus_core::config::pretty;
use argus_procs::RULES_FILE;
use argus_store::snapshot;
use serde_json::{Value, json};

use crate::ServeError;

const CONFIG_FILE: &str = "config.json";
const DB_FILE: &str = "dashboard.db";
pub use argus_core::shadow_mark::{SHADOW_PORT, marked};

fn copy_if_absent(node_home: &Path, shadow_home: &Path, name: &str) -> Result<(), ServeError> {
    let (from, to) = (node_home.join(name), shadow_home.join(name));
    if from.is_file() && !to.exists() {
        fs::create_dir_all(shadow_home)?;
        argus_core::fs::copy_atomic(&from, &to)?;
    }
    Ok(())
}

/// Copies the Node config, with the port of the shadow as its one change. A file that is not a
/// JSON object is copied as it is.
fn seed_config(node_home: &Path, shadow_home: &Path) -> Result<(), ServeError> {
    let (from, to) = (node_home.join(CONFIG_FILE), shadow_home.join(CONFIG_FILE));
    if !from.is_file() || to.exists() {
        return Ok(());
    }
    fs::create_dir_all(shadow_home)?;
    let text = fs::read_to_string(&from)?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(mut config)) => {
            config.insert("port".to_owned(), json!(SHADOW_PORT));
            config.insert(argus_core::shadow_mark::KEY.to_owned(), json!(true));
            argus_core::fs::write_atomic(&to, pretty(&Value::Object(config)))?;
        }
        _ => {
            argus_core::fs::write_atomic(&to, text)?;
        }
    }
    Ok(())
}

fn same_folder(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

/// The boards of `coord_home` that cannot be read at all (all NUL bytes after a crash, empty, or
/// not JSON), one line each with the cause. A shadow serves what it can and says this at start,
/// so a project that is missing from it is not taken for a parity gap.
#[must_use]
pub fn board_findings(coord_home: &Path) -> Vec<String> {
    damaged_boards(coord_home)
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// Gives a shadow home a copy of the Node database (`VACUUM INTO`, read-only on the Node side),
/// and the config (with the shadow's own port and the shadow marker) and the process rules, each only when the shadow home has none. Does nothing
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
    seed_config(node_home, shadow_home)?;
    copy_if_absent(node_home, shadow_home, RULES_FILE)
}

/// Makes the home a shadow for good: seeds it from the Node home when one is given, then writes
/// the shadow marker into its `config.json`, so every later start of a server on it (any verb,
/// the tray, the bridge) runs in shadow mode. The live default home is never marked.
///
/// # Errors
/// A copy or the config write that fails.
pub fn prepare(paths: &Paths, node_home: Option<&Path>) -> Result<(), ServeError> {
    if let Some(node_home) = node_home {
        seed(node_home, &paths.home)?;
    }
    argus_core::shadow_mark::mark(paths)?;
    Ok(())
}
