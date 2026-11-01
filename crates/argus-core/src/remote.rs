//! The local or remote switch: `<COORD_HOME>/remote.json` holds `{ "mode": "local" | "remote" }`.
//! A missing, unreadable or unknown file means local, so nothing calls GitHub unless the owner
//! said so. The file is looked at on every call (one stat) and read again only when its time or
//! size changed.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use crate::remote_state::read_state;

/// Where work goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Nothing leaves the machine: no `gh`, no GitHub API, no push.
    Local,
    /// The owner gave the go.
    Remote,
}

impl Mode {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Local => "local",
            Mode::Remote => "remote",
        }
    }
}

/// Anything that knows the mode. Services take this, so a test can set it without a file.
pub trait ModeSource: Send + Sync {
    /// The mode now.
    fn mode(&self) -> Mode;

    /// Whether the mode is local.
    fn is_local(&self) -> bool {
        self.mode() == Mode::Local
    }
}

/// The switch file of a coord home.
#[must_use]
pub fn switch_file(coord_home: &Path) -> PathBuf {
    coord_home.join("remote.json")
}

/// The integration checkouts of `ciRoots` in `remote.json` (project key, folder), in the order
/// of the file; empty when the file is absent, torn or has none.
#[must_use]
pub fn ci_roots(coord_home: &Path) -> Vec<(String, String)> {
    let roots = read_state(coord_home).ci_roots;
    let text =
        |(key, dir): (&String, &serde_json::Value)| Some((key.clone(), dir.as_str()?.to_owned()));
    roots.iter().filter_map(text).collect()
}

/// The integration checkout of one project, when `coord remote ci-root` set one.
#[must_use]
pub fn ci_root_of(coord_home: &Path, project: &str) -> Option<String> {
    ci_roots(coord_home)
        .into_iter()
        .find(|(key, _)| key == project)
        .map(|(_, dir)| dir)
        .filter(|dir| !dir.is_empty())
}

/// The last full remote sync of a project, as `coord remote synced` recorded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncRecord {
    /// When it was recorded (ISO text).
    pub at: String,
    /// Who recorded it.
    pub by: Option<String>,
    /// What went out; empty when the thread gave no note.
    pub note: String,
}

impl SyncRecord {
    /// The record of one `syncs` entry; `None` when it has no text `at`.
    #[must_use]
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        let text = |key: &str| value.get(key).and_then(serde_json::Value::as_str);
        Some(SyncRecord {
            at: text("at")?.to_owned(),
            by: text("by").map(str::to_owned),
            note: text("note").unwrap_or_default().to_owned(),
        })
    }

    /// When it was recorded, in epoch milliseconds; `None` when `at` is not a date.
    #[must_use]
    pub fn at_ms(&self) -> Option<i64> {
        crate::clock::parse_ms(&self.at)
    }

    /// The wire form: `{ at, by, note }`.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "at": self.at, "by": self.by, "note": self.note })
    }
}

/// The last full remote sync of every project in `remote.json` (`syncs`), in the order of the
/// file; empty when the file is absent, torn or has none.
#[must_use]
pub fn syncs(coord_home: &Path) -> Vec<(String, SyncRecord)> {
    let syncs = read_state(coord_home).syncs;
    let record = |(key, value): (&String, &serde_json::Value)| {
        Some((key.clone(), SyncRecord::from_json(value)?))
    };
    syncs.iter().filter_map(record).collect()
}

/// The last full remote sync of one project.
#[must_use]
pub fn last_sync(coord_home: &Path, project: &str) -> Option<SyncRecord> {
    syncs(coord_home)
        .into_iter()
        .find(|(key, _)| key == project)
        .map(|(_, record)| record)
}

fn parse(text: &str) -> Mode {
    let word = serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v["mode"].as_str().map(str::to_owned));
    match word.as_deref() {
        Some("remote") => Mode::Remote,
        _ => Mode::Local,
    }
}

type Stamp = (Option<SystemTime>, u64);

/// The switch read from its file, cached on the file's time and size.
pub struct Switch {
    file: PathBuf,
    seen: Mutex<Option<(Stamp, Mode)>>,
}

impl Switch {
    /// The switch of a coord home.
    #[must_use]
    pub fn new(coord_home: &Path) -> Self {
        Switch {
            file: switch_file(coord_home),
            seen: Mutex::new(None),
        }
    }

    fn stamp(&self) -> Option<Stamp> {
        let meta = fs::metadata(&self.file).ok()?;
        Some((meta.modified().ok(), meta.len()))
    }
}

impl ModeSource for Switch {
    fn mode(&self) -> Mode {
        let mut seen = self.seen.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(stamp) = self.stamp() else {
            *seen = None;
            return Mode::Local;
        };
        if let Some((have, mode)) = *seen {
            if have == stamp {
                return mode;
            }
        }
        let mode = fs::read_to_string(&self.file).map_or(Mode::Local, |t| parse(&t));
        *seen = Some((stamp, mode));
        mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(home: &Path, text: &str) {
        fs::write(switch_file(home), text).expect("write switch");
    }

    #[test]
    fn a_missing_file_is_local() {
        let home = tempfile::tempdir().expect("temp dir");
        let switch = Switch::new(home.path());
        assert_eq!(switch.mode(), Mode::Local);
        assert!(switch.is_local());
    }

    #[test]
    fn the_file_says_the_mode_and_a_change_is_seen() {
        let home = tempfile::tempdir().expect("temp dir");
        let switch = Switch::new(home.path());
        set(home.path(), r#"{ "mode": "remote" }"#);
        assert_eq!(switch.mode(), Mode::Remote);
        set(home.path(), r#"{ "mode": "local", "note": "x" }"#);
        assert_eq!(switch.mode(), Mode::Local);
        set(home.path(), r#"{"mode":"remote"}"#);
        assert_eq!(switch.mode(), Mode::Remote);
        fs::remove_file(switch_file(home.path())).expect("remove");
        assert_eq!(switch.mode(), Mode::Local);
    }

    #[test]
    fn a_damaged_or_unknown_file_is_local() {
        let home = tempfile::tempdir().expect("temp dir");
        let switch = Switch::new(home.path());
        for text in ["", "{", r#"{"mode":"cloud"}"#, r#"{"mode":1}"#, "[]"] {
            set(home.path(), text);
            assert_eq!(switch.mode(), Mode::Local, "{text}");
        }
    }

    #[test]
    fn the_syncs_read_back_and_an_entry_without_a_time_is_left_out() {
        let home = tempfile::tempdir().expect("temp dir");
        assert!(syncs(home.path()).is_empty());
        set(
            home.path(),
            r#"{"mode":"local","syncs":{"alpha":{"at":"2026-11-01T10:00:00.000Z","by":"alpha","note":"pushed 2 PRs"},"beta":{"note":"no time"},"gamma":{"at":"2026-11-01T11:00:00.000Z"}}}"#,
        );
        let all = syncs(home.path());
        assert_eq!(
            all.iter().map(|(key, _)| key.as_str()).collect::<Vec<_>>(),
            ["alpha", "gamma"]
        );
        let alpha = last_sync(home.path(), "alpha").expect("alpha");
        assert_eq!(alpha.by.as_deref(), Some("alpha"));
        assert_eq!(alpha.note, "pushed 2 PRs");
        assert_eq!(alpha.at_ms(), Some(1_793_527_200_000));
        assert_eq!(last_sync(home.path(), "gamma").expect("gamma").by, None);
        assert_eq!(last_sync(home.path(), "beta"), None);
    }
}
