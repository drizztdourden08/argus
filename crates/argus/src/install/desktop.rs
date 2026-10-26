//! The Claude Desktop entry of DESIGN 7.4: what it is, where `claude_desktop_config.json` lives,
//! and merging the entry into it without touching anything else.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

use super::env::{Env, Platform};

/// The name of the server in every Claude config.
pub const SERVER_NAME: &str = "coord-dashboard";

const CONFIG_FILE: &str = "claude_desktop_config.json";

/// The entry that runs `argus mcp`. Desktop has one global server with no project: the tools
/// take `project`, or start from `portfolio_overview`.
#[must_use]
pub fn desktop_entry(exe: &Path) -> Value {
    json!({ "mcpServers": { SERVER_NAME: { "command": exe.display().to_string(), "args": ["mcp"] } } })
}

/// Where Claude Desktop keeps its config on this platform.
#[must_use]
pub fn desktop_config_path(env: &Env) -> PathBuf {
    let dir = match env.platform {
        Platform::Windows => env.roaming(),
        Platform::Mac => env.home.join("Library").join("Application Support"),
        Platform::Other => env
            .xdg_config_home
            .clone()
            .unwrap_or_else(|| env.home.join(".config")),
    };
    dir.join("Claude").join(CONFIG_FILE)
}

/// What a merge did.
#[derive(Debug, PartialEq, Eq)]
pub struct Written {
    /// The config file.
    pub file: PathBuf,
    /// The copy of the file as it was, when there was one.
    pub backup: Option<PathBuf>,
}

fn stamp() -> String {
    let iso = now_iso();
    let whole = iso.split('.').next().unwrap_or(&iso);
    whole.chars().filter(|c| *c != '-' && *c != ':').collect()
}

fn read_object(file: &Path) -> Result<Map<String, Value>, String> {
    let text =
        fs::read_to_string(file).map_err(|e| format!("{} cannot be read: {e}", file.display()))?;
    let name = file.display();
    match serde_json::from_str::<Value>(&text) {
        Err(_) => Err(format!(
            "{name} is not valid JSON; fix or remove it, nothing was changed"
        )),
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(format!(
            "{name} does not hold a JSON object; nothing was changed"
        )),
    }
}

fn merged(current: Map<String, Value>, entry: &Value) -> Value {
    let mut out = current;
    let mut servers = match out.remove("mcpServers") {
        Some(Value::Object(map)) => map,
        _ => Map::new(),
    };
    if let Some(ours) = entry["mcpServers"].as_object() {
        servers.extend(ours.clone());
    }
    out.insert("mcpServers".to_owned(), Value::Object(servers));
    Value::Object(out)
}

fn backup_of(file: &Path) -> Result<PathBuf, String> {
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned());
    let copy = file.with_file_name(format!("{}.bak.{}", name.unwrap_or_default(), stamp()));
    argus_core::fs::copy_atomic(file, &copy)
        .map_err(|e| format!("the backup could not be written: {e}"))?;
    Ok(copy)
}

/// Merges `entry` into the Desktop config at `file`: only our server key changes, and the file
/// as it was is kept as `<file>.bak.<stamp>` first. A missing file is created.
///
/// # Errors
/// The file is not a JSON object, or a write failed; nothing is changed then.
pub fn write_desktop_config(file: &Path, entry: &Value) -> Result<Written, String> {
    let existed = file.exists();
    let current = if existed {
        read_object(file)?
    } else {
        Map::new()
    };
    let backup = if existed {
        Some(backup_of(file)?)
    } else {
        None
    };
    let text = serde_json::to_string_pretty(&merged(current, entry)).map_err(|e| e.to_string())?;
    argus_core::fs::write_atomic(file, format!("{text}\n"))
        .map_err(|e| format!("{} cannot be written: {e}", file.display()))?;
    Ok(Written {
        file: file.to_path_buf(),
        backup,
    })
}
