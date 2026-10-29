//! The remote switch (`docs/local-remote.md`): `<COORD_HOME>/remote.json`, `{ mode, since, by }`.
//! In `local` mode nothing calls `gh`, the GitHub API or the GitHub registry and nothing is
//! pushed; `remote` is the owner's go. A missing or unreadable file means `local`.

use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::clock::now_iso;
use crate::fs::write_atomic;

/// Nothing calls GitHub.
pub const LOCAL: &str = "local";
/// GitHub is used again.
pub const REMOTE: &str = "remote";

/// The switch as the file holds it; `since` and `by` are `None` while the file is absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Switch {
    /// `local` or `remote`.
    pub mode: String,
    /// When the mode was set (ISO text).
    pub since: Option<String>,
    /// Who set it.
    pub by: Option<String>,
}

impl Switch {
    fn default_local() -> Self {
        Switch {
            mode: LOCAL.to_owned(),
            since: None,
            by: None,
        }
    }
}

/// The file of the switch.
#[must_use]
pub fn remote_file(coord_home: &Path) -> PathBuf {
    coord_home.join("remote.json")
}

fn text_of(json: &Value, key: &str) -> Option<String> {
    json.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// The switch of a coord home: `local` when the file is absent or unreadable.
#[must_use]
pub fn read_remote(coord_home: &Path) -> Switch {
    let parsed = std::fs::read_to_string(remote_file(coord_home))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let Some(json) = parsed else {
        return Switch::default_local();
    };
    let mode = text_of(&json, "mode").filter(|m| m == LOCAL || m == REMOTE);
    Switch {
        mode: mode.unwrap_or_else(|| LOCAL.to_owned()),
        since: text_of(&json, "since"),
        by: text_of(&json, "by"),
    }
}

/// Sets the mode (a temp file and a rename) and returns what was written.
///
/// # Errors
/// The folder or the file cannot be written.
pub fn write_remote(coord_home: &Path, mode: &str, by: Option<&str>) -> io::Result<Switch> {
    let next = Switch {
        mode: mode.to_owned(),
        since: Some(now_iso()),
        by: by.map(str::to_owned),
    };
    let body = json!({ "mode": next.mode, "since": next.since, "by": next.by });
    let text = serde_json::to_string_pretty(&body).map_err(io::Error::other)?;
    write_atomic(&remote_file(coord_home), format!("{text}\n"))?;
    Ok(next)
}
