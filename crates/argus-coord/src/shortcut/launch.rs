//! The `.claude/launch.json` of a checkout: read it, merge configurations into it by name and
//! write it back. The Browser pane of Claude Desktop lists a configuration that has a `url` and
//! no command on a new tab, so Play opens the board and nothing is started.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::alias::shortcut_configs;
use crate::cli::exit::CoordError;

/// The `launch.json` of a checkout.
#[must_use]
pub fn launch_file(root: &Path) -> PathBuf {
    root.join(".claude").join("launch.json")
}

fn untouched(file: &Path, why: &str) -> CoordError {
    CoordError::refuse(format!(
        "{} {why}; fix it by hand, it was not touched.",
        file.display()
    ))
}

/// The JSON of an existing `launch.json`; `None` when there is none.
///
/// # Errors
/// Refused when it is not valid JSON, not an object, or has a `configurations` that is not a
/// list. The file is never written then.
pub fn read_launch(file: &Path) -> Result<Option<Value>, CoordError> {
    if !file.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(file)?;
    let json: Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| untouched(file, &format!("is not valid JSON ({e})")))?;
    if !json.is_object() {
        return Err(untouched(file, "is not a JSON object"));
    }
    if json.get("configurations").is_some_and(|c| !c.is_array()) {
        return Err(untouched(file, "has \"configurations\" that is not a list"));
    }
    Ok(Some(json))
}

/// What a write did.
#[derive(Debug, Clone, PartialEq)]
pub struct Written {
    /// The file.
    pub file: PathBuf,
    /// Whether anything was written.
    pub changed: bool,
    /// The entries that were new or different.
    pub wrote: Vec<Value>,
}

fn is_named(config: &Value, name: &Value) -> bool {
    config.is_object() && config["name"] == *name
}

/// Merges `entries` by name into the `launch.json` of `root`, in one write: every other
/// configuration and top-level key stays as it is, an entry of the same name replaces itself in
/// place, a new one goes last. Never overwrites a file it cannot read.
///
/// # Errors
/// As [`read_launch`]; the file or its folder cannot be written.
pub fn write_shortcuts(root: &Path, entries: &[Value]) -> Result<Written, CoordError> {
    let file = launch_file(root);
    let mut json =
        read_launch(&file)?.unwrap_or_else(|| json!({ "version": "0.0.1", "configurations": [] }));
    let mut list = json["configurations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut wrote = Vec::new();
    for entry in entries {
        match list.iter().position(|c| is_named(c, &entry["name"])) {
            Some(at) if list[at] == *entry => continue,
            Some(at) => list[at] = entry.clone(),
            None => list.push(entry.clone()),
        }
        wrote.push(entry.clone());
    }
    if wrote.is_empty() {
        return Ok(Written {
            file,
            changed: false,
            wrote,
        });
    }
    json["configurations"] = Value::Array(list);
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(
        &file,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&json).unwrap_or_default()
        ),
    )?;
    Ok(Written {
        file,
        changed: true,
        wrote,
    })
}

/// `url` without its port: `/:0$/` for a wanted alias, `/:\d+\/?$/` for a listed one.
fn without_port(url: &str) -> &str {
    let Some((head, tail)) = url.rsplit_once(':') else {
        return url;
    };
    let digits = tail.strip_suffix('/').unwrap_or(tail);
    if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
        head
    } else {
        url
    }
}

fn needs(root: &Path, project: &str, thread: &str) -> Result<bool, CoordError> {
    let wants: Vec<String> = shortcut_configs(project, project, thread, 0)?
        .iter()
        .map(|c| {
            let url = c["url"].as_str().unwrap_or_default();
            url.strip_suffix(":0").unwrap_or(url).to_owned()
        })
        .collect();
    let json = read_launch(&launch_file(root))?;
    let have: Vec<String> = json
        .as_ref()
        .and_then(|j| j["configurations"].as_array())
        .map(|list| {
            list.iter()
                .filter_map(|c| c["url"].as_str())
                .map(|u| without_port(u).to_owned())
                .collect()
        })
        .unwrap_or_default();
    Ok(wants.iter().any(|want| !have.contains(want)))
}

/// Should the thread be told to run `coord board shortcut`? Yes when the `launch.json` of its
/// checkout lacks a configuration whose url is one of the thread's aliases (any port; the
/// Overlord has two, so it is told while either is missing). No when all are there, when the
/// thread id cannot be a host name, and when the file cannot be read (the verb would refuse it
/// too).
#[must_use]
pub fn needs_shortcut(root: &Path, project: &str, thread: &str) -> bool {
    needs(root, project, thread).unwrap_or(false)
}
