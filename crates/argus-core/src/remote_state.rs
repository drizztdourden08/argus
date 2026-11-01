//! The content of `<COORD_HOME>/remote.json` as `coord remote` reads and writes it:
//! `{ mode, since, by, ciRoots, syncs }`. `ciRoots` maps a project key to its integration checkout
//! and `syncs` to the last full remote sync of a project (`{ at, by, note }`); both stay across
//! switches of the mode. A missing or unreadable file means `local` with no roots and no syncs.

use std::io;
use std::path::Path;

use serde_json::{Map, Value, json};

use crate::clock::now_iso;
use crate::fs::write_atomic;
use crate::remote::switch_file;

/// Nothing calls GitHub.
pub const LOCAL: &str = "local";
/// GitHub is used again.
pub const REMOTE: &str = "remote";

/// The switch as the file holds it; `since` and `by` are `None` while the file is absent.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// `local` or `remote`.
    pub mode: String,
    /// When the mode was set (ISO text).
    pub since: Option<String>,
    /// Who set it.
    pub by: Option<String>,
    /// Project key to integration checkout, in the order of the file.
    pub ci_roots: Map<String, Value>,
    /// Project key to the last full remote sync `{ at, by, note }`, in the order of the file.
    pub syncs: Map<String, Value>,
}

impl State {
    fn local() -> Self {
        State {
            mode: LOCAL.to_owned(),
            since: None,
            by: None,
            ci_roots: Map::new(),
            syncs: Map::new(),
        }
    }

    fn to_json(&self) -> Value {
        let mut body = json!({ "mode": self.mode, "since": self.since, "by": self.by });
        if !self.ci_roots.is_empty() {
            body["ciRoots"] = Value::Object(self.ci_roots.clone());
        }
        if !self.syncs.is_empty() {
            body["syncs"] = Value::Object(self.syncs.clone());
        }
        body
    }
}

fn object_of(json: &Value, key: &str) -> Map<String, Value> {
    json.get(key)
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

fn text_of(json: &Value, key: &str) -> Option<String> {
    json.get(key).and_then(Value::as_str).map(str::to_owned)
}

/// The state of a coord home: `local` with no roots when the file is absent or unreadable.
#[must_use]
pub fn read_state(coord_home: &Path) -> State {
    let parsed = std::fs::read_to_string(switch_file(coord_home))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let Some(json) = parsed else {
        return State::local();
    };
    let mode = text_of(&json, "mode").filter(|m| m == LOCAL || m == REMOTE);
    State {
        mode: mode.unwrap_or_else(|| LOCAL.to_owned()),
        since: text_of(&json, "since"),
        by: text_of(&json, "by"),
        ci_roots: object_of(&json, "ciRoots"),
        syncs: object_of(&json, "syncs"),
    }
}

fn save(coord_home: &Path, state: &State) -> io::Result<()> {
    let text = serde_json::to_string_pretty(&state.to_json()).map_err(io::Error::other)?;
    write_atomic(&switch_file(coord_home), format!("{text}\n"))
}

/// Sets the mode (a temp file and a rename); the integration checkouts stay. Returns what was
/// written.
///
/// # Errors
/// The folder or the file cannot be written.
pub fn write_mode(coord_home: &Path, mode: &str, by: Option<&str>) -> io::Result<State> {
    let kept = read_state(coord_home);
    let next = State {
        mode: mode.to_owned(),
        since: Some(now_iso()),
        by: by.map(str::to_owned),
        ci_roots: kept.ci_roots,
        syncs: kept.syncs,
    };
    save(coord_home, &next)?;
    Ok(next)
}

/// Sets, or with `None` clears, the integration checkout of a project; the mode stays.
///
/// # Errors
/// The folder or the file cannot be written.
pub fn set_ci_root(coord_home: &Path, project: &str, dir: Option<&str>) -> io::Result<State> {
    let mut next = read_state(coord_home);
    next.since.get_or_insert_with(now_iso);
    match dir {
        Some(dir) => {
            next.ci_roots.insert(project.to_owned(), json!(dir));
        }
        None => {
            next.ci_roots.shift_remove(project);
        }
    }
    save(coord_home, &next)?;
    Ok(next)
}

/// The longest note a sync record keeps.
const NOTE_MAX: usize = 400;

/// Records the last full remote sync of a project (the mode stays): `{ at, by, note }` with the
/// note cut to 400 characters. Returns the record.
///
/// # Errors
/// The folder or the file cannot be written.
pub fn record_sync(
    coord_home: &Path,
    project: &str,
    by: Option<&str>,
    note: &str,
) -> io::Result<Value> {
    let mut next = read_state(coord_home);
    next.since.get_or_insert_with(now_iso);
    let note: String = note.chars().take(NOTE_MAX).collect();
    let record = json!({ "at": now_iso(), "by": by, "note": note });
    next.syncs.insert(project.to_owned(), record.clone());
    save(coord_home, &next)?;
    Ok(record)
}
