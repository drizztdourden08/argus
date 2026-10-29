//! The local or remote switch: `<COORD_HOME>/remote.json` holds `{ "mode": "local" | "remote" }`.
//! A missing, unreadable or unknown file means local, so nothing calls GitHub unless the owner
//! said so. The file is looked at on every call (one stat) and read again only when its time or
//! size changed.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

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
}
